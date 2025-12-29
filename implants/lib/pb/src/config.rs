/// ActiveCallback holds configuration for agent callback behavior.
/// This is a protobuf message type used in the wire protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActiveCallback {
    #[prost(uint64, tag = "1")]
    pub retry_interval: u64,
    #[prost(uint64, tag = "2")]
    pub callback_interval: u64,
    #[prost(string, tag = "3")]
    pub callback_uri: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub transport_config: ::prost::alloc::string::String,
}

/// Config holds values necessary to configure an Agent.
/// This is a protobuf message type used in the wire protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Config {
    #[prost(message, optional, tag = "1")]
    pub info: ::core::option::Option<crate::c2::Beacon>,
    #[prost(message, optional, tag = "2")]
    pub active_callback: ::core::option::Option<ActiveCallback>,
    #[prost(bool, tag = "5")]
    pub run_once: bool,
}
