use std::{collections::HashMap, vec};

use anyhow::anyhow;
use exports::provider::{Dict, EdgeeRequest, Guest, Payload};
use serde::Serialize;

wit_bindgen::generate!({world: "data-collection"});
export!(AmplitudeComponent);

struct AmplitudeComponent;

impl AmplitudeComponent {
    fn build_headers() -> Vec<(String, String)> {
        let mut headers = vec![];
        headers.push((
            String::from("content-type"),
            String::from("application/json"),
        ));
        headers
    }
}

impl Guest for AmplitudeComponent {
    fn page(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        use serde_json::Value as v;

        // calculate session_id
        let session_id = edgee_payload.session.session_id.parse::<u64>().unwrap() * 1000;

        let mut amplitude_request =
            AmplitudeRequest::new(cred_map, vec![]).map_err(|e| e.to_string())?;

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
                AmplitudeEvent::new("session_end", edgee_payload.clone(), previous_session_id);
            session_end_event.time = edgee_payload.timestamp - 2;

            amplitude_request.events.push(session_end_event);
        }

        // session_start event
        if edgee_payload.session.session_start {
            let mut session_start_event =
                AmplitudeEvent::new("session_start", edgee_payload.clone(), session_id);
            session_start_event.time = edgee_payload.timestamp - 1;

            amplitude_request.events.push(session_start_event);
        }

        // page_view event
        let mut event =
            AmplitudeEvent::new("[Amplitude] Page Viewed", edgee_payload.clone(), session_id);
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
                event_props.insert(key.clone(), v::String(value.clone()));
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

        event.event_properties = event_props;

        amplitude_request.events.push(event);

        Ok(EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: String::from("https://api2.amplitude.com/2/httpapi"),
            headers: AmplitudeComponent::build_headers(),
            body: serde_json::to_string(&amplitude_request).map_err(|e| e.to_string())?,
        })
    }

    fn track(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if edgee_payload.track.name.is_empty() {
            return Err("Missing event name".to_string());
        }

        // calculate session_id
        let session_id = edgee_payload.session.session_id.parse::<u64>().unwrap() * 1000;

        // create a new event and prepare it
        let mut event =
            AmplitudeEvent::new(&edgee_payload.track.name, edgee_payload.clone(), session_id);

        // set event time
        event.time = edgee_payload.timestamp;

        // set event properties
        if !edgee_payload.track.properties.is_empty() {
            for (key, value) in edgee_payload.track.properties.clone().iter() {
                event
                    .event_properties
                    .insert(key.clone(), serde_json::to_value(value).unwrap_or_default());
            }
        }

        // create a new amplitude request with the track event in it
        let amplitude_request =
            AmplitudeRequest::new(cred_map, vec![event]).map_err(|e| e.to_string())?;

        Ok(EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: String::from("https://api2.amplitude.com/2/httpapi"),
            headers: AmplitudeComponent::build_headers(),
            body: serde_json::to_string(&amplitude_request).map_err(|e| e.to_string())?,
        })
    }

    fn identify(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if edgee_payload.identify.user_id.is_empty()
            || edgee_payload.identify.anonymous_id.is_empty()
        {
            return Err("Missing user id".to_string());
        }

        // calculate session_id
        let session_id = edgee_payload.session.session_id.parse::<u64>().unwrap() * 1000;

        // create a new event and prepare it
        let mut event = AmplitudeEvent::new("identify", edgee_payload.clone(), session_id);

        // set event time
        event.time = edgee_payload.timestamp;

        // set event properties
        if !edgee_payload.identify.properties.is_empty() {
            for (key, value) in edgee_payload.identify.properties.clone().iter() {
                event
                    .event_properties
                    .insert(key.clone(), serde_json::to_value(value).unwrap_or_default());
            }
        }

        // create a new amplitude request with the identify event in it
        let amplitude_request =
            AmplitudeRequest::new(cred_map, vec![event]).map_err(|e| e.to_string())?;

        Ok(EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: String::from("https://api2.amplitude.com/2/httpapi"),
            headers: AmplitudeComponent::build_headers(),
            body: serde_json::to_string(&amplitude_request).map_err(|e| e.to_string())?,
        })
    }
}

#[derive(Debug, Serialize)]
struct AmplitudeRequest {
    api_key: String,
    events: Vec<AmplitudeEvent>,
    options: AmplitudeOptions,
}

impl AmplitudeRequest {
    fn new(cred_map: Dict, events: Vec<AmplitudeEvent>) -> anyhow::Result<Self> {
        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let api_key = match cred.get("amplitude_api_key") {
            Some(key) => key,
            None => return Err(anyhow!("Missing Amplitude API KEY")),
        }
        .to_string();

        let options = AmplitudeOptions { min_id_length: 1 };

        Ok(Self {
            api_key,
            options,
            events,
        })
    }
}

#[derive(Debug, Default, Serialize)]
struct AmplitudeEvent {
    #[serde(skip_serializing_if = "String::is_empty")]
    user_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    device_id: String,
    event_type: String,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    event_properties: serde_json::Map<String, serde_json::Value>,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    user_properties: serde_json::Map<String, serde_json::Value>,
    time: i64,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    groups: HashMap<String, String>,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    group_properties: serde_json::Map<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip_user_properties_sync: Option<bool>,
    #[serde(skip_serializing_if = "String::is_empty")]
    app_version: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    platform: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    os_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    os_version: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    device_brand: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    device_manufacturer: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    device_model: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    carrier: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    country: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    region: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    city: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    dma: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    revenue: Option<f32>,
    #[serde(skip_serializing_if = "String::is_empty")]
    product_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    revenue_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location_lat: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location_lng: Option<f32>,
    #[serde(skip_serializing_if = "String::is_empty")]
    ip: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    idfa: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    idfv: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    adid: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    android_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<u64>,
    #[serde(skip_serializing_if = "String::is_empty")]
    insert_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan: Option<PlanProperties>,
    #[serde(skip_serializing_if = "String::is_empty")]
    user_agent: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    library: String,
}

impl AmplitudeEvent {
    fn new(event_type: &str, edgee_payload: Payload, session_id: u64) -> Self {
        use serde_json::Value as v;

        let mut event = Self::default();
        event.event_type = String::from(event_type);
        event.library = String::from("Edgee");
        event.platform = String::from("Web");

        let mut user_props = serde_json::Map::new();

        // identify
        if !edgee_payload.identify.user_id.is_empty() {
            event.user_id = edgee_payload.identify.user_id.clone();
        }
        if !edgee_payload.identify.anonymous_id.is_empty() {
            user_props.insert(
                "anonymous_id".to_string(),
                v::String(edgee_payload.identify.anonymous_id.clone()),
            );
        }

        // set edgee_id as device_id
        // todo continuity of the DeviceID
        event.device_id = edgee_payload.identify.edgee_id.clone();

        // set user_props HashMap<String, v>
        let mut set_user_props = serde_json::Map::new();
        let mut set_once_user_props = serde_json::Map::new();
        if !edgee_payload.page.referrer.is_empty() {
            set_user_props.insert(
                "referrer".to_string(),
                v::String(edgee_payload.page.referrer.clone()),
            );
            set_once_user_props.insert(
                "initial_referrer".to_string(),
                v::String(edgee_payload.page.referrer.clone()),
            );

            let parsed_referrer = url::Url::parse(&edgee_payload.page.referrer).unwrap();
            if let Some(referring_domain) = parsed_referrer.domain() {
                set_user_props.insert(
                    "referring_domain".to_string(),
                    v::String(referring_domain.to_string()),
                );
                set_once_user_props.insert(
                    "initial_referring_domain".to_string(),
                    v::String(referring_domain.to_string()),
                );
            }
        }
        // if edgee_payload.campaign is Some
        if !edgee_payload.campaign.name.is_empty() {
            set_user_props.insert(
                "utm_campaign".to_string(),
                v::String(edgee_payload.campaign.name.clone()),
            );
            set_once_user_props.insert(
                "initial_utm_campaign".to_string(),
                v::String(edgee_payload.campaign.name.clone()),
            );
        }
        if !edgee_payload.campaign.source.is_empty() {
            set_user_props.insert(
                "utm_source".to_string(),
                v::String(edgee_payload.campaign.source.clone()),
            );
            set_once_user_props.insert(
                "initial_utm_source".to_string(),
                v::String(edgee_payload.campaign.source.clone()),
            );
        }
        if !edgee_payload.campaign.medium.is_empty() {
            set_user_props.insert(
                "utm_medium".to_string(),
                v::String(edgee_payload.campaign.medium.clone()),
            );
            set_once_user_props.insert(
                "initial_utm_medium".to_string(),
                v::String(edgee_payload.campaign.medium.clone()),
            );
        }
        if !edgee_payload.campaign.term.is_empty() {
            set_user_props.insert(
                "utm_term".to_string(),
                v::String(edgee_payload.campaign.term.clone()),
            );
            set_once_user_props.insert(
                "initial_utm_term".to_string(),
                v::String(edgee_payload.campaign.term.clone()),
            );
        }
        if !edgee_payload.campaign.content.is_empty() {
            set_user_props.insert(
                "utm_content".to_string(),
                v::String(edgee_payload.campaign.content.clone()),
            );
            set_once_user_props.insert(
                "initial_utm_content".to_string(),
                v::String(edgee_payload.campaign.content.clone()),
            );
        }

        user_props.insert(
            "$set".to_string(),
            serde_json::to_value(set_user_props).unwrap_or_default(),
        );
        user_props.insert(
            "$setOnce".to_string(),
            serde_json::to_value(set_once_user_props).unwrap_or_default(),
        );

        // add custom user properties
        if !edgee_payload.identify.properties.is_empty() {
            for (key, value) in edgee_payload.identify.properties.clone().iter() {
                user_props.insert(key.clone(), value.clone().parse().unwrap_or_default());
            }
        }
        event.user_properties = user_props;

        event.user_agent = edgee_payload.client.user_agent.clone();
        event.language = edgee_payload.client.locale.clone();
        event.ip = edgee_payload.client.ip.clone();
        if session_id != 0 {
            event.session_id = Some(session_id);
        }

        event.os_name = edgee_payload.client.os_name.clone();
        event.os_version = edgee_payload.client.os_version.clone();
        event.device_model = edgee_payload.client.user_agent_model.clone();

        if !edgee_payload.client.city.is_empty() {
            event.city = edgee_payload.client.city.clone();
        }
        if !edgee_payload.client.region.is_empty() {
            event.region = edgee_payload.client.region.clone();
        }
        if !edgee_payload.client.country_code.is_empty() {
            event.country = edgee_payload.client.country_code.clone();
        }

        // todo missing following fields
        // missing event.device_brand
        // missing event.device_manufacturer
        // missing event.carrier
        // missing event.dma
        // missing event.price
        // missing event.quantity
        // missing event.revenue
        // missing event.product_id
        // missing event.revenue_type
        // missing event.location_lat
        // missing event.location_lng
        // missing event.idfa
        // missing event.idfv
        // missing event.adid
        // missing event.android_id
        // missing event.event_id
        // missing event.plan

        event
    }
}

#[derive(Debug, Serialize)]
struct AmplitudeOptions {
    min_id_length: u32,
}

#[derive(Debug, Serialize)]
struct PlanProperties {
    branch: String,
    source: String,
    version: String,
}
