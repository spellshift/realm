# imix-config

Configuration parsing and validation library for the Imix implant build process.

## Overview

This crate provides YAML parsing, validation, and transformation logic for Imix build configurations. It was extracted from the main `build.rs` to enable proper unit testing and avoid circular dependencies.

## Features

- **YAML Parsing**: Parse build configuration from YAML strings
- **Validation**: Comprehensive validation of configuration values
  - At least one callback must be configured
  - Proxy URI only supported for HTTP/HTTPS transports
  - DoH provider only supported for gRPC callbacks
  - Valid DoH provider values (cloudflare, google, quad9)
- **Transformation**: Prepare callbacks for runtime by encoding DoH provider in URI
- **Utility Functions**: Find first HTTPS callback for backward compatibility

## Usage

```rust
use imix_config::{parse_yaml_build_config, validate_config, prepare_callbacks};

let yaml = r#"
callbacks:
  - uri: "https://example.com:8000"
    interval: 10
    doh_provider: "cloudflare"
"#;

// Parse YAML
let config = parse_yaml_build_config(yaml)?;

// Validate configuration
validate_config(&config)?;

// Prepare callbacks for runtime
if let Some(callbacks) = &config.callbacks {
    let runtime_callbacks = prepare_callbacks(callbacks);
}
```

## Testing

Run the test suite:

```bash
cargo test
```

The test suite includes:
- Parsing various YAML configurations
- Validation of different error conditions
- Callback preparation with and without DoH
- Finding HTTPS callbacks
- Feature flag parsing

## Integration

This crate is used as a build dependency by the Imix implant:

```toml
[build-dependencies]
imix-config = { path = "../../bin/imix-config" }
```

## API

### Functions

- `parse_yaml_build_config(yaml_content: &str) -> Result<ImixBuildConfig, Box<dyn Error>>`
  - Parse YAML configuration from string

- `validate_config(config: &ImixBuildConfig) -> Result<(), ValidationError>`
  - Validate the parsed configuration

- `prepare_callbacks(callbacks: &[CallbackConfig]) -> Vec<CallbackConfig>`
  - Prepare callbacks for runtime by encoding DoH provider

- `find_first_https_callback(callbacks: &[CallbackConfig]) -> Option<&CallbackConfig>`
  - Find the first HTTPS callback in the list

### Types

- `ImixBuildConfig`: Main configuration structure
- `CallbackConfig`: Individual callback configuration
- `ValidationError`: Validation error types
