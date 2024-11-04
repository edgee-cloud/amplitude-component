mod amplitude_payload;

use amplitude_payload::AmplitudeEvent;
use amplitude_payload::AmplitudePayload;
use exports::provider::{Data, Dict, EdgeeRequest, Event, Guest};
use std::vec;
use crate::amplitude_payload::parse_value;

wit_bindgen::generate!({world: "data-collection"});
export!(AmplitudeComponent);

struct AmplitudeComponent;

impl Guest for AmplitudeComponent {
    fn page(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        use serde_json::Value as v;

        if let Data::Page(ref data) = edgee_event.data {
            let mut amplitude_payload =
                AmplitudePayload::new(cred_map).map_err(|e| e.to_string())?;

            // calculate session_id
            let session_id = edgee_event
                .context
                .session
                .session_id
                .parse::<u64>()
                .unwrap()
                * 1000;

            // session_end event
            if edgee_event.context.session.session_start
                && !edgee_event.context.session.previous_session_id.is_empty()
                && edgee_event.context.session.session_id
                    != edgee_event.context.session.previous_session_id
            {
                let previous_session_id_int = edgee_event
                    .context
                    .session
                    .previous_session_id
                    .clone()
                    .parse::<u64>()
                    .unwrap();
                let previous_session_id = previous_session_id_int * 1000;
                let mut session_end_event =
                    AmplitudeEvent::new("session_end", &edgee_event, previous_session_id)
                        .map_err(|e| e.to_string())?;
                session_end_event.time = edgee_event.timestamp - 2;

                amplitude_payload.events.push(session_end_event);
            }

            // session_start event
            if edgee_event.context.session.session_start {
                let mut session_start_event =
                    AmplitudeEvent::new("session_start", &edgee_event, session_id)
                        .map_err(|e| e.to_string())?;
                session_start_event.time = edgee_event.timestamp - 1;

                amplitude_payload.events.push(session_start_event);
            }

            // page_view event
            let mut event =
                AmplitudeEvent::new("[Amplitude] Page Viewed", &edgee_event, session_id)
                    .map_err(|e| e.to_string())?;
            event.time = edgee_event.timestamp;

            let mut event_props = serde_json::Map::new();

            // set page properties
            let page_location = format!("{}{}", data.url.clone(), data.search.clone());
            event_props.insert(
                "[Amplitude] Page Location".to_string(),
                v::String(page_location.clone()),
            );
            event_props.insert(
                "[Amplitude] Page Path".to_string(),
                v::String(data.path.clone()),
            );
            event_props.insert(
                "[Amplitude] Page Title".to_string(),
                v::String(data.title.clone()),
            );
            event_props.insert(
                "[Amplitude] Page URL".to_string(),
                v::String(data.url.clone()),
            );

            let parsed_url = url::Url::parse(data.url.clone().as_str()).unwrap();
            if let Some(page_domain) = parsed_url.domain() {
                event_props.insert(
                    "[Amplitude] Page Domain".to_string(),
                    v::String(page_domain.to_string()),
                );
            }

            if !data.name.is_empty() {
                event_props.insert("name".to_string(), v::String(data.name.clone()));
            }
            if !data.category.is_empty() {
                event_props.insert("category".to_string(), v::String(data.category.clone()));
            }
            if !data.keywords.is_empty() {
                event_props.insert(
                    "keywords".to_string(),
                    serde_json::to_value(data.keywords.clone()).unwrap_or_default(),
                );
            }

            // add custom page properties
            if !data.properties.is_empty() {
                for (key, value) in data.properties.clone().iter() {
                    event_props.insert(key.clone(), parse_value(value));
                }
            }

            // set campaign properties
            if !edgee_event.context.campaign.name.is_empty() {
                event_props.insert(
                    String::from("utm_campaign"),
                    v::String(edgee_event.context.campaign.name.clone()),
                );
            }

            if !edgee_event.context.campaign.source.is_empty() {
                event_props.insert(
                    String::from("utm_source"),
                    v::String(edgee_event.context.campaign.source.clone()),
                );
            }

            if !edgee_event.context.campaign.medium.is_empty() {
                event_props.insert(
                    String::from("utm_medium"),
                    v::String(edgee_event.context.campaign.medium.clone()),
                );
            }

            if !edgee_event.context.campaign.term.is_empty() {
                event_props.insert(
                    String::from("utm_term"),
                    v::String(edgee_event.context.campaign.term.clone()),
                );
            }

            if !edgee_event.context.campaign.content.is_empty() {
                event_props.insert(
                    String::from("utm_content"),
                    v::String(edgee_event.context.campaign.content.clone()),
                );
            }

            event.event_properties = Some(serde_json::to_value(event_props).unwrap());

            amplitude_payload.events.push(event);

            Ok(build_edgee_request(amplitude_payload))
        } else {
            Err("Missing page data".to_string())
        }
    }

    fn track(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Track(ref data) = edgee_event.data {
            if data.name.is_empty() {
                return Err("Missing event name".to_string());
            }

            let mut amplitude_payload =
                AmplitudePayload::new(cred_map).map_err(|e| e.to_string())?;

            // calculate session_id
            let session_id = edgee_event
                .context
                .session
                .session_id
                .parse::<u64>()
                .unwrap()
                * 1000;

            // create a new event and prepare it
            let mut event = AmplitudeEvent::new(&data.name, &edgee_event, session_id)
                .map_err(|e| e.to_string())?;

            // set event time
            event.time = edgee_event.timestamp;

            // set event properties
            let mut properties = serde_json::Map::new();
            if !data.properties.is_empty() {
                for (key, value) in data.properties.clone().iter() {
                    properties.insert(key.clone(), parse_value(value));
                }
            }
            if properties.len() > 0 {
                event.event_properties = Some(serde_json::to_value(properties).unwrap());
            }

            // add event to amplitude payload
            amplitude_payload.events.push(event);

            Ok(build_edgee_request(amplitude_payload))
        } else {
            Err("Missing track data".to_string())
        }
    }

    fn user(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        use serde_json::Value as v;

        if let Data::User(ref data) = edgee_event.data {
            if data.user_id.is_empty() && data.anonymous_id.is_empty() {
                return Err("user_id or anonymous_id is not set".to_string());
            }

            let mut amplitude_payload =
                AmplitudePayload::new(cred_map).map_err(|e| e.to_string())?;

            // calculate session_id
            let session_id = edgee_event
                .context
                .session
                .session_id
                .parse::<u64>()
                .unwrap()
                * 1000;

            // create a new event and prepare it
            let mut event = AmplitudeEvent::new("identify", &edgee_event, session_id)
                .map_err(|e| e.to_string())?;

            // set event time
            event.time = edgee_event.timestamp;

            // identify
            if !data.user_id.is_empty() {
                event.user_id = Option::from(data.user_id.clone());
            }

            // set user properties
            let mut properties = serde_json::Map::new();

            if !data.anonymous_id.is_empty() {
                properties.insert(
                    "anonymous_id".to_string(),
                    v::String(data.anonymous_id.clone()),
                );
            }

            if !data.properties.is_empty() {
                for (key, value) in data.properties.clone().iter() {
                    properties.insert(key.clone(), parse_value(value));
                }
            }

            if properties.len() > 0 {
                event.user_properties = Some(serde_json::to_value(properties).unwrap());
            }

            // add event to amplitude payload
            amplitude_payload.events.push(event);

            Ok(build_edgee_request(amplitude_payload))
        } else {
            Err("Missing user data".to_string())
        }
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
