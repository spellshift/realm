extern crate alloc;

#[cfg(feature = "stdlib")]
use eldritch_core::Interpreter;
#[cfg(feature = "stdlib")]
use eldritch_stdlib::{
    file::std::StdFileLibrary, http::std::StdHttpLibrary, process::std::StdProcessLibrary,
    random::std::StdRandomLibrary, regex::std::StdRegexLibrary,
};

#[cfg(feature = "fake_bindings")]
use eldritch_core::Interpreter;
#[cfg(feature = "fake_bindings")]
use eldritch_stdlib::{
    file::fake::FileLibraryFake, http::fake::HttpLibraryFake, regex::fake::RegexLibraryFake,
};

// Note: This test mimics the registration logic in `bin/repl.rs`
// and asserts that registered libraries are actually callable.

#[test]
fn test_integration_features() {
    #[cfg(feature = "stdlib")]
    {
        // Register stdlib libraries
        register_lib(StdFileLibrary);
        register_lib(StdHttpLibrary);
        register_lib(StdProcessLibrary);
        register_lib(StdRegexLibrary);
        register_lib(StdRandomLibrary);
    }

    #[cfg(feature = "fake_bindings")]
    #[cfg(not(feature = "stdlib"))]
    // Only run if stdlib is NOT enabled, to test fake mode specifically
    {
        // Register fake libraries
        register_lib(FileLibraryFake::default());
        register_lib(HttpLibraryFake::default());
        register_lib(RegexLibraryFake::default());
    }

    #[cfg(feature = "stdlib")]
    {
        let mut interp = Interpreter::new();

        // Verify `random.int` (stdlib specific)
        let script = "random.int(0, 100)";
        match interp.interpret(script) {
            Ok(Value::Int(v)) => assert!(v >= 0 && v < 100),
            Err(e) => panic!("stdlib random.int failed: {}", e),
            _ => panic!("stdlib random.int returned unexpected type"),
        }

        // Verify `process.list` (stdlib specific)
        // This might fail in strict environments, but usually passes on dev machines.
        // We just check if it returns a list or throws a specific "not supported" error,
        // but we expect it to at least be *callable*.
        let script = "process.list()";
        let res = interp.interpret(script);
        assert!(res.is_ok() || res.unwrap_err().contains("not supported"));
    }

    #[cfg(feature = "fake_bindings")]
    #[cfg(not(feature = "stdlib"))]
    // Only run if stdlib is NOT enabled, to test fake mode specifically
    {
        let mut interp = Interpreter::new();

        // Verify `file.read` (fake binding has pre-populated /home/user/notes.txt)
        let script = "file.read('/home/user/notes.txt')";
        match interp.interpret(script) {
            Ok(Value::String(s)) => assert_eq!(s, "secret plans"),
            Err(e) => panic!("fake file.read failed: {}", e),
            _ => panic!("fake file.read returned unexpected type"),
        }

        // Verify `random` is NOT available (fake bindings set usually doesn't include random in repl.rs logic for some reason?
        // Wait, bin/repl.rs only registers File, Http, Regex, Crypto for fake_bindings.
        // So `random` should be unknown.
        let script = "random";
        assert!(interp.interpret(script).is_err());
    }
}
