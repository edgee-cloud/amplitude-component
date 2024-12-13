<div align="center">

<p align="center">
  <a href="https://www.edgee.cloud">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://cdn.edgee.cloud/img/favicon-dark.svg">
      <img src="https://cdn.edgee.cloud/img/favicon.svg" height="100" alt="Edgee">
    </picture>
    <h1 align="center">Edgee</h1>
  </a>
</p>


**The full-stack edge platform for your edge oriented applications.**

[![Edgee](https://img.shields.io/badge/edgee-open%20source-blueviolet.svg)](https://www.edgee.cloud)
[![Edgee](https://img.shields.io/badge/slack-edgee-blueviolet.svg?logo=slack)](https://www.edgee.cloud/slack)
[![Docs](https://img.shields.io/badge/docs-published-blue)](https://docs.edgee.cloud)

</div>

This component implements the data collection protocol between [Amplitude](https://amplitude.com) and [Edgee](https://www.edgee.cloud).

#### Event mapping:

Here is the mapping between Edgee events and Amplitude events:

| Edgee event | Amplitude Event  |
|-------------|-----------|
| Page   | `[Amplitude] Page Viewed` _(and session_start/session_end if needed)_  |
| Track  | Name of the event |
| User   | `identify` |

Each time you make a `user` call, Edgee will send an `identify` event to Amplitude.

But when you make a `user` call using Edgee's JS library or Data Layer, the `user_id`, `anonymous_id` and `properties` are stored in the user's device.
This allows the user's data to be added to any subsequent page or follow-up calls for the user, so that you can correctly attribute these actions.

## Usage

- Download the latest version in our [releases page](../../releases). 
- Place the wasm file in a known place in your server (e.g. `/var/edgee/components`).
- Update your edgee proxy config:
```toml
[[destinations.data_collection]]
name = "amplitude"
component = "/var/edgee/components/amplitude.wasm"
credentials.amplitude_api_key = "..." 
```

## Contributing
If you're interested in contributing to Edgee, read our [contribution guidelines](./CONTRIBUTING.md)

## Reporting Security Vulnerabilities
If you've found a vulnerability or potential vulnerability in our code, please let us know at
[edgee-security](mailto:security@edgee.cloud).

## Building from source

To build the wasm file from source, you need to have installed 
- [Rust](https://www.rust-lang.org/tools/install)
- `wasm32-wasip2` target: run `rustup target add wasm32-wasip2`

Then you can run the following commands:

```bash
make build
```
