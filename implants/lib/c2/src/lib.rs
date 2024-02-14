pub mod pb {
    include!("generated/c2.rs");
}

mod grpc;
mod transport;
pub use grpc::GRPC;
pub use transport::Transport;
