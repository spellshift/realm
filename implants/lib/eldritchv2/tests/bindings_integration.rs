#[cfg(all(test, feature = "fake_bindings"))]
mod integration {
    extern crate alloc;
    use eldritchv2::{Interpreter, register_lib};
    use eldritchv2::bindings::{
        agent::AgentLibraryFake,
        assets::AssetsLibraryFake,
        crypto::CryptoLibraryFake,
        file::FileLibraryFake,
        http::HttpLibraryFake,
        pivot::PivotLibraryFake,
        process::ProcessLibraryFake,
        random::RandomLibraryFake,
        regex::RegexLibraryFake,
        report::ReportLibraryFake,
        sys::SysLibraryFake,
        time::TimeLibraryFake,
    };
    use alloc::string::ToString;

    fn register_all_fakes() {
        register_lib(AgentLibraryFake::default());
        register_lib(AssetsLibraryFake::default());
        register_lib(CryptoLibraryFake::default());
        register_lib(FileLibraryFake::default());
        register_lib(HttpLibraryFake::default());
        register_lib(PivotLibraryFake::default());
        register_lib(ProcessLibraryFake::default());
        register_lib(RandomLibraryFake::default());
        register_lib(RegexLibraryFake::default());
        register_lib(ReportLibraryFake::default());
        register_lib(SysLibraryFake::default());
        register_lib(TimeLibraryFake::default());
    }

    #[test]
    fn test_bindings_in_interpreter() {
        register_all_fakes();
        let mut interp = Interpreter::new();

        // Test File
        let script = "
file.write('/tmp/test', 'hello')
content = file.read('/tmp/test')
";
        let res = interp.interpret(script);
        assert!(res.is_ok(), "Script failed: {:?}", res.err());

        // Check side effect
        let read_script = "file.read('/tmp/test')";
        let res = interp.interpret(read_script).unwrap();
        match res {
            eldritchv2::Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string, got {:?}", res),
        }
    }

    #[test]
    fn test_process_list() {
        register_all_fakes();
        let mut interp = Interpreter::new();
        // Since list() returns empty vec in reduced implementation, let's just call it
        let script = "process.list()";
        let res = interp.interpret(script);
        assert!(res.is_ok());
    }
}
