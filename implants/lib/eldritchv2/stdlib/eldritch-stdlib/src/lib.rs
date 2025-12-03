pub use eldritch_libagent as agent;
pub use eldritch_libassets as assets;
pub use eldritch_libcrypto as crypto;
pub use eldritch_libfile as file;
pub use eldritch_libhttp as http;
pub use eldritch_libpivot as pivot;
pub use eldritch_libprocess as process;
pub use eldritch_librandom as random;
pub use eldritch_libregex as regex;
pub use eldritch_libreport as report;
pub use eldritch_libsys as sys;
pub use eldritch_libtime as time;

#[cfg(feature = "stdlib")]
pub fn register_all() {
    eldritch_core::register_lib(crypto::std::StdCryptoLibrary);
    eldritch_core::register_lib(file::std::StdFileLibrary);
    eldritch_core::register_lib(http::std::StdHttpLibrary);
    eldritch_core::register_lib(process::std::StdProcessLibrary);
    eldritch_core::register_lib(random::std::StdRandomLibrary);
    eldritch_core::register_lib(regex::std::StdRegexLibrary);
}
