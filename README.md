<div align="center">
<p align="center">
  <a href="https://www.edgee.cloud">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://cdn.edgee.cloud/img/component-dark.svg">
      <img src="https://cdn.edgee.cloud/img/component.svg" height="100" alt="Edgee">
    </picture>
  </a>
</p>
</div>


<h1 align="center">Amplitude Component for Edgee</h1>

[![Coverage Status](https://coveralls.io/repos/github/edgee-cloud/amplitude-component/badge.svg)](https://coveralls.io/github/edgee-cloud/amplitude-component)
[![GitHub issues](https://img.shields.io/github/issues/edgee-cloud/amplitude-component.svg)](https://github.com/edgee-cloud/amplitude-component/issues)
[![Edgee Component Registry](https://img.shields.io/badge/Edgee_Component_Registry-Public-green.svg)](https://www.edgee.cloud/edgee/amplitude)

This component enables seamless integration between [Edgee](https://www.edgee.cloud) and [Amplitude](https://amplitude.com), allowing you to collect and forward analytics data while respecting user privacy settings.

## Quick Start

1. Download the latest component version from our [releases page](../../releases)
2. Place the `amplitude.wasm` file in your server (e.g., `/var/edgee/components`)
3. Add the following configuration to your `edgee.toml`:

```toml
[[components.data_collection]]
id = "amplitude"
file = "/var/edgee/components/amplitude.wasm"
settings.amplitude_api_key = "..."  # Your Amplitude API Key
```

## Event Handling

### Event Mapping
The component maps Edgee events to Amplitude events as follows:

| Edgee Event | Amplitude Event | Description |
|-------------|----------------|-------------|
| Page        | `[Amplitude] Page Viewed` | Triggered when a user views a page (includes session_start/session_end if needed) |
| Track       | Custom Event | Uses the provided event name directly |
| User        | `identify` | Used for user identification |

### User Event Handling
User events in Amplitude serve multiple purposes:
- Triggers an `identify` call to Amplitude
- Stores `user_id`, `anonymous_id`, and `properties` on the user's device
- Enriches subsequent Page and Track events with user data
- Enables proper user attribution across sessions

## Configuration Options

### Basic Configuration
```toml
[[components.data_collection]]
id = "amplitude"
file = "/var/edgee/components/amplitude.wasm"
settings.amplitude_api_key = "..."

# Optional configurations
settings.amplitude_endpoint = "..."        # The default value is https://api2.amplitude.com/2/httpapi
settings.edgee_anonymization = true        # Enable/disable data anonymization
settings.edgee_default_consent = "pending" # Set default consent status
```

### Event Controls
Control which events are forwarded to Amplitude:
```toml
settings.edgee_page_event_enabled = true   # Enable/disable page view tracking
settings.edgee_track_event_enabled = true  # Enable/disable custom event tracking
settings.edgee_user_event_enabled = true   # Enable/disable user identification
```

### Consent Management
Before sending events to Amplitude, you can set the user consent using the Edgee SDK: 
```javascript
edgee.consent("granted");
```

Or using the Data Layer:
```html
<script id="__EDGEE_DATA_LAYER__" type="application/json">
  {
    "data_collection": {
      "consent": "granted"
    }
  }
</script>
```

If the consent is not set, the component will use the default consent status.

| Consent | Anonymization |
|---------|---------------|
| pending | true          |
| denied  | true          |
| granted | false         |


## Development

### Building from Source
Prerequisites:
- [Rust](https://www.rust-lang.org/tools/install)
- WASM target: `rustup target add wasm32-wasip2`
- wit-deps: `cargo install wit-deps`

Build command:
```bash
make wit-deps
make build
```

### Contributing
Interested in contributing? Read our [contribution guidelines](./CONTRIBUTING.md)

### Security
Report security vulnerabilities to [security@edgee.cloud](mailto:security@edgee.cloud)
