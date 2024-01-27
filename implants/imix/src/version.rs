macro_rules! crate_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

pub const VERSION: &'static str = crate_version!();
