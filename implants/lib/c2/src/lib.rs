pub mod pb {
    include!("c2.rs");
}

mod grpc;
mod tavern;
pub use grpc::GRPCTavernClient;
pub use tavern::TavernClient;
