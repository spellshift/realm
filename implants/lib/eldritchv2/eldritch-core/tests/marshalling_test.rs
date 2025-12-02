use eldritch_core::Value;
use eldritch_core::conversion::{FromValue, ToValue};
// Make sure to import the trait implementations from host_conversion
use eldritch_core::host_conversion::*;
use eldritch_core::eldritch_hostcontext::*; // Use re-exported crate
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;

// Helper to create arbitrary FileEntry
fn arb_file_entry() -> impl Strategy<Value = FileEntry> {
    (
        any::<String>(), // path
        any::<String>(), // name
        any::<u64>(),    // size
        any::<bool>(),   // is_dir
        any::<u64>(),    // created
        any::<u64>(),    // modified
        any::<u64>(),    // accessed
        any::<u32>(),    // mode
    )
        .prop_map(|(path, name, size, is_dir, created, modified, accessed, mode)| FileEntry {
            path,
            name,
            size,
            is_dir,
            created,
            modified,
            accessed,
            mode,
        })
}

// ExecRequest Strategy
fn arb_exec_request() -> impl Strategy<Value = ExecRequest> {
    (
        any::<String>(), // path
        any::<Vec<String>>(), // args
        any::<Vec<String>>(), // env
        any::<String>(), // cwd
        any::<bool>(), // background
    )
    .prop_map(|(path, args, env, cwd, background)| ExecRequest {
        path,
        args,
        env,
        cwd,
        background,
    })
}

proptest! {
    #[test]
    fn test_file_entry_conversion(entry in arb_file_entry()) {
        // Convert to Value
        let val = entry.clone().to_value();

        // Check structure
        if let Value::Dictionary(d) = &val {
            let dict = d.borrow();
            assert_eq!(dict.get("path").unwrap().to_string(), entry.path);
            assert_eq!(dict.get("name").unwrap().to_string(), entry.name);
            // ... strict type checks can be done here
        } else {
            panic!("Expected Dictionary");
        }
    }

    #[test]
    fn test_list_dir_request_conversion(path in any::<String>()) {
        let v = Value::String(path.clone());
        let req = ListDirRequest::from_value(&v).expect("Should parse string");
        assert_eq!(req.path, path);
    }

    #[test]
    fn test_process_info_conversion(
        pid in any::<u32>(),
        name in any::<String>(),
        exe in any::<String>()
    ) {
        let p = ProcessInfo {
            pid,
            ppid: 0,
            name: name.clone(),
            exe: exe.clone(),
            cmdline: "".to_string(),
            user: "".to_string(),
            start_time: 0,
            cpu_usage: 0,
            memory_usage: 0,
        };
        let val = p.to_value();
        if let Value::Dictionary(d) = val {
            let dict = d.borrow();
             assert_eq!(dict.get("pid").unwrap().to_string(), pid.to_string());
             assert_eq!(dict.get("name").unwrap().to_string(), name);
        } else {
             panic!("Expected Dict");
        }
    }

    #[test]
    fn test_exec_request_conversion(req in arb_exec_request()) {
        // Manually construct Value from req to simulate what user might pass
        let mut map = BTreeMap::new();
        map.insert("path".to_string(), Value::String(req.path.clone()));

        let args_val = Value::List(Rc::new(RefCell::new(req.args.iter().map(|s| Value::String(s.clone())).collect())));
        map.insert("args".to_string(), args_val);

        let env_val = Value::List(Rc::new(RefCell::new(req.env.iter().map(|s| Value::String(s.clone())).collect())));
        map.insert("env".to_string(), env_val);

        map.insert("cwd".to_string(), Value::String(req.cwd.clone()));
        map.insert("background".to_string(), Value::Bool(req.background));

        let v = Value::Dictionary(Rc::new(RefCell::new(map)));

        let parsed = ExecRequest::from_value(&v).expect("Should parse ExecRequest");
        assert_eq!(parsed.path, req.path);
        assert_eq!(parsed.args, req.args);
        assert_eq!(parsed.env, req.env);
        assert_eq!(parsed.cwd, req.cwd);
        assert_eq!(parsed.background, req.background);
    }

    #[test]
    fn test_file_write_request(path in any::<String>(), data in any::<Vec<u8>>(), append in any::<bool>()) {
        let mut map = BTreeMap::new();
        map.insert("path".to_string(), Value::String(path.clone()));
        map.insert("data".to_string(), Value::Bytes(data.clone()));
        map.insert("append".to_string(), Value::Bool(append));
        let v = Value::Dictionary(Rc::new(RefCell::new(map)));

        let req = FileWriteRequest::from_value(&v).expect("Should parse FileWriteRequest");
        assert_eq!(req.path, path);
        assert_eq!(req.data, data);
        assert_eq!(req.append, append);
    }

    #[test]
    fn test_env_set_request(key in any::<String>(), value in any::<String>()) {
        let mut map = BTreeMap::new();
        map.insert("key".to_string(), Value::String(key.clone()));
        map.insert("value".to_string(), Value::String(value.clone()));
        let v = Value::Dictionary(Rc::new(RefCell::new(map)));

        let req = EnvSetRequest::from_value(&v).expect("Should parse EnvSetRequest");
        assert_eq!(req.key, key);
        assert_eq!(req.value, value);
    }
}
