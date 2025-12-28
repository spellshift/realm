use assert_cmd::Command;
use predicates::prelude::*;

const VALID_CONFIG: &str = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5

callbacks:
  - type: grpc
    uri: "http://localhost:8080"
    doh: cloudflare
    proxy_uri: "http://localhost:3128"
  - type: grpc
    uri: "https://localhost:8443"
    doh: google
  - type: http1
    uri: "https1://localhost:8443"
  - type: http1
    uri: "http1://localhost:8080"
  - type: dns
    uri: "dns://localhost:53"
    domain: "example.com"
    query_type: "TXT"

run_once: true

features:
  - grpc
  - http1
  - dns
"#;

#[test]
fn test_valid_config() {
    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", VALID_CONFIG);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "cargo:rustc-env=IMIX_SERVER_PUBKEY=4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ=",
        ))
        .stdout(predicate::str::contains(
            "cargo:rustc-env=IMIX_CALLBACK_INTERVAL=5",
        ))
        .stdout(predicate::str::contains(
            "cargo:rustc-env=IMIX_RETRY_INTERVAL=5",
        ))
        .stdout(predicate::str::contains(
            "cargo:rustc-env=IMIX_RUN_ONCE=true",
        ))
        .stdout(predicate::str::contains(
            "cargo:rustc-env=IMIX_CALLBACK_URI=",
        ))
        .stdout(predicate::str::contains("cargo:rustc-cfg=feature=\"grpc\""))
        .stdout(predicate::str::contains(
            "cargo:rustc-cfg=feature=\"http1\"",
        ))
        .stdout(predicate::str::contains("cargo:rustc-cfg=feature=\"dns\""));
}

#[test]
fn test_callback_uri_format() {
    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", VALID_CONFIG);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that callback URIs are semicolon-delimited
    assert!(
        stdout.contains(";"),
        "Callback URIs should be semicolon-delimited"
    );

    // Check that DoH parameter is included
    assert!(
        stdout.contains("doh=cloudflare") || stdout.contains("doh%3Dcloudflare"),
        "DoH parameter should be in callback URI"
    );

    // Check that proxy_uri parameter is included (may be URL-encoded)
    assert!(
        stdout.contains("proxy_uri="),
        "Proxy URI parameter should be in callback URI"
    );
}

#[test]
fn test_no_config_env_var() {
    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    // Don't set IMIX_CONFIG - should exit silently
    cmd.assert().success();
}

#[test]
fn test_invalid_yaml() {
    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", "invalid: yaml: syntax");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse YAML"));
}

#[test]
fn test_invalid_server_pubkey_length() {
    let config = r#"
server_pubkey: "tooshort"
interval: 5
retry_interval: 5
callbacks:
  - type: grpc
    uri: "http://localhost:8080"
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("server_pubkey"));
}

#[test]
fn test_invalid_server_pubkey_not_base64() {
    let config = r#"
server_pubkey: "not-valid-base64-!@#$%^&*()_+{}|:<>?~`"
interval: 5
retry_interval: 5
callbacks:
  - type: grpc
    uri: "http://localhost:8080"
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("base64"));
}

#[test]
fn test_no_callbacks() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks: []
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("At least one callback"));
}

#[test]
fn test_invalid_callback_type() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: invalid_type
    uri: "http://localhost:8080"
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid callback type"));
}

#[test]
fn test_invalid_grpc_uri_scheme() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: grpc
    uri: "ftp://localhost:8080"
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert().failure().stderr(predicate::str::contains(
        "gRPC callback URI must use http or https scheme",
    ));
}

#[test]
fn test_invalid_http1_uri_scheme() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: http1
    uri: "http://localhost:8080"
features:
  - http1
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert().failure().stderr(predicate::str::contains(
        "HTTP1 callback URI must use http1 or https1 scheme",
    ));
}

#[test]
fn test_invalid_dns_uri_scheme() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: dns
    uri: "http://localhost:53"
    domain: "example.com"
    query_type: "TXT"
features:
  - dns
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert().failure().stderr(predicate::str::contains(
        "DNS callback URI must use dns scheme",
    ));
}

#[test]
fn test_dns_missing_domain() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: dns
    uri: "dns://localhost:53"
    query_type: "TXT"
features:
  - dns
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert().failure().stderr(predicate::str::contains(
        "DNS callback requires 'domain' field",
    ));
}

#[test]
fn test_dns_missing_query_type() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: dns
    uri: "dns://localhost:53"
    domain: "example.com"
features:
  - dns
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert().failure().stderr(predicate::str::contains(
        "DNS callback requires 'query_type' field",
    ));
}

#[test]
fn test_invalid_doh_provider() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: grpc
    uri: "http://localhost:8080"
    doh: invalid_provider
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid DoH provider"));
}

#[test]
fn test_doh_on_non_grpc() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: http1
    uri: "http1://localhost:8080"
    doh: cloudflare
features:
  - http1
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert().failure().stderr(predicate::str::contains(
        "DoH provider can only be set for gRPC callbacks",
    ));
}

#[test]
fn test_invalid_proxy_uri() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: grpc
    uri: "http://localhost:8080"
    proxy_uri: "not a valid uri"
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid proxy URI"));
}

#[test]
fn test_zero_interval() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 0
retry_interval: 5
callbacks:
  - type: grpc
    uri: "http://localhost:8080"
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert().failure().stderr(predicate::str::contains(
        "interval must be a positive integer",
    ));
}

#[test]
fn test_zero_retry_interval() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 0
callbacks:
  - type: grpc
    uri: "http://localhost:8080"
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert().failure().stderr(predicate::str::contains(
        "retry_interval must be a positive integer",
    ));
}

#[test]
fn test_invalid_feature() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: grpc
    uri: "http://localhost:8080"
features:
  - grpc
  - invalid_feature
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid feature"));
}

#[test]
fn test_legacy_env_vars_conflict() {
    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", VALID_CONFIG);
    cmd.env("IMIX_CALLBACK_URI", "http://localhost:8080");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Legacy environment variables"));
}

#[test]
fn test_run_once_false() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: grpc
    uri: "http://localhost:8080"
run_once: false
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // run_once should not be set when false
    assert!(
        !stdout.contains("IMIX_RUN_ONCE"),
        "IMIX_RUN_ONCE should not be set when run_once is false"
    );
}

#[test]
fn test_run_once_omitted() {
    let config = r#"
server_pubkey: "4RKWp9WVrVrEcaTQK7MuZHdFOlFg2pP33G4c7qFZGTQ="
interval: 5
retry_interval: 5
callbacks:
  - type: grpc
    uri: "http://localhost:8080"
features:
  - grpc
"#;

    let mut cmd = Command::cargo_bin("imix-config-builder").unwrap();
    cmd.env("IMIX_CONFIG", config);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // run_once should not be set when omitted
    assert!(
        !stdout.contains("IMIX_RUN_ONCE"),
        "IMIX_RUN_ONCE should not be set when run_once is omitted"
    );
}
