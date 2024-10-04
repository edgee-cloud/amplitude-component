mod amplitude_payload;

use amplitude_payload::AmplitudeEvent;
use amplitude_payload::AmplitudePayload;
use exports::provider::{Dict, EdgeeRequest, Guest, Payload};
use std::vec;

wit_bindgen::generate!({world: "data-collection"});
export!(AmplitudeComponent);

struct AmplitudeComponent;

impl Guest for AmplitudeComponent {
    fn page(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        use serde_json::Value as v;

        let mut amplitude_payload = AmplitudePayload::new(cred_map).map_err(|e| e.to_string())?;

        // calculate session_id
        let session_id = edgee_payload.session.session_id.parse::<u64>().unwrap() * 1000;

        // session_end event
        if edgee_payload.session.session_start
            && !edgee_payload.session.previous_session_id.is_empty()
            && edgee_payload.session.session_id != edgee_payload.session.previous_session_id
        {
            let previous_session_id_int = edgee_payload
                .session
                .previous_session_id
                .clone()
                .parse::<u64>()
                .unwrap();
            let previous_session_id = previous_session_id_int * 1000;
            let mut session_end_event =
                AmplitudeEvent::new("session_end", &edgee_payload, previous_session_id)
                    .map_err(|e| e.to_string())?;
            session_end_event.time = edgee_payload.timestamp - 2;

            amplitude_payload.events.push(session_end_event);
        }

        // session_start event
        if edgee_payload.session.session_start {
            let mut session_start_event =
                AmplitudeEvent::new("session_start", &edgee_payload, session_id)
                    .map_err(|e| e.to_string())?;
            session_start_event.time = edgee_payload.timestamp - 1;

            amplitude_payload.events.push(session_start_event);
        }

        // page_view event
        let mut event = AmplitudeEvent::new("[Amplitude] Page Viewed", &edgee_payload, session_id)
            .map_err(|e| e.to_string())?;
        event.time = edgee_payload.timestamp;

        let mut event_props = serde_json::Map::new();

        // set page properties
        let page_location = format!(
            "{}{}",
            edgee_payload.page.url.clone(),
            edgee_payload.page.search.clone()
        );
        event_props.insert(
            "[Amplitude] Page Location".to_string(),
            v::String(page_location.clone()),
        );
        event_props.insert(
            "[Amplitude] Page Path".to_string(),
            v::String(edgee_payload.page.path.clone()),
        );
        event_props.insert(
            "[Amplitude] Page Title".to_string(),
            v::String(edgee_payload.page.title.clone()),
        );
        event_props.insert(
            "[Amplitude] Page URL".to_string(),
            v::String(edgee_payload.page.url.clone()),
        );

        let parsed_url = url::Url::parse(edgee_payload.page.url.clone().as_str()).unwrap();
        if let Some(page_domain) = parsed_url.domain() {
            event_props.insert(
                "[Amplitude] Page Domain".to_string(),
                v::String(page_domain.to_string()),
            );
        }

        if !edgee_payload.page.name.is_empty() {
            event_props.insert(
                "name".to_string(),
                v::String(edgee_payload.page.name.clone()),
            );
        }
        if !edgee_payload.page.category.is_empty() {
            event_props.insert(
                "category".to_string(),
                v::String(edgee_payload.page.category.clone()),
            );
        }
        if !edgee_payload.page.keywords.is_empty() {
            event_props.insert(
                "keywords".to_string(),
                serde_json::to_value(edgee_payload.page.keywords.clone()).unwrap_or_default(),
            );
        }

        // add custom page properties
        if !edgee_payload.page.properties.is_empty() {
            for (key, value) in edgee_payload.page.properties.clone().iter() {
                event_props.insert(key.clone(), value.clone().parse().unwrap_or_default());
            }
        }

        // set campaign properties
        if !edgee_payload.campaign.name.is_empty() {
            event_props.insert(
                String::from("utm_campaign"),
                v::String(edgee_payload.campaign.name.clone()),
            );
        }

        if !edgee_payload.campaign.source.is_empty() {
            event_props.insert(
                String::from("utm_source"),
                v::String(edgee_payload.campaign.source.clone()),
            );
        }

        if !edgee_payload.campaign.medium.is_empty() {
            event_props.insert(
                String::from("utm_medium"),
                v::String(edgee_payload.campaign.medium.clone()),
            );
        }

        if !edgee_payload.campaign.term.is_empty() {
            event_props.insert(
                String::from("utm_term"),
                v::String(edgee_payload.campaign.term.clone()),
            );
        }

        if !edgee_payload.campaign.content.is_empty() {
            event_props.insert(
                String::from("utm_content"),
                v::String(edgee_payload.campaign.content.clone()),
            );
        }

        event.event_properties = Some(serde_json::to_value(event_props).unwrap());

        amplitude_payload.events.push(event);

        Ok(build_edgee_request(amplitude_payload))
    }

    fn track(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if edgee_payload.track.name.is_empty() {
            return Err("Missing event name".to_string());
        }

        let mut amplitude_payload = AmplitudePayload::new(cred_map).map_err(|e| e.to_string())?;

        // calculate session_id
        let session_id = edgee_payload.session.session_id.parse::<u64>().unwrap() * 1000;

        // create a new event and prepare it
        let mut event = AmplitudeEvent::new(&edgee_payload.track.name, &edgee_payload, session_id)
            .map_err(|e| e.to_string())?;

        // set event time
        event.time = edgee_payload.timestamp;

        // set event properties
        let mut properties = serde_json::Map::new();
        if !edgee_payload.track.properties.is_empty() {
            for (key, value) in edgee_payload.track.properties.clone().iter() {
                properties.insert(key.clone(), value.clone().parse().unwrap_or_default());
            }
        }
        if properties.len() > 0 {
            event.event_properties = Some(serde_json::to_value(properties).unwrap());
        }

        // add event to amplitude payload
        amplitude_payload.events.push(event);

        Ok(build_edgee_request(amplitude_payload))
    }

    fn identify(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if edgee_payload.identify.user_id.is_empty()
            || edgee_payload.identify.anonymous_id.is_empty()
        {
            return Err("Missing user id".to_string());
        }

        let mut amplitude_payload = AmplitudePayload::new(cred_map).map_err(|e| e.to_string())?;

        // calculate session_id
        let session_id = edgee_payload.session.session_id.parse::<u64>().unwrap() * 1000;

        // create a new event and prepare it
        let mut event = AmplitudeEvent::new("identify", &edgee_payload, session_id)
            .map_err(|e| e.to_string())?;

        // set event time
        event.time = edgee_payload.timestamp;

        // set event properties
        let mut properties = serde_json::Map::new();
        if !edgee_payload.identify.properties.is_empty() {
            for (key, value) in edgee_payload.identify.properties.clone().iter() {
                properties.insert(key.clone(), value.clone().parse().unwrap_or_default());
            }
        }
        if properties.len() > 0 {
            event.event_properties = Some(serde_json::to_value(properties).unwrap());
        }

        // add event to amplitude payload
        amplitude_payload.events.push(event);

        Ok(build_edgee_request(amplitude_payload))
    }
}

fn build_edgee_request(amplitude_payload: AmplitudePayload) -> EdgeeRequest {
    let mut headers = vec![];
    headers.push((
        String::from("content-type"),
        String::from("application/json"),
    ));

    EdgeeRequest {
        method: exports::provider::HttpMethod::Post,
        url: String::from("https://api2.amplitude.com/2/httpapi"),
        headers,
        body: serde_json::to_string(&amplitude_payload).unwrap(),
    }
}
