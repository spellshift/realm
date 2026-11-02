use super::UserLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(UserLibrary)]
pub struct UserLibraryFake;

impl UserLibrary for UserLibraryFake {
    fn info(&self) -> Result<BTreeMap<String, Value>, String> {
        let mut map = BTreeMap::new();
        map.insert("username".into(), Value::String("root".into()));
        map.insert("uid".into(), Value::Int(0));
        map.insert("gid".into(), Value::Int(0));
        Ok(map)
    }

    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut users = Vec::new();
        let mut root = BTreeMap::new();
        root.insert("principal".into(), Value::String("root".into()));
        root.insert("uid".into(), Value::Int(0));
        root.insert("gid".into(), Value::Int(0));
        users.push(root);
        Ok(users)
    }
}
