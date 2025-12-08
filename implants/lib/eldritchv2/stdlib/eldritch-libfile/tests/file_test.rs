use eldritch_core::{Interpreter, Value};
use eldritch_libfile::std::StdFileLibrary;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_file_ops() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_str().unwrap().replace("\\", "/");
    let file_path = format!("{}/test.txt", dir_path);

    let lib = StdFileLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    // write
    let code = format!("file.write('{}', 'hello')", file_path);
    interp.interpret(&code).unwrap();
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "hello");

    // read
    let code = format!("file.read('{}')", file_path);
    let res = interp.interpret(&code).unwrap();
    assert_eq!(res, Value::String("hello".to_string()));

    // exists
    let code = format!("file.exists('{}')", file_path);
    let res = interp.interpret(&code).unwrap();
    assert_eq!(res, Value::Bool(true));

    // append
    let code = format!("file.append('{}', ' world')", file_path);
    interp.interpret(&code).unwrap();
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "hello world");

    // remove
    let code = format!("file.remove('{}')", file_path);
    interp.interpret(&code).unwrap();
    assert!(!fs::metadata(&file_path).is_ok());

    // mkdir
    let subdir_path = format!("{}/subdir", dir_path);
    let code = format!("file.mkdir('{}')", subdir_path);
    interp.interpret(&code).unwrap();
    assert!(fs::metadata(&subdir_path).unwrap().is_dir());
}
