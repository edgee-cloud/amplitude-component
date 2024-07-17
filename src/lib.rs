use std::collections::HashMap;

use anyhow::anyhow;
use exports::provider::{Dict, EdgeeRequest, Guest, Payload};
use serde::Serialize;

wit_bindgen::generate!({world: "data-collection"});
export!(AmplitudeComponent);

struct AmplitudeComponent;

impl Guest for AmplitudeComponent {
    fn page(p: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        let mut headers = HashMap::new();
        headers.insert("content-type", String::from("application/json"));
        headers.insert("user-agent", p.client.user_agent.clone());
        headers.insert("x-forwarded-for", p.client.ip.clone());

        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let amplitude_request = AmplitudeRequest::new(p, cred).map_err(|e| e.to_string())?;

        let edgee_request = EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: String::from("https://api2.amplitude.com/2/httpapi"),
            headers: headers
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            body: serde_json::to_string(&amplitude_request).map_err(|e| e.to_string())?,
        };

        return Ok(edgee_request);
    }

    fn track(p: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        let mut headers = HashMap::new();
        headers.insert("content-type", String::from("application/json"));
        headers.insert("user-agent", p.client.user_agent.clone());
        headers.insert("x-forwarded-for", p.client.ip.clone());

        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let amplitude_request = AmplitudeRequest::new(p, cred).map_err(|e| e.to_string())?;

        let edgee_request = EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: String::from("https://api2.amplitude.com/2/httpapi"),
            headers: headers
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            body: serde_json::to_string(&amplitude_request).map_err(|e| e.to_string())?,
        };

        return Ok(edgee_request);
    }

    fn identify(p: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        let mut headers = HashMap::new();
        headers.insert("content-type", String::from("application/json"));
        headers.insert("user-agent", p.client.user_agent.clone());
        headers.insert("x-forwarded-for", p.client.ip.clone());

        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let amplitude_request = AmplitudeRequest::new(p, cred).map_err(|e| e.to_string())?;

        let edgee_request = EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: String::from("https://api2.amplitude.com/2/httpapi"),
            headers: headers
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            body: serde_json::to_string(&amplitude_request).map_err(|e| e.to_string())?,
        };

        return Ok(edgee_request);
    }
}

#[derive(Debug, Serialize)]
struct AmplitudeRequest {
    api_key: String,
    events: Vec<AmplitudeEvent>,
    options: AmplitudeOptions,
}

impl AmplitudeRequest {
    fn new(payload: Payload, cred: HashMap<String, String>) -> anyhow::Result<Self> {
        let api_key = match cred.get("amplitude_api_key") {
            Some(key) => key,
            None => return Err(anyhow!("Missing Amplitude API KEY")),
        }
        .to_string();

        let options = AmplitudeOptions { min_id_length: 1 };

        let events = match payload.event_type {
            exports::provider::EventType::Page => AmplitudeRequest::page_events(payload)?,
            // exports::provider::EventType::Track => AmplitudeRequest::track_events(payload)?,
            // exports::provider::EventType::Identify => AmplitudeEvent::identify_events(payload)?,
            _ => todo!(),
        };

        Ok(Self {
            api_key,
            options,
            events,
        })
    }

    fn page_events(edgee: Payload) -> anyhow::Result<Vec<AmplitudeEvent>> {
        let mut events = vec![];

        if let Some(evt) = AmplitudeEvent::session_end(edgee.clone()) {
            events.push(evt);
        }

        if let Some(evt) = AmplitudeEvent::session_start(edgee.clone()) {
            events.push(evt);
        }

        let mut evt = AmplitudeEvent::new("[Amplitude] Page Viewed", edgee.clone());
        evt.time = edgee.timestamp;

        evt.event_properties = serde_json::Map::new();

        let full_url = format!("{}{}", edgee.page.url.clone(), edgee.page.search.clone());
        evt.event_properties.insert(
            String::from("[Amplitude] Page Location"),
            serde_json::Value::String(full_url),
        );
        evt.event_properties.insert(
            String::from("[Amplitude] Page Path"),
            serde_json::Value::String(edgee.page.path.clone()),
        );
        evt.event_properties.insert(
            String::from("[Amplitude] Page Title"),
            serde_json::Value::String(edgee.page.title.clone()),
        );
        evt.event_properties.insert(
            String::from("[Amplitude] Page URL"),
            serde_json::Value::String(edgee.page.url.clone()),
        );

        let parsed_url = url::Url::parse(&edgee.page.url.clone())?;
        if let Some(domain) = parsed_url.domain().map(String::from) {
            evt.event_properties.insert(
                String::from("[Amplitude] Page Domain"),
                serde_json::Value::String(domain),
            );
        }

        if !edgee.page.name.is_empty() {
            evt.event_properties.insert(
                String::from("name"),
                serde_json::Value::String(edgee.page.name.clone()),
            );
        }

        if !edgee.page.category.is_empty() {
            evt.event_properties.insert(
                String::from("category"),
                serde_json::Value::String(edgee.page.category.clone()),
            );
        }

        if !edgee.page.keywords.is_empty() {
            evt.event_properties.insert(
                String::from("keywords"),
                serde_json::to_value(edgee.page.keywords.clone()).unwrap_or_default(),
            );
        }

        if !edgee.page.properties.is_empty() {
            for (key, value) in edgee.page.properties.iter() {
                evt.event_properties
                    .insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }

        if !edgee.campaign.name.is_empty() {
            evt.event_properties.insert(
                String::from("utm_campaign"),
                serde_json::Value::String(edgee.campaign.name.clone()),
            );
        }

        if !edgee.campaign.source.is_empty() {
            evt.event_properties.insert(
                String::from("utm_source"),
                serde_json::Value::String(edgee.campaign.source.clone()),
            );
        }

        if !edgee.campaign.medium.is_empty() {
            evt.event_properties.insert(
                String::from("utm_medium"),
                serde_json::Value::String(edgee.campaign.medium.clone()),
            );
        }

        if !edgee.campaign.term.is_empty() {
            evt.event_properties.insert(
                String::from("utm_term"),
                serde_json::Value::String(edgee.campaign.term.clone()),
            );
        }

        if !edgee.campaign.content.is_empty() {
            evt.event_properties.insert(
                String::from("utm_content"),
                serde_json::Value::String(edgee.campaign.content.clone()),
            );
        }

        events.push(evt);

        return Ok(events);
    }
}

#[derive(Debug, Default, Serialize)]
struct AmplitudeEvent {
    #[serde(skip_serializing_if = "String::is_empty")]
    user_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    device_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    event_type: String,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    event_properties: serde_json::Map<String, serde_json::Value>,
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
    fn new(event_type: &str, edgee: Payload) -> Self {
        use serde_json::Value as v;

        let mut event = Self::default();
        event.event_type = String::from(event_type);
        event.library = String::from("Edgee");
        event.platform = String::from("Web");
        event.device_id = edgee.identify.edgee_id;
        event.user_agent = edgee.client.user_agent.clone();
        event.ip = edgee.client.ip.clone();
        event.language = edgee.client.locale.clone();
        event.os_name = edgee.client.os_name.clone();
        event.os_version = edgee.client.os_version.clone();
        event.device_model = edgee.client.user_agent_model.clone();

        if !edgee.client.city.is_empty() {
            event.city = edgee.client.city.clone();
        }

        if !edgee.client.region.is_empty() {
            event.region = edgee.client.region.clone();
        }

        if !edgee.client.country_code.is_empty() {
            event.country = edgee.client.country_code.clone();
        }

        let mut user_props = serde_json::Map::new();
        if !edgee.identify.anonymous_id.is_empty() {
            user_props.insert(
                String::from("anonyous_id"),
                v::String(edgee.identify.anonymous_id),
            );
        }

        let mut set_user_props = serde_json::Map::new();
        let mut set_once_user_props = serde_json::Map::new();

        if !edgee.page.referrer.is_empty() {
            set_user_props.insert(
                String::from("referrer"),
                v::String(edgee.page.referrer.clone()),
            );
            set_once_user_props.insert(
                String::from("initial_referrer"),
                v::String(edgee.page.referrer.clone()),
            );

            let parsed_referer = url::Url::parse(&edgee.page.referrer).unwrap();
            if let Some(domain) = parsed_referer.domain().map(String::from) {
                set_user_props.insert(String::from("referring_domain"), v::String(domain.clone()));
                set_once_user_props.insert(
                    String::from("initial_referring_domain"),
                    v::String(domain.clone()),
                );
            }
        }

        if !edgee.campaign.name.is_empty() {
            set_user_props.insert(
                String::from("utm_campaign"),
                v::String(edgee.campaign.name.clone()),
            );
            set_once_user_props.insert(
                String::from("initial_utm_campaign"),
                v::String(edgee.campaign.name.clone()),
            );
        }

        if !edgee.campaign.source.is_empty() {
            set_user_props.insert(
                String::from("utm_source"),
                v::String(edgee.campaign.source.clone()),
            );
            set_once_user_props.insert(
                String::from("initial_utm_source"),
                v::String(edgee.campaign.source.clone()),
            );
        }

        if !edgee.campaign.medium.is_empty() {
            set_user_props.insert(
                String::from("utm_medium"),
                v::String(edgee.campaign.medium.clone()),
            );
            set_once_user_props.insert(
                String::from("initial_utm_medium"),
                v::String(edgee.campaign.medium.clone()),
            );
        }

        if !edgee.campaign.term.is_empty() {
            set_user_props.insert(
                String::from("utm_term"),
                v::String(edgee.campaign.term.clone()),
            );
            set_once_user_props.insert(
                String::from("initial_utm_term"),
                v::String(edgee.campaign.term.clone()),
            );
        }

        if !edgee.campaign.content.is_empty() {
            set_user_props.insert(
                String::from("utm_content"),
                v::String(edgee.campaign.content.clone()),
            );
            set_once_user_props.insert(
                String::from("initial_utm_content"),
                v::String(edgee.campaign.content.clone()),
            );
        }

        user_props.insert(String::from("$set"), v::Object(set_user_props));
        user_props.insert(String::from("$setOnce"), v::Object(set_once_user_props));

        if !edgee.identify.properties.is_empty() {
            for (key, value) in edgee.identify.properties.iter() {
                user_props.insert(key.clone(), v::String(value.clone()));
            }
        }

        event.event_properties = user_props;

        return event;
    }

    fn session_start(edgee: Payload) -> Option<Self> {
        if edgee.session.session_start {
            let mut event = AmplitudeEvent::new("session_start", edgee.clone());
            event.time = edgee.timestamp - 1;
            event.session_id = edgee
                .session
                .session_id
                .parse::<u64>()
                .map(|i| i * 1000)
                .ok()
                .or(Some(0));

            return Some(event);
        } else {
            return None;
        }
    }

    fn session_end(edgee: Payload) -> Option<Self> {
        if edgee.session.session_start
            && !edgee.session.previous_session_id.is_empty()
            && edgee.session.session_id != edgee.session.previous_session_id
        {
            let mut event = AmplitudeEvent::new("session_end", edgee.clone());
            event.time = edgee.timestamp - 2;
            event.session_id = edgee
                .session
                .previous_session_id
                .parse::<u64>()
                .map(|i| i * 1000)
                .ok()
                .or(Some(0));

            return Some(event);
        } else {
            return None;
        }
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
