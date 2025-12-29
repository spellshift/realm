pub mod constants;
pub mod system;
pub mod beacon;
pub mod builder;

// Re-export main types and traits
pub use builder::{Config, ConfigBuilder};
pub use constants::{CALLBACK_INTERVAL, CALLBACK_URI, RETRY_INTERVAL, RUN_ONCE};

// Re-export for convenience
pub mod prelude {
    pub use crate::builder::{Config, ConfigBuilder};
    pub use crate::constants::{CALLBACK_INTERVAL, CALLBACK_URI, RETRY_INTERVAL, RUN_ONCE};
    pub use crate::system::{get_host_platform, get_primary_ip, get_system_proxy};
}
