use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use crate::ast::Value;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::Mutex;

#[eldritch_library("file")]
pub trait FileLibrary {
    #[eldritch_method]
    fn append(&self, path: String, content: String) -> Result<(), String>;

    #[eldritch_method]
    fn compress(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn copy(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn decompress(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn exists(&self, path: String) -> Result<bool, String>;

    #[eldritch_method]
    fn follow(&self, path: String, fn_val: Value) -> Result<(), String>; // fn is reserved

    #[eldritch_method]
    fn is_dir(&self, path: String) -> Result<bool, String>;

    #[eldritch_method]
    fn is_file(&self, path: String) -> Result<bool, String>;

    /*
    #[eldritch_method]
    fn list(&self, path: String) -> Result<Vec<BTreeMap<String, Value>>, String>;
    */

    #[eldritch_method]
    fn mkdir(&self, path: String, parent: Option<bool>) -> Result<(), String>;

    #[eldritch_method]
    fn moveto(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn parent_dir(&self, path: String) -> Result<String, String>;

    #[eldritch_method]
    fn read(&self, path: String) -> Result<String, String>;

    #[eldritch_method]
    fn read_binary(&self, path: String) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    fn remove(&self, path: String) -> Result<(), String>;

    #[eldritch_method]
    fn replace(&self, path: String, pattern: String, value: String) -> Result<(), String>;

    #[eldritch_method]
    fn replace_all(&self, path: String, pattern: String, value: String) -> Result<(), String>;

    #[eldritch_method]
    fn temp_file(&self, name: Option<String>) -> Result<String, String>;

    /*
    #[eldritch_method]
    fn template(&self, template_path: String, dst: String, args: BTreeMap<String, Value>, autoescape: bool) -> Result<(), String>;
    */

    #[eldritch_method]
    fn timestomp(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn write(&self, path: String, content: String) -> Result<(), String>;

    /*
    #[eldritch_method]
    fn find(&self, path: String, name: Option<String>, file_type: Option<String>, permissions: Option<i64>, modified_time: Option<i64>, create_time: Option<i64>) -> Result<Vec<String>, String>;
    */
}

#[cfg(feature = "fake_bindings")]
#[derive(Debug)]
#[eldritch_library_impl(FileLibrary)]
pub struct FileLibraryFake {
    files: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
}

#[cfg(feature = "fake_bindings")]
impl Default for FileLibraryFake {
    fn default() -> Self {
        Self {
            files: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

#[cfg(feature = "fake_bindings")]
impl FileLibrary for FileLibraryFake {
    fn append(&self, path: String, content: String) -> Result<(), String> {
        let mut files = self.files.lock();
        let entry = files.entry(path).or_insert_with(Vec::new);
        entry.extend_from_slice(content.as_bytes());
        Ok(())
    }

    fn compress(&self, _src: String, _dst: String) -> Result<(), String> { Ok(()) }

    fn copy(&self, src: String, dst: String) -> Result<(), String> {
        let mut files = self.files.lock();
        if let Some(content) = files.get(&src).cloned() {
            files.insert(dst, content);
        }
        Ok(())
    }

    fn decompress(&self, _src: String, _dst: String) -> Result<(), String> { Ok(()) }

    fn exists(&self, path: String) -> Result<bool, String> {
        Ok(self.files.lock().contains_key(&path))
    }

    fn follow(&self, _path: String, _fn_val: Value) -> Result<(), String> { Ok(()) }

    fn is_dir(&self, _path: String) -> Result<bool, String> {
        // Simple mock: assume everything ending in / is a dir
        Ok(false)
    }

    fn is_file(&self, path: String) -> Result<bool, String> {
        Ok(self.files.lock().contains_key(&path))
    }

    /*
    fn list(&self, _path: String) -> Result<Vec<BTreeMap<String, Value>>, String> {
        // Return dummy file info
        let mut map = BTreeMap::new();
        map.insert("file_name".into(), Value::String("foo".into()));
        Ok(vec![map])
    }
    */

    fn mkdir(&self, _path: String, _parent: Option<bool>) -> Result<(), String> { Ok(()) }

    fn moveto(&self, src: String, dst: String) -> Result<(), String> {
        let mut files = self.files.lock();
        if let Some(content) = files.remove(&src) {
            files.insert(dst, content);
        }
        Ok(())
    }

    fn parent_dir(&self, _path: String) -> Result<String, String> {
        Ok(String::from(".."))
    }

    fn read(&self, path: String) -> Result<String, String> {
        Ok(self.files.lock().get(&path)
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .unwrap_or_default())
    }

    fn read_binary(&self, path: String) -> Result<Vec<u8>, String> {
        Ok(self.files.lock().get(&path).cloned().unwrap_or_default())
    }

    fn remove(&self, path: String) -> Result<(), String> {
        self.files.lock().remove(&path);
        Ok(())
    }

    fn replace(&self, _path: String, _pattern: String, _value: String) -> Result<(), String> { Ok(()) }

    fn replace_all(&self, _path: String, _pattern: String, _value: String) -> Result<(), String> { Ok(()) }

    fn temp_file(&self, name: Option<String>) -> Result<String, String> {
        Ok(name.unwrap_or_else(|| String::from("/tmp/random")))
    }

    /*
    fn template(&self, _template_path: String, _dst: String, _args: BTreeMap<String, Value>, _autoescape: bool) -> Result<(), String> { Ok(()) }
    */

    fn timestomp(&self, _src: String, _dst: String) -> Result<(), String> { Ok(()) }

    fn write(&self, path: String, content: String) -> Result<(), String> {
        self.files.lock().insert(path, content.into_bytes());
        Ok(())
    }

    /*
    fn find(&self, _path: String, _name: Option<String>, _file_type: Option<String>, _permissions: Option<i64>, _modified_time: Option<i64>, _create_time: Option<i64>) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    */
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_file_fake() {
        let file = FileLibraryFake::default();
        file.write("test.txt".into(), "hello".into()).unwrap();
        assert!(file.exists("test.txt".into()).unwrap());
        assert_eq!(file.read("test.txt".into()).unwrap(), "hello");

        file.append("test.txt".into(), " world".into()).unwrap();
        assert_eq!(file.read("test.txt".into()).unwrap(), "hello world");

        file.copy("test.txt".into(), "copy.txt".into()).unwrap();
        assert!(file.exists("copy.txt".into()).unwrap());

        file.remove("test.txt".into()).unwrap();
        assert!(!file.exists("test.txt".into()).unwrap());
    }
}
