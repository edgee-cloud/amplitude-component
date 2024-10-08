use crate::exports::provider::{Dict, Payload};
use anyhow::anyhow;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug, Default)]
pub(crate) struct AmplitudePayload {
    api_key: String,
    pub(crate) events: Vec<AmplitudeEvent>,
    options: AmplitudeOptions,
}

impl AmplitudePayload {
    pub(crate) fn new(cred_map: Dict) -> anyhow::Result<Self> {
        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let api_key = match cred.get("amplitude_api_key") {
            Some(key) => key,
            None => return Err(anyhow!("Missing Amplitude API KEY")),
        }
        .to_string();

        Ok(Self {
            api_key,
            options: AmplitudeOptions {
                min_id_length: Option::from(1),
            },
            events: vec![],
        })
    }
}

#[derive(Serialize, Debug, Default)]
pub(crate) struct AmplitudeEvent {
    #[serde(rename = "user_id", skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(rename = "device_id", skip_serializing_if = "Option::is_none")]
    device_id: Option<String>,
    event_type: String,
    #[serde(rename = "event_properties", skip_serializing_if = "Option::is_none")]
    pub(crate) event_properties: Option<serde_json::Value>,
    #[serde(rename = "user_properties", skip_serializing_if = "Option::is_none")]
    user_properties: Option<serde_json::Value>,
    pub(crate) time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    groups: Option<HashMap<String, String>>,
    #[serde(rename = "group_properties", skip_serializing_if = "Option::is_none")]
    group_properties: Option<HashMap<String, serde_json::Value>>,
    #[serde(
        rename = "$skip_user_properties_sync",
        skip_serializing_if = "Option::is_none"
    )]
    skip_user_properties_sync: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    app_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    os_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    os_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_brand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    carrier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dma: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    revenue: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    revenue_type: Option<String>,
    #[serde(rename = "location_lat", skip_serializing_if = "Option::is_none")]
    location_lat: Option<f32>,
    #[serde(rename = "location_lng", skip_serializing_if = "Option::is_none")]
    location_lng: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idfa: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idfv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    adid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    android_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    insert_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan: Option<PlanProperties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    library: Option<String>,
}

impl AmplitudeEvent {
    pub(crate) fn new(
        event_type: &str,
        edgee_payload: &Payload,
        session_id: u64,
    ) -> anyhow::Result<Self> {
        use serde_json::Value as v;

        let mut event = Self::default();
        event.event_type = String::from(event_type);
        event.library = Option::from(String::from("Edgee"));
        event.platform = Option::from(String::from("Web"));

        let mut user_props = serde_json::Map::new();

        // identify
        if !edgee_payload.identify.user_id.is_empty() {
            event.user_id = Option::from(edgee_payload.identify.user_id.clone());
        }
        if !edgee_payload.identify.anonymous_id.is_empty() {
            user_props.insert(
                "anonymous_id".to_string(),
                v::String(edgee_payload.identify.anonymous_id.clone()),
            );
        }

        // set edgee_id as device_id
        // todo continuity of the DeviceID
        event.device_id = Option::from(edgee_payload.identify.edgee_id.clone());

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

            let parsed_referrer = url::Url::parse(&edgee_payload.page.referrer)?;
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

        // utm_campaign, utm_source, utm_medium, utm_term, utm_content
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
        event.user_properties = Some(serde_json::to_value(user_props)?);

        event.user_agent = Option::from(edgee_payload.client.user_agent.clone());
        event.language = Option::from(edgee_payload.client.locale.clone());
        event.ip = Option::from(edgee_payload.client.ip.clone());
        if session_id != 0 {
            event.session_id = Some(session_id);
        }

        if !edgee_payload.client.os_name.is_empty() {
            event.os_name = Option::from(edgee_payload.client.os_name.clone());
        }
        if !edgee_payload.client.os_version.is_empty() {
            event.os_version = Option::from(edgee_payload.client.os_version.clone());
        }
        if !edgee_payload.client.user_agent_model.is_empty() {
            event.device_model = Option::from(edgee_payload.client.user_agent_model.clone());
        }

        if !edgee_payload.client.city.is_empty() {
            event.city = Option::from(edgee_payload.client.city.clone());
        }
        if !edgee_payload.client.region.is_empty() {
            event.region = Option::from(edgee_payload.client.region.clone());
        }
        if !edgee_payload.client.country_code.is_empty() {
            event.country = Option::from(edgee_payload.client.country_code.clone());
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

        Ok(event)
    }
}

#[derive(Serialize, Debug, Default)]
struct AmplitudeOptions {
    #[serde(rename = "min_id_length", skip_serializing_if = "Option::is_none")]
    min_id_length: Option<i32>,
}

#[derive(Serialize, Debug, Default)]
struct PlanProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}