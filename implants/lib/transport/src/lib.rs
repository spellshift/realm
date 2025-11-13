#[cfg(all(feature = "grpc", feature = "http"))]
compile_error!("only one transport may be selected");
#[cfg(all(feature = "grpc-doh", feature = "http"))]
compile_error!("grpc-doh is only supported by the grpc transport");

#[cfg(feature = "grpc")]
mod grpc;
#[cfg(feature = "grpc")]
pub use grpc::GRPC as ActiveTransport;

#[cfg(feature = "grpc-doh")]
mod dns_resolver;

#[cfg(feature = "http")]
mod http;
#[cfg(feature = "http")]
pub use http::HTTP as ActiveTransport;

#[cfg(feature = "mock")]
mod mock;
#[cfg(feature = "mock")]
pub use mock::MockTransport;

mod transport;
pub use transport::Transport;
