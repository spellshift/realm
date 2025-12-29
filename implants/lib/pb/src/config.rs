/// Config holds values necessary to configure an Agent.
/// This is a protobuf message type used in the wire protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Config {
    #[prost(message, optional, tag = "1")]
    pub info: ::core::option::Option<crate::c2::Beacon>,
    #[prost(string, tag = "2")]
    pub callback_uri: ::prost::alloc::string::String, // Now includes query params
    // REMOVED: proxy_uri (tag 3) - now in callback_uri query params
    // REMOVED: retry_interval (tag 4) - now in callback_uri query params
    #[prost(bool, tag = "5")]
    pub run_once: bool,
}
