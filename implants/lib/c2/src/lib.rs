pub mod pb {
    include!("c2.rs");
}

mod c2_manual;
mod tavern;
pub use c2_manual::TavernClient;
