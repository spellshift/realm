macro_rules! crate_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

pub const VERSION: &str = crate_version!();
