use super::super::agent::ImixAgent;
use super::super::task::TaskRegistry;
use eldritch::agent::agent::Agent;
use pb::config::Config;
use std::sync::Arc;
use transport::MockTransport;

#[allow(clippy::field_reassign_with_default)]
#[tokio::test]
async fn test_imix_agent_get_callback_interval_error() {
    let mut config = Config::default();
    config.info = None; // Ensure no beacon info to trigger error

    let transport = MockTransport::default();
    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(config, transport, handle, registry);

    let result = agent.get_callback_interval_u64();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No beacon info"));
}

#[allow(clippy::field_reassign_with_default)]
#[tokio::test]
async fn test_imix_agent_get_callback_interval_success() {
    let mut config = Config::default();
    config.info = Some(pb::c2::Beacon {
        available_transports: Some(pb::c2::AvailableTransports {
            transports: vec![pb::c2::Transport {
                uri: "http://example.com/callback".to_string(),
                interval: 10,
                ..Default::default()
            }],
            active_index: 0,
        }),
        ..Default::default()
    });

    let transport = MockTransport::default();
    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(config, transport, handle, registry);

    let result = agent.get_callback_interval_u64();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 10);
}

#[allow(clippy::field_reassign_with_default)]
#[tokio::test]
async fn test_set_callback_uri_new_transport() {
    // Create config with multiple initial transports
    let mut config = Config::default();
    config.info = Some(pb::c2::Beacon {
        available_transports: Some(pb::c2::AvailableTransports {
            transports: vec![
                pb::c2::Transport {
                    uri: "http://primary.example.com".to_string(),
                    interval: 5,
                    ..Default::default()
                },
                pb::c2::Transport {
                    uri: "https://secondary.example.com".to_string(),
                    interval: 5,
                    ..Default::default()
                },
            ],
            active_index: 0,
        }),
        ..Default::default()
    });

    let transport = MockTransport::default();
    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(config, transport, handle, registry);

    // Run in thread for block_on
    let agent_clone = agent.clone();
    let result = std::thread::spawn(move || {
        // Verify initial state - should have 2 URIs, active is the first one
        let initial_uris = agent_clone
            .list_callback_uris()
            .expect("Failed to list URIs");
        assert_eq!(initial_uris.len(), 2);

        let initial_active = agent_clone
            .get_active_callback_uri()
            .expect("Failed to get active URI");
        assert_eq!(initial_active, "http://primary.example.com");

        // Add a new URI that doesn't exist
        let new_uri = "https://new.example.com".to_string();
        agent_clone
            .set_callback_uri(new_uri.clone())
            .expect("Failed to set new callback URI");

        // Verify the new URI was added and is now active
        let updated_uris = agent_clone
            .list_callback_uris()
            .expect("Failed to list URIs after update");
        assert_eq!(
            updated_uris.len(),
            3,
            "Should have 3 URIs after adding new one"
        );
        assert!(
            updated_uris.contains(&new_uri),
            "New URI should be in the list"
        );

        let active_uri = agent_clone
            .get_active_callback_uri()
            .expect("Failed to get active URI after update");
        assert_eq!(active_uri, new_uri, "New URI should be the active one");
    })
    .join();

    assert!(result.is_ok(), "Test thread panicked");
}

#[allow(clippy::field_reassign_with_default)]
#[tokio::test]
async fn test_set_callback_uri_existing_transport() {
    // Create config with multiple transports
    let mut config = Config::default();
    config.info = Some(pb::c2::Beacon {
        available_transports: Some(pb::c2::AvailableTransports {
            transports: vec![
                pb::c2::Transport {
                    uri: "http://primary.example.com".to_string(),
                    interval: 5,
                    ..Default::default()
                },
                pb::c2::Transport {
                    uri: "https://secondary.example.com".to_string(),
                    interval: 5,
                    ..Default::default()
                },
                pb::c2::Transport {
                    uri: "dns://8.8.8.8".to_string(),
                    interval: 5,
                    ..Default::default()
                },
            ],
            active_index: 0,
        }),
        ..Default::default()
    });

    let transport = MockTransport::default();
    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(config, transport, handle, registry);

    // Run in thread for block_on
    let agent_clone = agent.clone();
    let result = std::thread::spawn(move || {
        // Verify initial active URI
        let initial_active = agent_clone
            .get_active_callback_uri()
            .expect("Failed to get initial active URI");
        assert_eq!(initial_active, "http://primary.example.com");

        // Switch to an existing URI (the DNS one)
        let existing_uri = "dns://8.8.8.8".to_string();
        agent_clone
            .set_callback_uri(existing_uri.clone())
            .expect("Failed to set existing callback URI");

        // Verify the URI list didn't grow (no duplicate added)
        let uris = agent_clone
            .list_callback_uris()
            .expect("Failed to list URIs");
        assert_eq!(uris.len(), 3, "Should still have 3 URIs, no duplicates");

        // Verify the active URI changed to the existing one
        let active_uri = agent_clone
            .get_active_callback_uri()
            .expect("Failed to get active URI after switch");
        assert_eq!(
            active_uri, existing_uri,
            "Should have switched to the existing DNS URI"
        );
    })
    .join();

    assert!(result.is_ok(), "Test thread panicked");
}
