use eldritch_core::{Interpreter, Value};
use eldritch_libhttp::std::StdHttpLibrary;
use tempfile::NamedTempFile;
use std::fs;
use httptest::{Server, Expectation, matchers::*, responders::*};

#[test]
fn test_http_get() {
    let server = Server::run();
    server.expect(
        Expectation::matching(request::method_path("GET", "/foo"))
            .respond_with(status_code(200).body("bar")),
    );
    let url = server.url("/foo");
    let url_str = url.to_string();

    let lib = StdHttpLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    let code = format!("http.get('{}')", url_str);
    let res = interp.interpret(&code).unwrap();

    if let Value::Dictionary(d) = res {
        let dict = d.read();
        // The keys are "status_code" and "body".

        let status = dict.get(&Value::String("status_code".to_string()));
        let body = dict.get(&Value::String("body".to_string()));

        assert!(status.is_some(), "Status missing in {:?}", dict);
        assert!(body.is_some(), "Body missing in {:?}", dict);

        match status.unwrap() {
            Value::Int(s) => assert_eq!(*s, 200),
            v => panic!("Expected Int status, got {:?}", v),
        }

        match body.unwrap() {
            Value::String(s) => assert_eq!(s, "bar"),
            Value::Bytes(b) => assert_eq!(b, b"bar"),
            v => panic!("Expected String/Bytes body, got {:?}", v),
        }

    } else {
        panic!("Expected dict, got {:?}", res);
    }
}

#[test]
fn test_http_post() {
    let server = Server::run();
    server.expect(
        Expectation::matching(request::method_path("POST", "/post"))
            .respond_with(status_code(201).body("created")),
    );
    let url = server.url("/post");
    let url_str = url.to_string();

    let lib = StdHttpLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    let code = format!("http.post('{}', b'data')", url_str);
    let res = interp.interpret(&code).unwrap();

    if let Value::Dictionary(d) = res {
        let dict = d.read();

        let status = dict.get(&Value::String("status_code".to_string()));
        let body = dict.get(&Value::String("body".to_string()));

        assert!(status.is_some(), "Status missing in {:?}", dict);
        match status.unwrap() {
            Value::Int(s) => assert_eq!(*s, 201),
            v => panic!("Expected Int status, got {:?}", v),
        }

        match body.unwrap() {
            Value::String(s) => assert_eq!(s, "created"),
            Value::Bytes(b) => assert_eq!(b, b"created"),
            v => panic!("Expected String/Bytes body, got {:?}", v),
        }
    } else {
        panic!("Expected dict, got {:?}", res);
    }
}

#[test]
fn test_http_download() {
    let server = Server::run();
    server.expect(
        Expectation::matching(request::method_path("GET", "/dl"))
            .respond_with(status_code(200).body("content")),
    );
    let url = server.url("/dl");
    let url_str = url.to_string();

    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_str().unwrap().replace("\\", "/");

    let lib = StdHttpLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    let code = format!("http.download('{}', '{}')", url_str, path);
    interp.interpret(&code).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "content");
}
