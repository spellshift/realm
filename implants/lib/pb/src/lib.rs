pub mod eldritch {
    include!("generated/eldritch.rs");
}
pub mod c2 {
    include!("generated/c2.rs");
}
pub mod dns {
    include!("generated/dns.rs");
}
pub mod portal {
    include!("generated/portal.rs");
}
pub mod trace {
    include!("generated/trace.rs");
}
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod xchacha;
