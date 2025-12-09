use super::FileLibrary;
use ::std::fs::{self, File, OpenOptions};
use ::std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use ::std::path::Path;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::{Context, Result as AnyhowResult};
use eldritch_core::{Interpreter, Value};
use eldritch_macros::eldritch_library_impl;

#[cfg(unix)]
use nix::unistd::{Gid, Group, Uid, User};

#[cfg(feature = "stdlib")]
use flate2::Compression;
#[cfg(feature = "stdlib")]
use glob::glob;
#[cfg(feature = "stdlib")]
use regex::bytes::{NoExpand, Regex};
#[cfg(feature = "stdlib")]
use tar::{Archive, Builder, HeaderMode};
#[cfg(feature = "stdlib")]
use tempfile::NamedTempFile;
#[cfg(feature = "stdlib")]
use tera::{Context as TeraContext, Tera};

#[cfg(feature = "stdlib")]
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Debug, Default)]
#[eldritch_library_impl(FileLibrary)]
pub struct StdFileLibrary;

impl FileLibrary for StdFileLibrary {
    fn append(&self, path: String, content: String) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Failed to open file {path}: {e}"))?;

        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write to file {path}: {e}"))?;

        Ok(())
    }

    fn compress(&self, src: String, dst: String) -> Result<(), String> {
        compress_impl(src, dst).map_err(|e| e.to_string())
    }

    fn copy(&self, src: String, dst: String) -> Result<(), String> {
        fs::copy(&src, &dst).map_err(|e| format!("Failed to copy {src} to {dst}: {e}"))?;
        Ok(())
    }

    fn decompress(&self, src: String, dst: String) -> Result<(), String> {
        decompress_impl(src, dst).map_err(|e| e.to_string())
    }

    fn exists(&self, path: String) -> Result<bool, String> {
        Ok(Path::new(&path).exists())
    }

    fn follow(&self, path: String, fn_val: Value) -> Result<(), String> {
        follow_impl(path, fn_val).map_err(|e| e.to_string())
    }

    fn is_dir(&self, path: String) -> Result<bool, String> {
        Ok(Path::new(&path).is_dir())
    }

    fn is_file(&self, path: String) -> Result<bool, String> {
        Ok(Path::new(&path).is_file())
    }

    fn list(&self, path: String) -> Result<Vec<BTreeMap<String, Value>>, String> {
        list_impl(path).map_err(|e| e.to_string())
    }

    fn mkdir(&self, path: String, parent: Option<bool>) -> Result<(), String> {
        if parent.unwrap_or(false) {
            fs::create_dir_all(&path)
        } else {
            fs::create_dir(&path)
        }
        .map_err(|e| format!("Failed to create directory {path}: {e}"))
    }

    fn move_(&self, src: String, dst: String) -> Result<(), String> {
        fs::rename(&src, &dst).map_err(|e| format!("Failed to move {src} to {dst}: {e}"))
    }

    fn parent_dir(&self, path: String) -> Result<String, String> {
        let path = Path::new(&path);
        let parent = path
            .parent()
            .ok_or_else(|| "Failed to get parent directory".to_string())?;

        parent
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Failed to convert path to string".to_string())
    }

    fn read(&self, path: String) -> Result<String, String> {
        fs::read_to_string(&path).map_err(|e| format!("Failed to read file {path}: {e}"))
    }

    fn read_binary(&self, path: String) -> Result<Vec<u8>, String> {
        fs::read(&path).map_err(|e| format!("Failed to read file {path}: {e}"))
    }

    fn remove(&self, path: String) -> Result<(), String> {
        let p = Path::new(&path);
        if p.is_dir() {
            fs::remove_dir_all(p)
        } else {
            fs::remove_file(p)
        }
        .map_err(|e| format!("Failed to remove {path}: {e}"))
    }

    fn replace(&self, path: String, pattern: String, value: String) -> Result<(), String> {
        replace_impl(path, pattern, value, false).map_err(|e| e.to_string())
    }

    fn replace_all(&self, path: String, pattern: String, value: String) -> Result<(), String> {
        replace_impl(path, pattern, value, true).map_err(|e| e.to_string())
    }

    fn temp_file(&self, name: Option<String>) -> Result<String, String> {
        let temp_dir = ::std::env::temp_dir();
        let file_name = name.unwrap_or_else(|| {
            // Simple random name generation if None
            format!(
                "eldritch_{}",
                ::std::time::SystemTime::now()
                    .duration_since(::std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )
        });
        let path = temp_dir.join(file_name);
        path.to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Failed to convert temp path to string".to_string())
    }

    fn template(
        &self,
        template_path: String,
        dst: String,
        args: BTreeMap<String, Value>,
        autoescape: bool,
    ) -> Result<(), String> {
        template_impl(template_path, dst, args, autoescape).map_err(|e| e.to_string())
    }

    fn timestomp(&self, _src: String, _dst: String) -> Result<(), String> {
        Err("timestomp is unimplemented".to_string())
    }

    fn write(&self, path: String, content: String) -> Result<(), String> {
        fs::write(&path, content).map_err(|e| format!("Failed to write to file {path}: {e}"))
    }

    fn find(
        &self,
        path: String,
        name: Option<String>,
        file_type: Option<String>,
        permissions: Option<i64>,
        modified_time: Option<i64>,
        create_time: Option<i64>,
    ) -> Result<Vec<String>, String> {
        find_impl(
            path,
            name,
            file_type,
            permissions,
            modified_time,
            create_time,
        )
        .map_err(|e| e.to_string())
    }
}

// Implementations

#[cfg(feature = "stdlib")]
fn follow_impl(path: String, fn_val: Value) -> AnyhowResult<()> {
    // get pos to end of file
    let mut file = File::open(&path)?;
    let mut pos = fs::metadata(&path)?.len();

    // set up watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(Path::new(&path), RecursiveMode::NonRecursive)?;

    // We need an interpreter to run the callback.
    // If it's a user function, it captures its environment (closure).
    // If it's native (like print), it needs an environment with a printer.
    // We try to re-use the printer from the closure if available, else default.

    let mut printer = None;
    if let Value::Function(f) = &fn_val {
        printer = Some(f.closure.read().printer.clone());
    }

    // Since this is blocking, we can create one interpreter instance and reuse it
    let mut interp = if let Some(p) = printer {
        Interpreter::new_with_printer(p)
    } else {
        Interpreter::new()
    };

    // watch
    for _event in rx.into_iter().flatten() {
        // ignore any event that didn't change the pos
        if let Ok(meta) = file.metadata() {
            if meta.len() == pos {
                continue;
            }
        } else {
            continue;
        }

        // read from pos to end of file
        file.seek(SeekFrom::Start(pos))?;

        let mut reader = BufReader::new(&file);
        let mut bytes_read = 0;

        loop {
            let mut line = String::new();
            // read_line includes the delimiter
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                break;
            }
            bytes_read += n as u64;

            // Trim trailing newline for consistency with lines() which strips it?
            // V1 used `reader.lines()` which strips newline.
            // read_line keeps it. We should strip it.
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }

            let line_val = Value::String(line);

            // Execute callback
            // We use define_variable + interpret as a robust way to call without internal API access
            interp.define_variable("_follow_cb", fn_val.clone());
            interp.define_variable("_follow_line", line_val);

            match interp.interpret("_follow_cb(_follow_line)") {
                Ok(_) => {}
                Err(e) => return Err(anyhow::anyhow!(e)),
            }
        }

        // update pos based on actual bytes read
        pos += bytes_read;
    }
    Ok(())
}

#[cfg(not(feature = "stdlib"))]
fn follow_impl(_path: String, _fn_val: Value) -> AnyhowResult<()> {
    Err(anyhow::anyhow!("follow not supported in no_std or without stdlib feature"))
}

fn compress_impl(src: String, dst: String) -> AnyhowResult<()> {
    let src_path = Path::new(&src);

    // Determine if we need to tar
    let tmp_tar_file_src = NamedTempFile::new()?;
    let tmp_src = if src_path.is_dir() {
        let tmp_path = tmp_tar_file_src.path().to_str().unwrap().to_string();
        tar_dir(&src, &tmp_path)?;
        tmp_path
    } else {
        src.clone()
    };

    let f_src = ::std::io::BufReader::new(File::open(&tmp_src)?);
    let f_dst = ::std::io::BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&dst)?,
    );

    let mut deflater = flate2::write::GzEncoder::new(f_dst, Compression::fast());
    let mut reader = f_src;
    ::std::io::copy(&mut reader, &mut deflater)?;
    deflater.finish()?;

    Ok(())
}

fn tar_dir(src: &str, dst: &str) -> AnyhowResult<()> {
    let src_path = Path::new(src);
    let file = File::create(dst)?;
    let mut tar_builder = Builder::new(file);
    tar_builder.mode(HeaderMode::Deterministic);

    let src_name = src_path.file_name().context("Failed to get file name")?;

    tar_builder.append_dir_all(src_name, src_path)?;
    tar_builder.finish()?;
    Ok(())
}

fn decompress_impl(src: String, dst: String) -> AnyhowResult<()> {
    let f_src = ::std::io::BufReader::new(File::open(&src)?);
    let mut decoder = flate2::read::GzDecoder::new(f_src);

    let mut decoded_data = Vec::new();
    decoder.read_to_end(&mut decoded_data)?;

    // Try as tar
    // Create a temp dir to verify if it is a tar
    if Archive::new(decoded_data.as_slice()).entries().is_ok() {
        // It's likely a tar

        let dst_path = Path::new(&dst);
        if !dst_path.exists() {
            fs::create_dir_all(dst_path)?;
        }

        let mut archive = Archive::new(decoded_data.as_slice());

        let tmp_dir = tempfile::tempdir()?;
        match archive.unpack(tmp_dir.path()) {
            Ok(_) => {
                if dst_path.exists() {
                    fs::remove_dir_all(dst_path).ok(); // ignore fail
                }

                // Keep the temp dir content by moving it
                let path = tmp_dir.keep();
                fs::rename(&path, &dst)?;
                Ok(())
            }
            Err(_) => {
                // Not a tar or unpack failed. Write raw bytes.
                if dst_path.exists() && dst_path.is_dir() {
                    fs::remove_dir_all(dst_path)?;
                }
                fs::write(&dst, decoded_data)?;
                Ok(())
            }
        }
    } else {
        // Not a tar
        fs::write(&dst, decoded_data)?;
        Ok(())
    }
}

fn list_impl(path: String) -> AnyhowResult<Vec<BTreeMap<String, Value>>> {
    let mut final_res = Vec::new();

    // Glob
    for entry in glob(&path)? {
        match entry {
            Ok(path_buf) => {
                // If I implement `handle_list` roughly:
                if path_buf.is_dir() {
                    for entry in fs::read_dir(&path_buf)? {
                        let entry = entry?;
                        final_res.push(create_dict_from_file(&entry.path())?);
                    }
                } else {
                    final_res.push(create_dict_from_file(&path_buf)?);
                }
            }
            Err(e) => eprintln!("Glob error: {e:?}"),
        }
    }
    Ok(final_res)
}

fn create_dict_from_file(path: &Path) -> AnyhowResult<BTreeMap<String, Value>> {
    let metadata = fs::metadata(path)?;
    let mut dict = BTreeMap::new();

    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    dict.insert("file_name".to_string(), Value::String(name));

    let is_dir = metadata.is_dir();
    // Map to "file", "dir", "link", etc if possible.
    // V1 uses FileType enum.
    let type_str = if is_dir { "dir" } else { "file" }; // simplified
    dict.insert("type".to_string(), Value::String(type_str.to_string()));

    dict.insert("size".to_string(), Value::Int(metadata.len() as i64));

    // Permissions (simplified)
    #[cfg(unix)]
    use ::std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    let perms = format!("{:o}", metadata.permissions().mode());
    #[cfg(not(unix))]
    let perms = if metadata.permissions().readonly() {
        "r"
    } else {
        "rw"
    }
    .to_string();

    dict.insert("permissions".to_string(), Value::String(perms));

    // Owner and Group
    #[cfg(unix)]
    {
        use ::std::os::unix::fs::MetadataExt;
        let uid = metadata.uid();
        let gid = metadata.gid();

        let user = User::from_uid(Uid::from_raw(uid)).ok().flatten();
        let group = Group::from_gid(Gid::from_raw(gid)).ok().flatten();

        let owner_name = user.map(|u| u.name).unwrap_or_else(|| uid.to_string());
        let group_name = group.map(|g| g.name).unwrap_or_else(|| gid.to_string());

        dict.insert("owner".to_string(), Value::String(owner_name));
        dict.insert("group".to_string(), Value::String(group_name));
    }
    #[cfg(not(unix))]
    {
        // Fallback for Windows or others
        dict.insert("owner".to_string(), Value::String("".to_string()));
        dict.insert("group".to_string(), Value::String("".to_string()));
    }

    // Absolute Path
    let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    dict.insert(
        "absolute_path".to_string(),
        Value::String(abs_path.to_string_lossy().to_string()),
    );

    // Times
    if let Ok(m) = metadata.modified() {
        if let Ok(d) = m.duration_since(::std::time::UNIX_EPOCH) {
            dict.insert(
                "modified".to_string(),
                Value::String(d.as_secs().to_string()),
            );
        }
    }

    Ok(dict)
}

fn replace_impl(path: String, pattern: String, value: String, all: bool) -> AnyhowResult<()> {
    let data = fs::read(&path)?;
    let re = Regex::new(&pattern)?;

    let result = if all {
        re.replace_all(&data, NoExpand(value.as_bytes()))
    } else {
        re.replace(&data, NoExpand(value.as_bytes()))
    };

    fs::write(&path, result)?;
    Ok(())
}

fn template_impl(
    template_path: String,
    dst: String,
    args: BTreeMap<String, Value>,
    autoescape: bool,
) -> AnyhowResult<()> {
    let mut context = TeraContext::new();
    for (k, v) in args {
        // Convert Value to serde_json::Value
        let json_val = value_to_json(v);
        context.insert(k, &json_val);
    }

    let data = fs::read(&template_path)?;
    let template_content = String::from_utf8_lossy(&data);

    let res_content = Tera::one_off(&template_content, &context, autoescape)?;
    fs::write(&dst, res_content)?;
    Ok(())
}

fn value_to_json(v: Value) -> serde_json::Value {
    use serde_json::Value as JsonValue;
    match v {
        Value::None => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(b),
        Value::Int(i) => JsonValue::Number(serde_json::Number::from(i)),
        Value::Float(f) => serde_json::Number::from_f64(f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        Value::String(s) => JsonValue::String(s),
        Value::List(l) => {
            let list = l.read();
            let vec: Vec<JsonValue> = list.iter().map(|v| value_to_json(v.clone())).collect();
            JsonValue::Array(vec)
        }
        Value::Dictionary(d) => {
            let dict = d.read();
            let mut map = serde_json::Map::new();
            for (k, v) in dict.iter() {
                if let Value::String(key) = k {
                    map.insert(key.clone(), value_to_json(v.clone()));
                } else {
                    map.insert(k.to_string(), value_to_json(v.clone()));
                }
            }
            JsonValue::Object(map)
        }
        _ => JsonValue::String(format!("{v}")), // Fallback for types not easily mappable
    }
}

fn find_impl(
    path: String,
    name: Option<String>,
    file_type: Option<String>,
    permissions: Option<i64>,
    modified_time: Option<i64>,
    create_time: Option<i64>,
) -> AnyhowResult<Vec<String>> {
    let mut out = Vec::new();
    let root = Path::new(&path);
    if !root.is_dir() {
        return Ok(out);
    }

    // Recursive search
    find_recursive(
        root,
        &mut out,
        &name,
        &file_type,
        permissions,
        modified_time,
        create_time,
    )?;

    Ok(out)
}

fn find_recursive(
    dir: &Path,
    out: &mut Vec<String>,
    name: &Option<String>,
    file_type: &Option<String>,
    permissions: Option<i64>,
    modified_time: Option<i64>,
    create_time: Option<i64>,
) -> AnyhowResult<()> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_recursive(
                    &path,
                    out,
                    name,
                    file_type,
                    permissions,
                    modified_time,
                    create_time,
                )?;
            }

            if check_path(
                &path,
                name,
                file_type,
                permissions,
                modified_time,
                create_time,
            )? {
                if let Ok(p) = path.canonicalize() {
                    out.push(p.to_string_lossy().to_string());
                } else {
                    out.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(())
}

fn check_path(
    path: &Path,
    name: &Option<String>,
    file_type: &Option<String>,
    permissions: Option<i64>,
    modified_time: Option<i64>,
    create_time: Option<i64>,
) -> AnyhowResult<bool> {
    if let Some(n) = name {
        if let Some(fname) = path.file_name() {
            if !fname.to_string_lossy().contains(n) {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    }

    if let Some(ft) = file_type {
        if ft == "file" && !path.is_file() {
            return Ok(false);
        }
        if ft == "dir" && !path.is_dir() {
            return Ok(false);
        }
    }

    // Note: Permissions check on V1 was strict (==).
    if let Some(p) = permissions {
        #[cfg(unix)]
        {
            use ::std::os::unix::fs::PermissionsExt;
            let meta = path.metadata()?;
            if (meta.permissions().mode() & 0o777) as i64 != p {
                return Ok(false);
            }
        }
    }

    if let Some(mt) = modified_time {
        let meta = path.metadata()?;
        if let Ok(t) = meta.modified() {
            if t.duration_since(::std::time::UNIX_EPOCH)?.as_secs() as i64 != mt {
                return Ok(false);
            }
        }
    }

    if let Some(ct) = create_time {
        let meta = path.metadata()?;
        if let Ok(t) = meta.created() {
            if t.duration_since(::std::time::UNIX_EPOCH)?.as_secs() as i64 != ct {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

// Tests
#[cfg(test)]
mod tests {

    // use sha256::try_digest; // Removed per error
    use super::*;
    use alloc::collections::BTreeMap;
    use eldritch_core::Value;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_ops_basic() -> AnyhowResult<()> {
        let lib = StdFileLibrary;
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_string_lossy().to_string();

        // Write
        lib.write(path.clone(), "hello".to_string()).unwrap();
        assert_eq!(lib.read(path.clone()).unwrap(), "hello");

        // Append
        lib.append(path.clone(), " world".to_string()).unwrap();
        assert_eq!(lib.read(path.clone()).unwrap(), "hello world");

        // Exists
        assert!(lib.exists(path.clone()).unwrap());

        // Remove
        // Note: NamedTempFile removes on drop, but we can try removing it manually
        // lib.remove(path.clone()).unwrap();
        // assert!(!lib.exists(path.clone()).unwrap());

        Ok(())
    }

    #[test]
    fn test_compress_decompress() -> AnyhowResult<()> {
        let lib = StdFileLibrary;
        let content = "Compression Test";

        let tmp_src = NamedTempFile::new()?;
        let src_path = tmp_src.path().to_string_lossy().to_string();
        fs::write(&src_path, content)?;

        let tmp_dst = NamedTempFile::new()?;
        let dst_path = tmp_dst.path().to_string_lossy().to_string();

        lib.compress(src_path.clone(), dst_path.clone()).unwrap();

        let tmp_out = NamedTempFile::new()?;
        let out_path = tmp_out.path().to_string_lossy().to_string();

        lib.decompress(dst_path, out_path.clone()).unwrap();

        let res = fs::read_to_string(&out_path)?;
        assert_eq!(res, content);

        Ok(())
    }

    #[test]
    fn test_template() -> AnyhowResult<()> {
        let lib = StdFileLibrary;
        let tmp_tmpl = NamedTempFile::new()?;
        let tmpl_path = tmp_tmpl.path().to_string_lossy().to_string();

        fs::write(&tmpl_path, "Hello {{ name }}")?;

        let tmp_out = NamedTempFile::new()?;
        let out_path = tmp_out.path().to_string_lossy().to_string();

        let mut args = BTreeMap::new();
        args.insert("name".to_string(), Value::String("World".to_string()));

        lib.template(tmpl_path, out_path.clone(), args, true)
            .unwrap();

        assert_eq!(fs::read_to_string(&out_path)?, "Hello World");

        Ok(())
    }

    #[test]
    fn test_list_owner_group() -> AnyhowResult<()> {
        let lib = StdFileLibrary;
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_string_lossy().to_string();

        let files = lib.list(path).unwrap();
        assert_eq!(files.len(), 1);
        let f = &files[0];

        assert!(f.contains_key("owner"));
        assert!(f.contains_key("group"));
        assert!(f.contains_key("absolute_path"));

        // On unix, owner/group should not be empty (usually)
        // But in some containers or weird envs, it might be stringified ID.
        // We just check presence as requested by the user's error message.

        // Check absolute_path
        if let Value::String(abs) = &f["absolute_path"] {
            assert!(!abs.is_empty());
            assert!(std::path::Path::new(abs).is_absolute());
        } else {
            panic!("absolute_path is not a string");
        }

        Ok(())
    }

    #[test]
    fn test_follow() -> AnyhowResult<()> {
        // We verify that follow can be called and executes callback.
        // Since it's blocking, we use a callback that throws an error to exit the loop.

        let lib = StdFileLibrary;
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_string_lossy().to_string();

        // Write initial content
        lib.write(path.clone(), "line1\n".to_string()).unwrap();

        // Create a thread to update file after a delay, to trigger watcher
        let path_clone = path.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            // Append line
             let mut file = OpenOptions::new()
                .append(true)
                .open(path_clone)
                .unwrap();
            file.write_all(b"line2\n").unwrap();
        });

        // Define a native function value that simulates a callback throwing an error on specific input
        // Since we can't easily construct a Value::Function here without parsing (unless we use Interpreter to make one),
        // we can use Interpreter to create the value.

        let mut interp = Interpreter::new();
        let code = r#"
def cb(line):
    if line == "line2":
        fail("STOP")
cb
"#;
        let fn_val = interp.interpret(code).map_err(|e| anyhow::anyhow!(e))?;

        // Call follow. It should block until "line2" is written, then cb is called, throws error, and follow returns Err.
        let res = lib.follow(path, fn_val);

        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("STOP"));

        Ok(())
    }
}
