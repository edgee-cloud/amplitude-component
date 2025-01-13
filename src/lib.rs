mod amplitude_payload;

use crate::amplitude_payload::parse_value;
use amplitude_payload::AmplitudeEvent;
use amplitude_payload::AmplitudePayload;
use exports::edgee::protocols::provider::Data;
use exports::edgee::protocols::provider::Dict;
use exports::edgee::protocols::provider::EdgeeRequest;
use exports::edgee::protocols::provider::Event;
use exports::edgee::protocols::provider::Guest;
use exports::edgee::protocols::provider::HttpMethod;
use std::vec;

wit_bindgen::generate!({world: "data-collection", path: "wit", with: { "edgee:protocols/provider": generate }});

export!(AmplitudeComponent);

const DEFAULT_ENDPOINT: &str = "https://api2.amplitude.com/2/httpapi";

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
            if !properties.is_empty() {
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

            if !properties.is_empty() {
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
    let headers = vec![(
        String::from("content-type"),
        String::from("application/json"),
    )];

    EdgeeRequest {
        method: HttpMethod::Post,
        url: amplitude_payload.endpoint.clone(),
        headers,
        body: serde_json::to_string(&amplitude_payload).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exports::edgee::protocols::provider::{
        Campaign, Client, Context, EventType, PageData, Session, TrackData, UserData,
    };
    use exports::edgee::protocols::provider::Consent;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    fn sample_user_data(edgee_id: String) -> UserData {
        return UserData {
            user_id: "123".to_string(),
            anonymous_id: "456".to_string(),
            edgee_id: edgee_id,
            properties: vec![
                ("prop1".to_string(), "true".to_string()),
                ("prop2".to_string(), "false".to_string()),
                ("prop3".to_string(), "10".to_string()),
                ("prop4".to_string(), "ok".to_string()),
            ],
        };
    }

    fn sample_user_data_invalid_without_ids() -> UserData {
        return UserData {
            user_id: "".to_string(),      // empty
            anonymous_id: "".to_string(), // empty
            edgee_id: "abc".to_string(),
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
            ],
        };
    }

    fn sample_context(edgee_id: String, locale: String, session_start: bool) -> Context {
        return Context {
            page: sample_page_data(),
            user: sample_user_data(edgee_id),
            client: Client {
                city: "Paris".to_string(),
                ip: "192.168.0.1".to_string(),
                locale: locale,
                timezone: "CET".to_string(),
                user_agent: "Chrome".to_string(),
                user_agent_architecture: "fuck knows".to_string(),
                user_agent_bitness: "64".to_string(),
                user_agent_full_version_list: "abc".to_string(),
                user_agent_version_list: "abc".to_string(),
                user_agent_mobile: "mobile".to_string(),
                user_agent_model: "don't know".to_string(),
                os_name: "MacOS".to_string(),
                os_version: "latest".to_string(),
                screen_width: 1024,
                screen_height: 768,
                screen_density: 2.0,
                continent: "Europe".to_string(),
                country_code: "FR".to_string(),
                country_name: "France".to_string(),
                region: "West Europe".to_string(),
            },
            campaign: Campaign {
                name: "random".to_string(),
                source: "random".to_string(),
                medium: "random".to_string(),
                term: "random".to_string(),
                content: "random".to_string(),
                creative_format: "random".to_string(),
                marketing_tactic: "random".to_string(),
            },
            session: Session {
                session_id: "123".to_string(),
                previous_session_id: "345".to_string(),
                session_count: 2,
                session_start: session_start,
                first_seen: 123,
                last_seen: 123,
            },
        };
    }

    fn sample_page_data() -> PageData {
        return PageData {
            name: "page name".to_string(),
            category: "category".to_string(),
            keywords: vec!["value1".to_string(), "value2".into()],
            title: "page title".to_string(),
            url: "https://example.com/full-url?test=1".to_string(),
            path: "/full-path".to_string(),
            search: "?test=1".to_string(),
            referrer: "https://example.com/another-page".to_string(),
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        };
    }

    fn sample_page_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Page,
            data: Data::Page(sample_page_data()),
            context: sample_context(edgee_id, locale, session_start),
            consent: consent,
        };
    }

    fn sample_track_data(event_name: String) -> TrackData {
        return TrackData {
            name: event_name,
            products: vec![],
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        };
    }

    fn sample_track_event(
        event_name: String,
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Track,
            data: Data::Track(sample_track_data(event_name)),
            context: sample_context(edgee_id, locale, session_start),
            consent: consent,
        };
    }

    fn sample_user_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(sample_user_data(edgee_id.clone())),
            context: sample_context(edgee_id, locale, session_start),
            consent: consent,
        };
    }

    fn sample_user_event_without_ids(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        let user_data = sample_user_data_invalid_without_ids();
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(user_data.clone()),
            context: sample_context(edgee_id, locale, session_start),
            consent: consent,
        };
    }

    fn sample_credentials() -> Vec<(String, String)> {
        return vec![("amplitude_api_key".to_string(), "abc".to_string())];
    }

    #[test]
    fn page_with_consent() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = sample_credentials();
        let result = AmplitudeComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
        assert_eq!(
            edgee_request
                .url
                .starts_with("https://api2.amplitude.com/2/httpapi"),
            true
        );
        // add more checks (headers, querystring, etc.)
    }

    #[test]
    fn page_without_consent() {
        let event = sample_page_event(None, "abc".to_string(), "fr".to_string(), true);
        let credentials = sample_credentials();
        let result = AmplitudeComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn page_with_edgee_id_uuid() {
        let event = sample_page_event(None, Uuid::new_v4().to_string(), "fr".to_string(), true);
        let credentials = sample_credentials();
        let result = AmplitudeComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn page_with_empty_locale() {
        let event = sample_page_event(None, Uuid::new_v4().to_string(), "".to_string(), true);

        let credentials = sample_credentials();
        let result = AmplitudeComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn page_not_session_start() {
        let event = sample_page_event(None, Uuid::new_v4().to_string(), "".to_string(), false);
        let credentials = sample_credentials();
        let result = AmplitudeComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn page_without_measurement_id_fails() {
        let event = sample_page_event(None, "abc".to_string(), "fr".to_string(), true);
        let credentials: Vec<(String, String)> = vec![]; // empty
        let result = AmplitudeComponent::page(event, credentials); // this should panic!
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn track_with_consent() {
        let event = sample_track_event(
            "event-name".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = sample_credentials();
        let result = AmplitudeComponent::track(event, credentials);
        assert_eq!(result.clone().is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn track_with_empty_name_fails() {
        let event = sample_track_event(
            "".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = sample_credentials();
        let result = AmplitudeComponent::track(event, credentials);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn user_event() {
        let event = sample_user_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = sample_credentials();
        let result = AmplitudeComponent::user(event, credentials);

        assert_eq!(result.clone().is_err(), false);
    }

    #[test]
    fn user_event_without_ids_fails() {
        let event = sample_user_event_without_ids(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = sample_credentials();
        let result = AmplitudeComponent::user(event, credentials);

        assert_eq!(result.clone().is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("is not set"),
            true
        );
    }

    #[test]
    fn track_event_without_user_context_properties_and_empty_user_id() {
        let mut event = sample_track_event(
            "event-name".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        event.context.user.properties = vec![]; // empty context user properties
        event.context.user.user_id = "".to_string(); // empty context user id
        let credentials = sample_credentials();
        let result = AmplitudeComponent::track(event, credentials);
        //println!("Error: {}", result.clone().err().unwrap().to_string().as_str());
        assert_eq!(result.clone().is_err(), false);
    }
}
