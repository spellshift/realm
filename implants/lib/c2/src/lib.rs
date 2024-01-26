pub mod pb {
    include!("c2.rs");
}

mod c2_manual;
pub use c2_manual::TavernClient;
