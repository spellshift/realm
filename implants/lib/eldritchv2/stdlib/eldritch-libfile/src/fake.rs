
use eldritch_core::Value;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;
use spin::Mutex;
use super::FileLibrary;


#[derive(Debug, Clone)]
enum FsEntry {
    File(Vec<u8>),
    Dir(BTreeMap<String, FsEntry>),
}

#[derive(Debug)]
#[eldritch_library_impl(FileLibrary)]
pub struct FileLibraryFake {
    root: Arc<Mutex<FsEntry>>,
}

impl Default for FileLibraryFake {
    fn default() -> Self {
        let mut root_map = BTreeMap::new();

        // /tmp
        root_map.insert("tmp".to_string(), FsEntry::Dir(BTreeMap::new()));

        // /home/user
        let mut user_map = BTreeMap::new();
        user_map.insert(
            "notes.txt".to_string(),
            FsEntry::File(b"secret plans".to_vec()),
        );
        user_map.insert("todo.txt".to_string(), FsEntry::File(b"buy milk".to_vec()));

        let mut home_map = BTreeMap::new();
        home_map.insert("user".to_string(), FsEntry::Dir(user_map));

        root_map.insert("home".to_string(), FsEntry::Dir(home_map));

        // /etc
        let mut etc_map = BTreeMap::new();
        etc_map.insert(
            "passwd".to_string(),
            FsEntry::File(b"root:x:0:0:root:/root:/bin/bash\n".to_vec()),
        );
        root_map.insert("etc".to_string(), FsEntry::Dir(etc_map));

        Self {
            root: Arc::new(Mutex::new(FsEntry::Dir(root_map))),
        }
    }
}

impl FileLibraryFake {
    // Helper to normalize path. Handles . and ..
    fn normalize_path(path: &str) -> Vec<String> {
        let parts = path.split('/').filter(|p| !p.is_empty() && *p != ".");
        let mut stack = Vec::new();
        for part in parts {
            if part == ".." {
                stack.pop();
            } else {
                stack.push(part.to_string());
            }
        }
        stack
    }

    fn traverse<'a>(current: &'a mut FsEntry, parts: &[String]) -> Option<&'a mut FsEntry> {
        if parts.is_empty() {
            return Some(current);
        }
        match current {
            FsEntry::Dir(map) => {
                if let Some(next) = map.get_mut(&parts[0]) {
                    Self::traverse(next, &parts[1..])
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl FileLibrary for FileLibraryFake {
    fn append(&self, path: String, content: String) -> Result<(), String> {
        let mut root = self.root.lock();
        let parts = Self::normalize_path(&path);

        if let Some(entry) = Self::traverse(&mut root, &parts) {
            if let FsEntry::File(data) = entry {
                data.extend_from_slice(content.as_bytes());
                return Ok(());
            }
            return Err("Not a file".to_string());
        }
        Err("Path not found".to_string())
    }

    fn compress(&self, _src: String, _dst: String) -> Result<(), String> {
        Ok(())
    }

    fn copy(&self, src: String, dst: String) -> Result<(), String> {
        let mut root = self.root.lock();
        let src_parts = Self::normalize_path(&src);
        let dst_parts = Self::normalize_path(&dst);

        // Clone content first
        let content = if let Some(entry) = Self::traverse(&mut root, &src_parts) {
            entry.clone()
        } else {
            return Err("Source not found".to_string());
        };

        if dst_parts.is_empty() {
            return Err("Invalid destination".to_string());
        }
        let (parent_parts, file_name) = dst_parts.split_at(dst_parts.len() - 1);
        let file_name = &file_name[0];

        if let Some(parent) = Self::traverse(&mut root, parent_parts) {
            if let FsEntry::Dir(map) = parent {
                map.insert(file_name.clone(), content);
                return Ok(());
            }
            return Err("Destination parent is not a directory".to_string());
        }
        Err("Destination path not found".to_string())
    }

    fn decompress(&self, _src: String, _dst: String) -> Result<(), String> {
        Ok(())
    }

    fn exists(&self, path: String) -> Result<bool, String> {
        let mut root = self.root.lock();
        let parts = Self::normalize_path(&path);
        Ok(Self::traverse(&mut root, &parts).is_some())
    }

    fn follow(&self, _path: String, _fn_val: Value) -> Result<(), String> {
        Ok(())
    }

    fn is_dir(&self, path: String) -> Result<bool, String> {
        let mut root = self.root.lock();
        let parts = Self::normalize_path(&path);
        if let Some(FsEntry::Dir(_)) = Self::traverse(&mut root, &parts) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn is_file(&self, path: String) -> Result<bool, String> {
        let mut root = self.root.lock();
        let parts = Self::normalize_path(&path);
        if let Some(FsEntry::File(_)) = Self::traverse(&mut root, &parts) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn list(&self, path: String) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut root = self.root.lock();
        let parts = Self::normalize_path(&path);

        if let Some(FsEntry::Dir(map)) = Self::traverse(&mut root, &parts) {
            let mut result = Vec::new();
            for (name, entry) in map.iter() {
                let mut info = BTreeMap::new();
                info.insert("file_name".to_string(), Value::String(name.clone()));
                info.insert(
                    "is_dir".to_string(),
                    Value::Bool(matches!(entry, FsEntry::Dir(_))),
                );
                info.insert(
                    "size".to_string(),
                    Value::Int(match entry {
                        FsEntry::File(d) => d.len() as i64,
                        FsEntry::Dir(_) => 4096,
                    }),
                );
                result.push(info);
            }
            Ok(result)
        } else {
            Err("Not a directory".to_string())
        }
    }

    fn mkdir(&self, path: String, _parent: Option<bool>) -> Result<(), String> {
        let mut root = self.root.lock();
        let parts = Self::normalize_path(&path);
        if parts.is_empty() {
            return Ok(());
        }

        let (parent_parts, dir_name) = parts.split_at(parts.len() - 1);
        let dir_name = &dir_name[0];

        if let Some(parent) = Self::traverse(&mut root, parent_parts) {
            if let FsEntry::Dir(map) = parent {
                map.insert(dir_name.clone(), FsEntry::Dir(BTreeMap::new()));
                return Ok(());
            }
            return Err("Parent is not a directory".to_string());
        }
        // TODO: handle parent creation if _parent is true
        Err("Parent path not found".to_string())
    }

    fn move_(&self, src: String, dst: String) -> Result<(), String> {
        self.copy(src.clone(), dst)?;
        self.remove(src)
    }

    fn parent_dir(&self, path: String) -> Result<String, String> {
        let parts = Self::normalize_path(&path);
        if parts.is_empty() {
            return Ok("/".to_string());
        }
        let parent = &parts[0..parts.len() - 1];
        Ok(format!("/{}", parent.join("/")))
    }

    fn read(&self, path: String) -> Result<String, String> {
        let mut root = self.root.lock();
        let parts = Self::normalize_path(&path);

        if let Some(FsEntry::File(data)) = Self::traverse(&mut root, &parts) {
            Ok(String::from_utf8_lossy(data).into_owned())
        } else {
            Err("File not found".to_string())
        }
    }

    fn read_binary(&self, path: String) -> Result<Vec<u8>, String> {
        let mut root = self.root.lock();
        let parts = Self::normalize_path(&path);

        if let Some(FsEntry::File(data)) = Self::traverse(&mut root, &parts) {
            Ok(data.clone())
        } else {
            Err("File not found".to_string())
        }
    }

    fn remove(&self, path: String) -> Result<(), String> {
        let mut root = self.root.lock();
        let parts = Self::normalize_path(&path);
        if parts.is_empty() {
            return Err("Cannot remove root".to_string());
        }

        let (parent_parts, name) = parts.split_at(parts.len() - 1);
        let name = &name[0];

        if let Some(FsEntry::Dir(map)) = Self::traverse(&mut root, parent_parts) {
            map.remove(name);
            return Ok(());
        }
        Err("Parent not found".to_string())
    }

    fn replace(&self, _path: String, _pattern: String, _value: String) -> Result<(), String> {
        Ok(())
    }

    fn replace_all(&self, _path: String, _pattern: String, _value: String) -> Result<(), String> {
        Ok(())
    }

    fn temp_file(&self, name: Option<String>) -> Result<String, String> {
        let name = name.unwrap_or_else(|| "random".to_string());
        Ok(format!("/tmp/{}", name))
    }

    fn template(
        &self,
        _template_path: String,
        _dst: String,
        _args: BTreeMap<String, Value>,
        _autoescape: bool,
    ) -> Result<(), String> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn timestomp(
        &self,
        path: String,
        mtime: Option<Value>,
        atime: Option<Value>,
        ctime: Option<Value>,
        ref_file: Option<String>,
    ) -> Result<(), String> {
        Ok(())
    }

    fn write(&self, path: String, content: String) -> Result<(), String> {
        let mut root = self.root.lock();
        let parts = Self::normalize_path(&path);
        if parts.is_empty() {
            return Err("Invalid path".to_string());
        }

        let (parent_parts, name) = parts.split_at(parts.len() - 1);
        let name = &name[0];

        if let Some(parent) = Self::traverse(&mut root, parent_parts) {
            if let FsEntry::Dir(map) = parent {
                map.insert(name.clone(), FsEntry::File(content.into_bytes()));
                return Ok(());
            }
            return Err("Parent is not a directory".to_string());
        }
        Err("Parent path not found".to_string())
    }

    fn find(
        &self,
        _path: String,
        _name: Option<String>,
        _file_type: Option<String>,
        _permissions: Option<i64>,
        _modified_time: Option<i64>,
        _create_time: Option<i64>,
    ) -> Result<Vec<String>, String> {
        // Simple BFS/DFS to find all files
        Ok(Vec::new())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_file_fake() {
        let file = FileLibraryFake::default();

        // Exists
        assert!(file.exists("/home/user/notes.txt".into()).unwrap());
        assert!(!file.exists("/home/user/missing.txt".into()).unwrap());

        // Read
        assert_eq!(
            file.read("/home/user/notes.txt".into()).unwrap(),
            "secret plans"
        );

        // Write
        file.write("/tmp/test.txt".into(), "hello".into()).unwrap();
        assert_eq!(file.read("/tmp/test.txt".into()).unwrap(), "hello");

        // List
        let items = file.list("/home/user".into()).unwrap();
        assert!(items
            .iter()
            .any(|x| x.get("file_name").unwrap().to_string() == "notes.txt"));

        // Mkdir
        file.mkdir("/home/user/docs".into(), None).unwrap();
        assert!(file.is_dir("/home/user/docs".into()).unwrap());

        // Copy
        file.copy(
            "/home/user/notes.txt".into(),
            "/tmp/notes_backup.txt".into(),
        )
        .unwrap();
        assert_eq!(
            file.read("/tmp/notes_backup.txt".into()).unwrap(),
            "secret plans"
        );

        // Remove
        file.remove("/tmp/notes_backup.txt".into()).unwrap();
        assert!(!file.exists("/tmp/notes_backup.txt".into()).unwrap());
    }
}
