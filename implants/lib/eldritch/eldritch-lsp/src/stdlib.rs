use serde::Deserialize;
use std::collections::BTreeMap;

const ELDRITCH_DOCS_JSON: &str =
    include_str!("../../../../../tavern/internal/www/src/assets/eldritch-docs.json");

#[derive(Debug, Deserialize)]
struct RawDocEntry {
    #[allow(dead_code)]
    signature: String,
    description: String,
}

#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub name: &'static str,
    pub ty: &'static str,
    pub required: bool,
    pub default_val: Option<&'static str>,
}

impl ParamSpec {
    pub const fn req(name: &'static str, ty: &'static str) -> Self {
        Self {
            name,
            ty,
            required: true,
            default_val: None,
        }
    }

    pub const fn opt(name: &'static str, ty: &'static str, default_val: &'static str) -> Self {
        Self {
            name,
            ty,
            required: false,
            default_val: Some(default_val),
        }
    }

    pub fn format_label(&self) -> String {
        if let Some(def) = self.default_val {
            if self.ty.is_empty() {
                format!("{}={}", self.name, def)
            } else {
                format!("{}: {} = {}", self.name, self.ty, def)
            }
        } else if self.ty.is_empty() {
            self.name.to_string()
        } else {
            format!("{}: {}", self.name, self.ty)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FuncSpec {
    pub name: &'static str,
    pub qualified_name: &'static str,
    pub module: Option<&'static str>,
    pub params: Vec<ParamSpec>,
    pub return_type: &'static str,
    pub varargs: bool,
    pub kwargs: bool,
    pub deprecated_replacement: Option<&'static str>,
    pub description: String,
}

impl FuncSpec {
    pub fn min_args(&self) -> usize {
        self.params.iter().filter(|p| p.required).count()
    }

    pub fn max_args(&self) -> Option<usize> {
        if self.varargs {
            None
        } else {
            Some(self.params.len())
        }
    }

    pub fn signature_string(&self) -> String {
        let mut parts: Vec<String> = self.params.iter().map(|p| p.format_label()).collect();
        if self.varargs && !parts.iter().any(|p| p.starts_with('*')) {
            parts.push("*args".to_string());
        }
        if self.kwargs && !parts.iter().any(|p| p.starts_with("**")) {
            parts.push("**kwargs".to_string());
        }
        format!(
            "{}({}) -> {}",
            self.qualified_name,
            parts.join(", "),
            self.return_type
        )
    }

    pub fn snippet(&self) -> String {
        let req_params: Vec<&ParamSpec> = self
            .params
            .iter()
            .filter(|p| p.required && !p.name.starts_with('*'))
            .collect();
        if req_params.is_empty() {
            format!("{}()$0", self.name)
        } else {
            let placeholders: Vec<String> = req_params
                .iter()
                .enumerate()
                .map(|(idx, p)| format!("${{{}:{}}}", idx + 1, p.name))
                .collect();
            format!("{}({})$0", self.name, placeholders.join(", "))
        }
    }

    pub fn markdown_docs(&self) -> String {
        let mut md = format!("```python\n{}\n```\n", self.signature_string());
        if let Some(repl) = self.deprecated_replacement {
            md.push_str(&format!(
                "\n> **Deprecated in Eldritch 0.4.1:** Use `{}` instead.\n",
                repl
            ));
        }
        if !self.description.is_empty() {
            md.push('\n');
            md.push_str(self.description.trim_end_matches("---").trim());
        }
        md
    }
}

#[derive(Debug, Clone)]
pub struct ModuleSpec {
    pub name: &'static str,
    pub description: String,
    pub deprecated_replacement: Option<&'static str>,
    pub functions: BTreeMap<String, FuncSpec>,
    pub removed_functions: BTreeMap<&'static str, &'static str>,
}

impl ModuleSpec {
    pub fn markdown_docs(&self) -> String {
        let mut md = format!("### Module `{}`\n\n", self.name);
        if let Some(repl) = self.deprecated_replacement {
            md.push_str(&format!("> **Deprecated in Eldritch 0.4.1:** {}\n\n", repl));
        }
        md.push_str(self.description.trim_end_matches("---").trim());
        let active_funcs: Vec<&FuncSpec> = self
            .functions
            .values()
            .filter(|f| f.deprecated_replacement.is_none() && !f.name.starts_with('_'))
            .collect();
        if !active_funcs.is_empty() {
            md.push_str("\n\n**Functions:**\n");
            for f in active_funcs {
                md.push_str(&format!("- `{}`\n", f.signature_string()));
            }
        }
        md
    }
}

#[derive(Debug, Clone)]
pub struct MethodSpec {
    pub receiver_type: &'static str,
    pub name: &'static str,
    pub params: Vec<ParamSpec>,
    pub return_type: &'static str,
    pub description: &'static str,
}

impl MethodSpec {
    pub fn signature_string(&self) -> String {
        let parts: Vec<String> = self.params.iter().map(|p| p.format_label()).collect();
        format!(
            "{}.{}({}) -> {}",
            self.receiver_type,
            self.name,
            parts.join(", "),
            self.return_type
        )
    }

    pub fn markdown_docs(&self) -> String {
        format!(
            "```python\n{}\n```\n\n{}",
            self.signature_string(),
            self.description
        )
    }
}

#[derive(Debug, Clone)]
pub struct StdlibIndex {
    pub modules: BTreeMap<String, ModuleSpec>,
    pub builtins: BTreeMap<String, FuncSpec>,
    pub methods: BTreeMap<String, Vec<MethodSpec>>,
}

impl Default for StdlibIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl StdlibIndex {
    pub fn new() -> Self {
        let raw_docs: BTreeMap<String, RawDocEntry> =
            serde_json::from_str(ELDRITCH_DOCS_JSON).unwrap_or_default();

        let doc_for = |key: &str, fallback: &str| -> String {
            raw_docs
                .get(key)
                .map(|d| d.description.trim_end_matches("---").trim().to_string())
                .unwrap_or_else(|| fallback.to_string())
        };

        let mut modules = BTreeMap::new();
        let mut add_module =
            |name: &'static str,
             fallback_desc: &str,
             deprecated_replacement: Option<&'static str>,
             funcs: Vec<FuncSpec>,
             removed: Vec<(&'static str, &'static str)>| {
                let mut fn_map = BTreeMap::new();
                for mut f in funcs {
                    if f.description.is_empty() {
                        f.description = doc_for(f.qualified_name, "");
                    }
                    fn_map.insert(f.name.to_string(), f);
                }
                let mut rem_map = BTreeMap::new();
                for (k, v) in removed {
                    rem_map.insert(k, v);
                }
                modules.insert(
                    name.to_string(),
                    ModuleSpec {
                        name,
                        description: doc_for(name, fallback_desc),
                        deprecated_replacement,
                        functions: fn_map,
                        removed_functions: rem_map,
                    },
                );
            };

        let mfn = |module: &'static str,
                   name: &'static str,
                   qualified_name: &'static str,
                   params: Vec<ParamSpec>,
                   return_type: &'static str,
                   deprecated_replacement: Option<&'static str>,
                   fallback_desc: &str|
         -> FuncSpec {
            FuncSpec {
                name,
                qualified_name,
                module: Some(module),
                params,
                return_type,
                varargs: false,
                kwargs: false,
                deprecated_replacement,
                description: doc_for(qualified_name, fallback_desc),
            }
        };

        // 1. aes
        add_module(
            "aes",
            "AES encryption and decryption functions for raw bytes and files.",
            None,
            vec![
                mfn(
                    "aes",
                    "decrypt",
                    "aes.decrypt",
                    vec![
                        ParamSpec::req("key", "Bytes | str"),
                        ParamSpec::req("iv", "Bytes | str"),
                        ParamSpec::req("data", "Bytes"),
                    ],
                    "Bytes",
                    None,
                    "Decrypts data using AES-CBC with the given 16-byte key and IV.",
                ),
                mfn(
                    "aes",
                    "decrypt_file",
                    "aes.decrypt_file",
                    vec![
                        ParamSpec::req("src", "str"),
                        ParamSpec::req("dst", "str"),
                        ParamSpec::req("key", "str"),
                    ],
                    "None",
                    None,
                    "Decrypts the src file using the given 16-byte key and writes to dst.",
                ),
                mfn(
                    "aes",
                    "encrypt",
                    "aes.encrypt",
                    vec![
                        ParamSpec::req("key", "Bytes | str"),
                        ParamSpec::req("iv", "Bytes | str"),
                        ParamSpec::req("data", "Bytes | str"),
                    ],
                    "Bytes",
                    None,
                    "Encrypts data using AES-CBC with the given 16-byte key and IV.",
                ),
                mfn(
                    "aes",
                    "encrypt_file",
                    "aes.encrypt_file",
                    vec![
                        ParamSpec::req("src", "str"),
                        ParamSpec::req("dst", "str"),
                        ParamSpec::req("key", "str"),
                    ],
                    "None",
                    None,
                    "Encrypts the src file using the given 16-byte key and writes to dst.",
                ),
            ],
            vec![],
        );

        // 2. agent
        add_module(
            "agent",
            "Functions for interacting with the running agent configuration, transports, and tasks.",
            None,
            vec![
                mfn(
                    "agent",
                    "callback_interval",
                    "agent.callback_interval",
                    vec![],
                    "int",
                    None,
                    "Returns the current callback interval in seconds.",
                ),
                mfn(
                    "agent",
                    "config",
                    "agent.config",
                    vec![],
                    "Dict[str, Any]",
                    None,
                    "Returns the current configuration of the agent as a dictionary.",
                ),
                mfn(
                    "agent",
                    "list_tasks",
                    "agent.list_tasks",
                    vec![],
                    "List[Dict]",
                    None,
                    "Returns a list of currently running or queued tasks on the agent.",
                ),
                mfn(
                    "agent",
                    "list_transports",
                    "agent.list_transports",
                    vec![],
                    "List[str]",
                    None,
                    "Returns a list of available transport names supported by the agent.",
                ),
                mfn(
                    "agent",
                    "reset_transport",
                    "agent.reset_transport",
                    vec![],
                    "None",
                    None,
                    "Resets the active agent transport.",
                ),
                mfn(
                    "agent",
                    "set_callback_interval",
                    "agent.set_callback_interval",
                    vec![ParamSpec::req("interval", "int")],
                    "None",
                    None,
                    "Changes the running agent's callback interval (in seconds).",
                ),
                mfn(
                    "agent",
                    "set_callback_uri",
                    "agent.set_callback_uri",
                    vec![ParamSpec::req("uri", "str")],
                    "None",
                    None,
                    "Changes the running agent's callback URI.",
                ),
                mfn(
                    "agent",
                    "stop_task",
                    "agent.stop_task",
                    vec![ParamSpec::req("task_id", "int")],
                    "None",
                    None,
                    "Stops a specific background task by its ID.",
                ),
                mfn(
                    "agent",
                    "transport",
                    "agent.transport",
                    vec![],
                    "str",
                    None,
                    "Returns the name of the currently active transport.",
                ),
                mfn(
                    "agent",
                    "_terminate_this_process_clowntown",
                    "agent._terminate_this_process_clowntown",
                    vec![],
                    "None",
                    None,
                    "Terminates the agent process immediately.",
                ),
                // Deprecated in 0.4.1
                mfn(
                    "agent",
                    "get_callback_interval",
                    "agent.get_callback_interval",
                    vec![],
                    "int",
                    Some("agent.callback_interval()"),
                    "Deprecated: use `agent.callback_interval()` instead.",
                ),
                mfn(
                    "agent",
                    "get_config",
                    "agent.get_config",
                    vec![],
                    "Dict[str, Any]",
                    Some("agent.config()"),
                    "Deprecated: use `agent.config()` instead.",
                ),
                mfn(
                    "agent",
                    "get_transport",
                    "agent.get_transport",
                    vec![],
                    "str",
                    Some("agent.transport()"),
                    "Deprecated: use `agent.transport()` instead.",
                ),
            ],
            vec![],
        );

        // 3. assets
        add_module(
            "assets",
            "Interact with embedded files bundled with the tome or agent.",
            None,
            vec![
                mfn(
                    "assets",
                    "copy",
                    "assets.copy",
                    vec![ParamSpec::req("src", "str"), ParamSpec::req("dest", "str")],
                    "None",
                    None,
                    "Copies an embedded asset file to disk at `dest`.",
                ),
                mfn(
                    "assets",
                    "list",
                    "assets.list",
                    vec![],
                    "List[str]",
                    None,
                    "Returns a list of embedded asset file names.",
                ),
                mfn(
                    "assets",
                    "read",
                    "assets.read",
                    vec![ParamSpec::req("name", "str")],
                    "Bytes",
                    None,
                    "Reads an embedded asset file and returns its contents as `Bytes`.",
                ),
            ],
            vec![(
                "read_binary",
                "assets.read(name) (returns Bytes in Eldritch 0.4.1)",
            )],
        );

        // 4. base64
        add_module(
            "base64",
            "Base64 encoding and decoding functions.",
            None,
            vec![
                mfn(
                    "base64",
                    "decode",
                    "base64.decode",
                    vec![
                        ParamSpec::req("content", "str"),
                        ParamSpec::opt("encode_type", "Optional[str]", "None"),
                    ],
                    "Bytes",
                    None,
                    "Decodes base64 text into `Bytes`.",
                ),
                mfn(
                    "base64",
                    "encode",
                    "base64.encode",
                    vec![
                        ParamSpec::req("content", "Bytes | str"),
                        ParamSpec::opt("encode_type", "Optional[str]", "None"),
                    ],
                    "str",
                    None,
                    "Encodes `Bytes` or `str` into a base64 string.",
                ),
            ],
            vec![],
        );

        // 5. chain
        add_module(
            "chain",
            "Multi-agent C2 chaining operations.",
            None,
            vec![mfn(
                "chain",
                "tcp",
                "chain.tcp",
                vec![ParamSpec::req("addr", "str")],
                "None",
                None,
                "Connects to a downstream chained agent over TCP at `addr` (host:port) and proxies traffic.",
            )],
            vec![],
        );

        // 6. dns
        add_module(
            "dns",
            "DNS resolution and record lookup functions.",
            None,
            vec![mfn(
                "dns",
                "list",
                "dns.list",
                vec![
                    ParamSpec::req("domain", "str"),
                    ParamSpec::opt("kind", "Optional[str]", "None"),
                    ParamSpec::opt("nameserver", "Optional[str]", "None"),
                ],
                "List[str]",
                None,
                "Queries DNS records for `domain`.",
            )],
            vec![],
        );

        // 7. file
        add_module(
            "file",
            "Filesystem inspection and manipulation operations.",
            None,
            vec![
                mfn(
                    "file",
                    "append",
                    "file.append",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::req("content", "Bytes | str"),
                    ],
                    "None",
                    None,
                    "Appends `content` (`Bytes` or `str`) to `path`.",
                ),
                mfn(
                    "file",
                    "copy",
                    "file.copy",
                    vec![ParamSpec::req("src", "str"), ParamSpec::req("dst", "str")],
                    "None",
                    None,
                    "Copies a file from `src` to `dst`.",
                ),
                mfn(
                    "file",
                    "exists",
                    "file.exists",
                    vec![ParamSpec::req("path", "str")],
                    "bool",
                    None,
                    "Returns `True` if `path` exists on disk.",
                ),
                mfn(
                    "file",
                    "find",
                    "file.find",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::opt("name", "Optional[str]", "None"),
                        ParamSpec::opt("file_type", "Optional[str]", "None"),
                        ParamSpec::opt("permissions", "Optional[int]", "None"),
                        ParamSpec::opt("modified_time", "Optional[int]", "None"),
                        ParamSpec::opt("create_time", "Optional[int]", "None"),
                    ],
                    "List[str]",
                    None,
                    "Finds files under `path` matching the specified filter criteria.",
                ),
                mfn(
                    "file",
                    "follow",
                    "file.follow",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::req("fn_val", "Callable"),
                    ],
                    "None",
                    None,
                    "Follows changes to `path` and invokes `fn_val(line)` for new lines.",
                ),
                mfn(
                    "file",
                    "is_dir",
                    "file.is_dir",
                    vec![ParamSpec::req("path", "str")],
                    "bool",
                    None,
                    "Returns `True` if `path` exists and is a directory.",
                ),
                mfn(
                    "file",
                    "is_file",
                    "file.is_file",
                    vec![ParamSpec::req("path", "str")],
                    "bool",
                    None,
                    "Returns `True` if `path` exists and is a regular file.",
                ),
                mfn(
                    "file",
                    "list",
                    "file.list",
                    vec![ParamSpec::opt("path", "Optional[str]", "None")],
                    "List[Dict]",
                    None,
                    "Lists directory entries at `path` (supports glob patterns).",
                ),
                mfn(
                    "file",
                    "list_named_pipes",
                    "file.list_named_pipes",
                    vec![ParamSpec::opt("detailed", "Optional[bool]", "None")],
                    "List[str] | List[Dict]",
                    None,
                    "Lists named pipes on Windows.",
                ),
                mfn(
                    "file",
                    "list_recent",
                    "file.list_recent",
                    vec![
                        ParamSpec::opt("path", "Optional[str]", "None"),
                        ParamSpec::opt("limit", "Optional[int]", "None"),
                    ],
                    "List[str]",
                    None,
                    "Lists the most recently modified files under `path`.",
                ),
                mfn(
                    "file",
                    "mkdir",
                    "file.mkdir",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::opt("parent", "Optional[bool]", "None"),
                    ],
                    "None",
                    None,
                    "Creates a directory at `path`.",
                ),
                mfn(
                    "file",
                    "move",
                    "file.move",
                    vec![ParamSpec::req("src", "str"), ParamSpec::req("dst", "str")],
                    "None",
                    None,
                    "Moves or renames `src` to `dst`.",
                ),
                mfn(
                    "file",
                    "parent_dir",
                    "file.parent_dir",
                    vec![ParamSpec::req("path", "str")],
                    "str",
                    None,
                    "Returns the parent directory of `path`.",
                ),
                mfn(
                    "file",
                    "read",
                    "file.read",
                    vec![ParamSpec::req("path", "str")],
                    "Bytes",
                    None,
                    "Reads the contents of `path` (supports globs) and returns `Bytes`.",
                ),
                mfn(
                    "file",
                    "read_named_pipe",
                    "file.read_named_pipe",
                    vec![
                        ParamSpec::req("name", "str"),
                        ParamSpec::opt("max_bytes", "Optional[int]", "None"),
                    ],
                    "str",
                    None,
                    "Reads available data from a Windows named pipe without blocking indefinitely.",
                ),
                mfn(
                    "file",
                    "remove",
                    "file.remove",
                    vec![ParamSpec::req("path", "str")],
                    "None",
                    None,
                    "Removes the file or directory at `path`.",
                ),
                mfn(
                    "file",
                    "replace",
                    "file.replace",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::req("pattern", "str"),
                        ParamSpec::req("value", "str"),
                    ],
                    "None",
                    None,
                    "Replaces the first match of regex `pattern` in `path` with `value`.",
                ),
                mfn(
                    "file",
                    "replace_all",
                    "file.replace_all",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::req("pattern", "str"),
                        ParamSpec::req("value", "str"),
                    ],
                    "None",
                    None,
                    "Replaces all matches of regex `pattern` in `path` with `value`.",
                ),
                mfn(
                    "file",
                    "timestomp",
                    "file.timestomp",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::opt("mtime", "Optional[int | str]", "None"),
                        ParamSpec::opt("atime", "Optional[int | str]", "None"),
                        ParamSpec::opt("ctime", "Optional[int | str]", "None"),
                        ParamSpec::opt("ref_file", "Optional[str]", "None"),
                    ],
                    "None",
                    None,
                    "Modifies timestamps (`mtime`, `atime`, `ctime`) on `path`.",
                ),
                mfn(
                    "file",
                    "tmp",
                    "file.tmp",
                    vec![ParamSpec::opt("name", "Optional[str]", "None")],
                    "str",
                    None,
                    "Creates or returns a temporary file path.",
                ),
                mfn(
                    "file",
                    "tmp_dir",
                    "file.tmp_dir",
                    vec![],
                    "str",
                    None,
                    "Returns the system temporary directory path.",
                ),
                mfn(
                    "file",
                    "which",
                    "file.which",
                    vec![ParamSpec::req("bin", "str")],
                    "str",
                    None,
                    "Finds the full executable path of `bin` on `PATH`.",
                ),
                mfn(
                    "file",
                    "write",
                    "file.write",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::req("content", "Bytes | str"),
                    ],
                    "None",
                    None,
                    "Writes `content` (`Bytes` or `str`) to a new file at `path`.",
                ),
                // Deprecated on `file` in 0.4.1
                mfn(
                    "file",
                    "compress",
                    "file.compress",
                    vec![ParamSpec::req("src", "str"), ParamSpec::req("dst", "str")],
                    "None",
                    Some("gzip.compress_file(src, dst)"),
                    "Deprecated: use `gzip.compress_file(src, dst)` instead.",
                ),
                mfn(
                    "file",
                    "decompress",
                    "file.decompress",
                    vec![ParamSpec::req("src", "str"), ParamSpec::req("dst", "str")],
                    "None",
                    Some("gzip.decompress_file(src, dst)"),
                    "Deprecated: use `gzip.decompress_file(src, dst)` instead.",
                ),
                mfn(
                    "file",
                    "pwd",
                    "file.pwd",
                    vec![],
                    "Optional[str]",
                    Some("process.cwd()"),
                    "Deprecated: use `process.cwd()` instead.",
                ),
                mfn(
                    "file",
                    "temp_file",
                    "file.temp_file",
                    vec![ParamSpec::opt("name", "Optional[str]", "None")],
                    "str",
                    Some("file.tmp(name)"),
                    "Deprecated: use `file.tmp(name)` instead.",
                ),
                mfn(
                    "file",
                    "template",
                    "file.template",
                    vec![
                        ParamSpec::req("template_path", "str"),
                        ParamSpec::req("dst", "str"),
                        ParamSpec::req("args", "Dict[str, Any]"),
                        ParamSpec::req("autoescape", "bool"),
                    ],
                    "None",
                    Some("template.render_file(template_path, dst, args, autoescape)"),
                    "Deprecated: use `template.render_file(...)` instead.",
                ),
                mfn(
                    "file",
                    "template_str",
                    "file.template_str",
                    vec![
                        ParamSpec::req("template", "str"),
                        ParamSpec::req("args", "Dict[str, Any]"),
                        ParamSpec::req("autoescape", "bool"),
                    ],
                    "str",
                    Some("template.render(template, args, autoescape)"),
                    "Deprecated: use `template.render(...)` instead.",
                ),
            ],
            vec![
                (
                    "read_binary",
                    "file.read(path) (returns Bytes in Eldritch 0.4.1)",
                ),
                (
                    "write_binary",
                    "file.write(path, content) (accepts Bytes or str in Eldritch 0.4.1)",
                ),
                (
                    "append_binary",
                    "file.append(path, content) (accepts Bytes or str in Eldritch 0.4.1)",
                ),
            ],
        );

        // 8. gzip
        add_module(
            "gzip",
            "Gzip compression and decompression for raw bytes and files.",
            None,
            vec![
                mfn(
                    "gzip",
                    "compress",
                    "gzip.compress",
                    vec![ParamSpec::req("data", "Bytes | str")],
                    "Bytes",
                    None,
                    "Compresses `data` using gzip and returns compressed `Bytes`.",
                ),
                mfn(
                    "gzip",
                    "compress_file",
                    "gzip.compress_file",
                    vec![ParamSpec::req("src", "str"), ParamSpec::req("dst", "str")],
                    "None",
                    None,
                    "Compresses file `src` using gzip and writes it to `dst`.",
                ),
                mfn(
                    "gzip",
                    "decompress",
                    "gzip.decompress",
                    vec![ParamSpec::req("data", "Bytes")],
                    "Bytes",
                    None,
                    "Decompresses gzip-compressed `Bytes`.",
                ),
                mfn(
                    "gzip",
                    "decompress_file",
                    "gzip.decompress_file",
                    vec![ParamSpec::req("src", "str"), ParamSpec::req("dst", "str")],
                    "None",
                    None,
                    "Decompresses gzip file `src` and writes it to `dst`.",
                ),
            ],
            vec![],
        );

        // 9. hash
        add_module(
            "hash",
            "Cryptographic hashing functions (`md5`, `sha1`, `sha256`, `file`).",
            None,
            vec![
                mfn(
                    "hash",
                    "file",
                    "hash.file",
                    vec![
                        ParamSpec::req("file", "str"),
                        ParamSpec::opt("algo", "Optional[str]", "\"sha256\""),
                    ],
                    "str",
                    None,
                    "Computes the hex digest of `file` using `algo` (`md5`, `sha1`, or `sha256`).",
                ),
                mfn(
                    "hash",
                    "md5",
                    "hash.md5",
                    vec![ParamSpec::req("data", "Bytes | str")],
                    "str",
                    None,
                    "Returns the MD5 hex digest of `data`.",
                ),
                mfn(
                    "hash",
                    "sha1",
                    "hash.sha1",
                    vec![ParamSpec::req("data", "Bytes | str")],
                    "str",
                    None,
                    "Returns the SHA-1 hex digest of `data`.",
                ),
                mfn(
                    "hash",
                    "sha256",
                    "hash.sha256",
                    vec![ParamSpec::req("data", "Bytes | str")],
                    "str",
                    None,
                    "Returns the SHA-256 hex digest of `data`.",
                ),
            ],
            vec![],
        );

        // 10. http
        add_module(
            "http",
            "HTTP and HTTPS client requests and file downloads.",
            None,
            vec![
                mfn(
                    "http",
                    "download",
                    "http.download",
                    vec![
                        ParamSpec::req("uri", "str"),
                        ParamSpec::req("dst", "str"),
                        ParamSpec::opt("allow_insecure", "Optional[bool]", "None"),
                    ],
                    "None",
                    None,
                    "Downloads a file from `uri` to `dst` on disk.",
                ),
                mfn(
                    "http",
                    "get",
                    "http.get",
                    vec![
                        ParamSpec::req("uri", "str"),
                        ParamSpec::opt("query_params", "Optional[Dict[str, str]]", "None"),
                        ParamSpec::opt("headers", "Optional[Dict[str, str]]", "None"),
                        ParamSpec::opt("allow_insecure", "Optional[bool]", "None"),
                    ],
                    "Dict[str, Any]",
                    None,
                    "Sends an HTTP GET request to `uri`.",
                ),
                mfn(
                    "http",
                    "post",
                    "http.post",
                    vec![
                        ParamSpec::req("uri", "str"),
                        ParamSpec::opt("body", "Optional[Bytes | str]", "None"),
                        ParamSpec::opt("form", "Optional[Dict[str, str]]", "None"),
                        ParamSpec::opt("headers", "Optional[Dict[str, str]]", "None"),
                        ParamSpec::opt("allow_insecure", "Optional[bool]", "None"),
                    ],
                    "Dict[str, Any]",
                    None,
                    "Sends an HTTP POST request to `uri`.",
                ),
            ],
            vec![],
        );

        // 11. json
        add_module(
            "json",
            "JSON serialization, deserialization, and validation.",
            None,
            vec![
                mfn(
                    "json",
                    "decode",
                    "json.decode",
                    vec![ParamSpec::req("content", "Bytes | str")],
                    "Any",
                    None,
                    "Parses a JSON string or bytes into an Eldritch value.",
                ),
                mfn(
                    "json",
                    "encode",
                    "json.encode",
                    vec![ParamSpec::req("content", "Any")],
                    "str",
                    None,
                    "Serializes an Eldritch value into a JSON string.",
                ),
                mfn(
                    "json",
                    "is_valid",
                    "json.is_valid",
                    vec![ParamSpec::req("content", "Bytes | str")],
                    "bool",
                    None,
                    "Returns `True` if `content` is valid JSON.",
                ),
            ],
            vec![],
        );

        // 12. linux
        add_module(
            "linux",
            "Linux-specific system inspection operations.",
            None,
            vec![mfn(
                "linux",
                "uname",
                "linux.uname",
                vec![],
                "Dict[str, str]",
                None,
                "Returns Linux kernel and machine information from `uname`.",
            )],
            vec![],
        );

        // 13. net
        add_module(
            "net",
            "Network interfaces, active connections, and TCP/UDP client operations.",
            None,
            vec![
                mfn(
                    "net",
                    "conns",
                    "net.conns",
                    vec![],
                    "List[Dict]",
                    None,
                    "Returns active network connections (TCP/UDP sockets and owning PIDs).",
                ),
                mfn(
                    "net",
                    "interfaces",
                    "net.interfaces",
                    vec![],
                    "List[Dict]",
                    None,
                    "Returns network interfaces and their assigned IP addresses.",
                ),
                mfn(
                    "net",
                    "tcp",
                    "net.tcp",
                    vec![
                        ParamSpec::req("address", "str"),
                        ParamSpec::req("port", "int"),
                        ParamSpec::req("data", "Bytes | str"),
                    ],
                    "str",
                    None,
                    "Sends `data` over TCP to `address:port` and returns the response.",
                ),
                mfn(
                    "net",
                    "udp",
                    "net.udp",
                    vec![
                        ParamSpec::req("address", "str"),
                        ParamSpec::req("port", "int"),
                        ParamSpec::req("data", "Bytes | str"),
                    ],
                    "str",
                    None,
                    "Sends `data` over UDP to `address:port` and returns the response.",
                ),
            ],
            vec![],
        );

        // 14. portal
        add_module(
            "portal",
            "Bi-directional portal streams (SOCKS5 / port forwarding).",
            None,
            vec![mfn(
                "portal",
                "create",
                "portal.create",
                vec![],
                "None",
                None,
                "Opens a bi-directional portal stream on the current task.",
            )],
            vec![],
        );

        // 15. process
        add_module(
            "process",
            "Process enumeration, inspection, environment variables, and signal control.",
            None,
            vec![
                mfn(
                    "process",
                    "cwd",
                    "process.cwd",
                    vec![],
                    "Optional[str]",
                    None,
                    "Returns the current working directory of the agent process.",
                ),
                mfn(
                    "process",
                    "env",
                    "process.env",
                    vec![],
                    "Dict[str, str]",
                    None,
                    "Returns a dictionary of the current process's environment variables.",
                ),
                mfn(
                    "process",
                    "info",
                    "process.info",
                    vec![ParamSpec::opt("pid", "Optional[int]", "None")],
                    "Dict[str, Any]",
                    None,
                    "Returns detailed information about `pid` (or the current process if `None`).",
                ),
                mfn(
                    "process",
                    "kill",
                    "process.kill",
                    vec![ParamSpec::req("pid", "int")],
                    "None",
                    None,
                    "Terminates the process with the specified `pid`.",
                ),
                mfn(
                    "process",
                    "list",
                    "process.list",
                    vec![ParamSpec::opt("include_env", "Optional[bool]", "None")],
                    "List[Dict]",
                    None,
                    "Returns a list of running processes on the system.",
                ),
                mfn(
                    "process",
                    "name",
                    "process.name",
                    vec![ParamSpec::req("pid", "int")],
                    "str",
                    None,
                    "Returns the process name for the given `pid`.",
                ),
                mfn(
                    "process",
                    "pid",
                    "process.pid",
                    vec![],
                    "int",
                    None,
                    "Returns the process ID (PID) of the running agent.",
                ),
                mfn(
                    "process",
                    "signal",
                    "process.signal",
                    vec![
                        ParamSpec::req("pid", "int"),
                        ParamSpec::req("signal", "str"),
                    ],
                    "None",
                    None,
                    "Sends `signal` to the process with `pid` on Unix systems.",
                ),
                // Deprecated on `process` in 0.4.1
                mfn(
                    "process",
                    "netstat",
                    "process.netstat",
                    vec![],
                    "List[Dict]",
                    Some("net.conns()"),
                    "Deprecated: use `net.conns()` instead.",
                ),
            ],
            vec![],
        );

        // 16. random
        add_module(
            "random",
            "Cryptographically secure random value generation.",
            None,
            vec![
                mfn(
                    "random",
                    "bool",
                    "random.bool",
                    vec![],
                    "bool",
                    None,
                    "Returns a cryptographically random boolean.",
                ),
                mfn(
                    "random",
                    "bytes",
                    "random.bytes",
                    vec![ParamSpec::req("len", "int")],
                    "Bytes",
                    None,
                    "Returns `len` cryptographically random bytes.",
                ),
                mfn(
                    "random",
                    "int",
                    "random.int",
                    vec![ParamSpec::req("min", "int"), ParamSpec::req("max", "int")],
                    "int",
                    None,
                    "Returns a random integer in `[min, max)`.",
                ),
                mfn(
                    "random",
                    "string",
                    "random.string",
                    vec![
                        ParamSpec::req("len", "int"),
                        ParamSpec::opt("charset", "Optional[str]", "None"),
                    ],
                    "str",
                    None,
                    "Returns a random string of length `len`.",
                ),
                mfn(
                    "random",
                    "uuid",
                    "random.uuid",
                    vec![],
                    "str",
                    None,
                    "Generates a random UUID v4 string.",
                ),
            ],
            vec![],
        );

        // 17. regex
        add_module(
            "regex",
            "Regular expression matching and replacement operations.",
            None,
            vec![
                mfn(
                    "regex",
                    "match",
                    "regex.match",
                    vec![
                        ParamSpec::req("haystack", "str"),
                        ParamSpec::req("pattern", "str"),
                    ],
                    "str",
                    None,
                    "Returns the first capture group match of `pattern` in `haystack`.",
                ),
                mfn(
                    "regex",
                    "match_all",
                    "regex.match_all",
                    vec![
                        ParamSpec::req("haystack", "str"),
                        ParamSpec::req("pattern", "str"),
                    ],
                    "List[str]",
                    None,
                    "Returns all matches of `pattern` in `haystack`.",
                ),
                mfn(
                    "regex",
                    "replace",
                    "regex.replace",
                    vec![
                        ParamSpec::req("haystack", "str"),
                        ParamSpec::req("pattern", "str"),
                        ParamSpec::req("value", "str"),
                    ],
                    "str",
                    None,
                    "Replaces the first match of `pattern` in `haystack` with `value`.",
                ),
                mfn(
                    "regex",
                    "replace_all",
                    "regex.replace_all",
                    vec![
                        ParamSpec::req("haystack", "str"),
                        ParamSpec::req("pattern", "str"),
                        ParamSpec::req("value", "str"),
                    ],
                    "str",
                    None,
                    "Replaces all matches of `pattern` in `haystack` with `value`.",
                ),
            ],
            vec![],
        );

        // 18. registry
        add_module(
            "registry",
            "Windows Registry inspection and modification operations.",
            None,
            vec![
                mfn(
                    "registry",
                    "read",
                    "registry.read",
                    vec![ParamSpec::req("path", "str")],
                    "Dict[str, Any]",
                    None,
                    "Reads values under the specified Windows registry key `path`.",
                ),
                mfn(
                    "registry",
                    "write",
                    "registry.write",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::req("regname", "str"),
                        ParamSpec::req("regtype", "str"),
                        ParamSpec::req("regvalue", "str"),
                    ],
                    "bool",
                    None,
                    "Writes a value to the specified Windows registry key `path`.",
                ),
            ],
            vec![],
        );

        // 19. report
        add_module(
            "report",
            "Structured reporting of files, credentials, screenshots, and process lists to Tavern.",
            None,
            vec![
                mfn(
                    "report",
                    "file",
                    "report.file",
                    vec![ParamSpec::req("path", "str")],
                    "None",
                    None,
                    "Exfiltrates and reports the file at `path` to Tavern.",
                ),
                mfn(
                    "report",
                    "ntlm_hash",
                    "report.ntlm_hash",
                    vec![
                        ParamSpec::req("username", "str"),
                        ParamSpec::req("hash", "str"),
                    ],
                    "None",
                    None,
                    "Reports a captured NTLM hash credential to Tavern.",
                ),
                mfn(
                    "report",
                    "process_list",
                    "report.process_list",
                    vec![ParamSpec::req("list", "List[Dict]")],
                    "None",
                    None,
                    "Reports a structured process list to Tavern.",
                ),
                mfn(
                    "report",
                    "screenshot",
                    "report.screenshot",
                    vec![],
                    "None",
                    None,
                    "Captures and reports screenshots of active displays to Tavern.",
                ),
                mfn(
                    "report",
                    "ssh_key",
                    "report.ssh_key",
                    vec![
                        ParamSpec::req("username", "str"),
                        ParamSpec::req("key", "str"),
                    ],
                    "None",
                    None,
                    "Reports a captured SSH private key credential to Tavern.",
                ),
                mfn(
                    "report",
                    "user_password",
                    "report.user_password",
                    vec![
                        ParamSpec::req("username", "str"),
                        ParamSpec::req("password", "str"),
                    ],
                    "None",
                    None,
                    "Reports a captured plaintext username/password credential to Tavern.",
                ),
            ],
            vec![],
        );

        // 20. scan
        add_module(
            "scan",
            "Network discovery and port scanning operations (`arp`, `ports`).",
            None,
            vec![
                mfn(
                    "scan",
                    "arp",
                    "scan.arp",
                    vec![ParamSpec::req("target_cidrs", "List[str]")],
                    "List[Dict]",
                    None,
                    "Performs an ARP scan across `target_cidrs`.",
                ),
                mfn(
                    "scan",
                    "ports",
                    "scan.ports",
                    vec![
                        ParamSpec::req("target_cidrs", "List[str]"),
                        ParamSpec::req("ports", "List[int]"),
                        ParamSpec::req("protocol", "str"),
                        ParamSpec::req("timeout", "int"),
                        ParamSpec::opt("fd_limit", "Optional[int]", "None"),
                    ],
                    "List[Dict]",
                    None,
                    "Scans `ports` across `target_cidrs` using `protocol` (`tcp` or `udp`).",
                ),
            ],
            vec![],
        );

        // 21. ssh
        add_module(
            "ssh",
            "Remote command execution, file transfer, and agent deployment over SSH.",
            None,
            vec![
                mfn(
                    "ssh",
                    "copy",
                    "ssh.copy",
                    vec![
                        ParamSpec::req("target", "str"),
                        ParamSpec::req("port", "int"),
                        ParamSpec::req("src", "str"),
                        ParamSpec::req("dst", "str"),
                        ParamSpec::req("username", "str"),
                        ParamSpec::opt("password", "Optional[str]", "None"),
                        ParamSpec::opt("key", "Optional[str]", "None"),
                        ParamSpec::opt("key_password", "Optional[str]", "None"),
                        ParamSpec::opt("timeout", "Optional[int]", "None"),
                    ],
                    "str",
                    None,
                    "Copies local file `src` to remote `dst` over SSH/SFTP.",
                ),
                mfn(
                    "ssh",
                    "deploy",
                    "ssh.deploy",
                    vec![
                        ParamSpec::req("ips", "List[str]"),
                        ParamSpec::req("credentials", "List[Dict]"),
                        ParamSpec::req("cmd", "str"),
                        ParamSpec::opt("privesc_cmd", "Optional[str]", "None"),
                        ParamSpec::opt("payload", "Optional[str]", "None"),
                        ParamSpec::opt("payload_dst", "Optional[str]", "None"),
                        ParamSpec::opt("timeout", "Optional[int]", "None"),
                        ParamSpec::opt("retries", "Optional[int]", "None"),
                    ],
                    "List[Dict]",
                    None,
                    "Sprays SSH credentials across `ips` to deploy and execute a payload.",
                ),
                mfn(
                    "ssh",
                    "exec",
                    "ssh.exec",
                    vec![
                        ParamSpec::req("target", "str"),
                        ParamSpec::req("port", "int"),
                        ParamSpec::req("command", "str"),
                        ParamSpec::req("username", "str"),
                        ParamSpec::opt("password", "Optional[str]", "None"),
                        ParamSpec::opt("key", "Optional[str]", "None"),
                        ParamSpec::opt("key_password", "Optional[str]", "None"),
                        ParamSpec::opt("timeout", "Optional[int]", "None"),
                    ],
                    "Dict[str, Any]",
                    None,
                    "Executes `command` on `target:port` over SSH.",
                ),
            ],
            vec![],
        );

        // 22. sys
        add_module(
            "sys",
            "System OS detection, hostname lookup, and command/shell execution.",
            None,
            vec![
                mfn(
                    "sys",
                    "exec",
                    "sys.exec",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::req("args", "List[str]"),
                        ParamSpec::opt("disown", "Optional[bool]", "None"),
                        ParamSpec::opt("env_vars", "Optional[Dict[str, str]]", "None"),
                        ParamSpec::opt("input", "Optional[Bytes | str]", "None"),
                    ],
                    "Dict[str, Any]",
                    None,
                    "Executes binary `path` with `args` and returns `stdout`, `stderr`, and `status`.",
                ),
                mfn(
                    "sys",
                    "hostname",
                    "sys.hostname",
                    vec![],
                    "str",
                    None,
                    "Returns the system hostname.",
                ),
                mfn(
                    "sys",
                    "info",
                    "sys.info",
                    vec![],
                    "Dict[str, str]",
                    None,
                    "Returns OS architecture, distro, platform, and version information.",
                ),
                mfn(
                    "sys",
                    "is_bsd",
                    "sys.is_bsd",
                    vec![],
                    "bool",
                    None,
                    "Returns `True` if running on a BSD operating system.",
                ),
                mfn(
                    "sys",
                    "is_linux",
                    "sys.is_linux",
                    vec![],
                    "bool",
                    None,
                    "Returns `True` if running on Linux.",
                ),
                mfn(
                    "sys",
                    "is_macos",
                    "sys.is_macos",
                    vec![],
                    "bool",
                    None,
                    "Returns `True` if running on macOS.",
                ),
                mfn(
                    "sys",
                    "is_windows",
                    "sys.is_windows",
                    vec![],
                    "bool",
                    None,
                    "Returns `True` if running on Windows.",
                ),
                mfn(
                    "sys",
                    "shell",
                    "sys.shell",
                    vec![ParamSpec::req("cmd", "str")],
                    "Dict[str, Any]",
                    None,
                    "Executes `cmd` in the platform's default shell and returns `stdout`, `stderr`, and `status`.",
                ),
                // Deprecated on `sys` in 0.4.1
                mfn(
                    "sys",
                    "dll_inject",
                    "sys.dll_inject",
                    vec![
                        ParamSpec::req("dll_path", "str"),
                        ParamSpec::req("pid", "int"),
                    ],
                    "None",
                    Some("windows.dll_inject(dll_path, pid)"),
                    "Deprecated: use `windows.dll_inject(dll_path, pid)` instead.",
                ),
                mfn(
                    "sys",
                    "dll_reflect",
                    "sys.dll_reflect",
                    vec![
                        ParamSpec::req("dll_bytes", "Bytes | List[int]"),
                        ParamSpec::req("pid", "int"),
                        ParamSpec::req("function_name", "str"),
                    ],
                    "None",
                    Some("windows.dll_reflect(dll_bytes, pid, function_name)"),
                    "Deprecated: use `windows.dll_reflect(dll_bytes, pid, function_name)` instead.",
                ),
                mfn(
                    "sys",
                    "get_env",
                    "sys.get_env",
                    vec![],
                    "Dict[str, str]",
                    Some("process.env()"),
                    "Deprecated: use `process.env()` instead.",
                ),
                mfn(
                    "sys",
                    "get_ip",
                    "sys.get_ip",
                    vec![],
                    "List[Dict]",
                    Some("net.interfaces()"),
                    "Deprecated: use `net.interfaces()` instead.",
                ),
                mfn(
                    "sys",
                    "get_os",
                    "sys.get_os",
                    vec![],
                    "Dict[str, str]",
                    Some("sys.info()"),
                    "Deprecated: use `sys.info()` instead.",
                ),
                mfn(
                    "sys",
                    "get_pid",
                    "sys.get_pid",
                    vec![],
                    "int",
                    Some("process.pid()"),
                    "Deprecated: use `process.pid()` instead.",
                ),
                mfn(
                    "sys",
                    "get_reg",
                    "sys.get_reg",
                    vec![ParamSpec::req("path", "str")],
                    "Dict[str, Any]",
                    Some("registry.read(path)"),
                    "Deprecated: use `registry.read(path)` instead.",
                ),
                mfn(
                    "sys",
                    "get_user",
                    "sys.get_user",
                    vec![],
                    "Dict[str, Any]",
                    Some("user.info()"),
                    "Deprecated: use `user.info()` instead.",
                ),
                mfn(
                    "sys",
                    "impersonate",
                    "sys.impersonate",
                    vec![ParamSpec::req("pid", "int")],
                    "int",
                    Some("windows.impersonate(pid)"),
                    "Deprecated: use `windows.impersonate(pid)` instead.",
                ),
                mfn(
                    "sys",
                    "list_users",
                    "sys.list_users",
                    vec![],
                    "List[Dict]",
                    Some("user.list()"),
                    "Deprecated: use `user.list()` instead.",
                ),
                mfn(
                    "sys",
                    "tokens",
                    "sys.tokens",
                    vec![ParamSpec::opt("pid", "Optional[int]", "None")],
                    "List[Dict]",
                    Some("windows.tokens(pid)"),
                    "Deprecated: use `windows.tokens(pid)` instead.",
                ),
                mfn(
                    "sys",
                    "use_token",
                    "sys.use_token",
                    vec![ParamSpec::req("id", "int")],
                    "bool",
                    Some("windows.use_token(id)"),
                    "Deprecated: use `windows.use_token(id)` instead.",
                ),
                mfn(
                    "sys",
                    "whereis",
                    "sys.whereis",
                    vec![ParamSpec::req("bin", "str")],
                    "str",
                    Some("file.which(bin)"),
                    "Deprecated: use `file.which(bin)` instead.",
                ),
                mfn(
                    "sys",
                    "write_reg",
                    "sys.write_reg",
                    vec![
                        ParamSpec::req("path", "str"),
                        ParamSpec::req("regname", "str"),
                        ParamSpec::req("regtype", "str"),
                        ParamSpec::req("regvalue", "str"),
                    ],
                    "bool",
                    Some("registry.write(path, regname, regtype, regvalue)"),
                    "Deprecated: use `registry.write(path, regname, regtype, regvalue)` instead.",
                ),
            ],
            vec![],
        );

        // 23. template
        add_module(
            "template",
            "Jinja2 template rendering from strings and files.",
            None,
            vec![
                mfn(
                    "template",
                    "render",
                    "template.render",
                    vec![
                        ParamSpec::req("template", "str"),
                        ParamSpec::req("args", "Dict[str, Any]"),
                        ParamSpec::req("autoescape", "bool"),
                    ],
                    "str",
                    None,
                    "Renders a Jinja2 `template` string with `args`.",
                ),
                mfn(
                    "template",
                    "render_file",
                    "template.render_file",
                    vec![
                        ParamSpec::req("template_path", "str"),
                        ParamSpec::req("dst", "str"),
                        ParamSpec::req("args", "Dict[str, Any]"),
                        ParamSpec::req("autoescape", "bool"),
                    ],
                    "None",
                    None,
                    "Renders Jinja2 file `template_path` with `args` and writes the output to `dst`.",
                ),
            ],
            vec![],
        );

        // 24. time
        add_module(
            "time",
            "Time retrieval, formatting, parsing, and sleep functions.",
            None,
            vec![
                mfn(
                    "time",
                    "format",
                    "time.format",
                    vec![
                        ParamSpec::req("input", "int"),
                        ParamSpec::req("format", "str"),
                    ],
                    "str",
                    None,
                    "Formats an epoch timestamp `input` using `format`.",
                ),
                mfn(
                    "time",
                    "now",
                    "time.now",
                    vec![],
                    "int",
                    None,
                    "Returns the current UTC UNIX epoch timestamp in seconds.",
                ),
                mfn(
                    "time",
                    "sleep",
                    "time.sleep",
                    vec![ParamSpec::req("secs", "int | float")],
                    "None",
                    None,
                    "Pauses execution for `secs` seconds.",
                ),
                mfn(
                    "time",
                    "to_epoch",
                    "time.to_epoch",
                    vec![
                        ParamSpec::req("input", "str"),
                        ParamSpec::req("format", "str"),
                    ],
                    "int",
                    None,
                    "Parses timestamp string `input` according to `format` and returns UNIX epoch seconds.",
                ),
                // Deprecated on `time` in 0.4.1
                mfn(
                    "time",
                    "format_to_epoch",
                    "time.format_to_epoch",
                    vec![
                        ParamSpec::req("input", "str"),
                        ParamSpec::req("format", "str"),
                    ],
                    "int",
                    Some("time.to_epoch(input, format)"),
                    "Deprecated: use `time.to_epoch(input, format)` instead.",
                ),
                mfn(
                    "time",
                    "format_to_readable",
                    "time.format_to_readable",
                    vec![
                        ParamSpec::req("input", "int"),
                        ParamSpec::req("format", "str"),
                    ],
                    "str",
                    Some("time.format(input, format)"),
                    "Deprecated: use `time.format(input, format)` instead.",
                ),
            ],
            vec![],
        );

        // 25. user
        add_module(
            "user",
            "User account enumeration and current user identity inspection.",
            None,
            vec![
                mfn(
                    "user",
                    "info",
                    "user.info",
                    vec![],
                    "Dict[str, Any]",
                    None,
                    "Returns identity information for the current process user.",
                ),
                mfn(
                    "user",
                    "list",
                    "user.list",
                    vec![],
                    "List[Dict]",
                    None,
                    "Lists all user accounts on the system.",
                ),
            ],
            vec![],
        );

        // 26. windows
        add_module(
            "windows",
            "Windows-specific operations including DLL injection, reflective loading, and token manipulation.",
            None,
            vec![
                mfn(
                    "windows",
                    "dll_inject",
                    "windows.dll_inject",
                    vec![
                        ParamSpec::req("dll_path", "str"),
                        ParamSpec::req("pid", "int"),
                    ],
                    "None",
                    None,
                    "Injects a DLL on disk at `dll_path` into process `pid`.",
                ),
                mfn(
                    "windows",
                    "dll_reflect",
                    "windows.dll_reflect",
                    vec![
                        ParamSpec::req("dll_bytes", "Bytes | List[int]"),
                        ParamSpec::req("pid", "int"),
                        ParamSpec::req("function_name", "str"),
                    ],
                    "None",
                    None,
                    "Reflectively loads `dll_bytes` in memory into process `pid` calling `function_name`.",
                ),
                mfn(
                    "windows",
                    "impersonate",
                    "windows.impersonate",
                    vec![ParamSpec::req("pid", "int")],
                    "int",
                    None,
                    "Steals and activates a process token from `pid`, returning the token store ID.",
                ),
                mfn(
                    "windows",
                    "tokens",
                    "windows.tokens",
                    vec![ParamSpec::opt("pid", "Optional[int]", "None")],
                    "List[Dict]",
                    None,
                    "Lists stored tokens (when `pid` is `None`) or token privileges for `pid`.",
                ),
                mfn(
                    "windows",
                    "use_token",
                    "windows.use_token",
                    vec![ParamSpec::req("id", "int")],
                    "bool",
                    None,
                    "Activates a stored token by `id` (`0` reverts to process token).",
                ),
            ],
            vec![],
        );

        // 27. Deprecated `crypto` module
        add_module(
            "crypto",
            "Deprecated compatibility module. Use `aes`, `hash`, `base64`, `json`, or `.encode()`/`.decode()` instead.",
            Some("Use `aes`, `hash`, `base64`, `json`, or `.encode()`/`.decode()` instead."),
            vec![
                mfn(
                    "crypto",
                    "aes_decrypt",
                    "crypto.aes_decrypt",
                    vec![
                        ParamSpec::req("key", "Bytes | str"),
                        ParamSpec::req("iv", "Bytes | str"),
                        ParamSpec::req("data", "Bytes"),
                    ],
                    "Bytes",
                    Some("aes.decrypt(key, iv, data)"),
                    "Deprecated: use `aes.decrypt(key, iv, data)` instead.",
                ),
                mfn(
                    "crypto",
                    "aes_decrypt_file",
                    "crypto.aes_decrypt_file",
                    vec![
                        ParamSpec::req("src", "str"),
                        ParamSpec::req("dst", "str"),
                        ParamSpec::req("key", "str"),
                    ],
                    "None",
                    Some("aes.decrypt_file(src, dst, key)"),
                    "Deprecated: use `aes.decrypt_file(src, dst, key)` instead.",
                ),
                mfn(
                    "crypto",
                    "aes_encrypt",
                    "crypto.aes_encrypt",
                    vec![
                        ParamSpec::req("key", "Bytes | str"),
                        ParamSpec::req("iv", "Bytes | str"),
                        ParamSpec::req("data", "Bytes | str"),
                    ],
                    "Bytes",
                    Some("aes.encrypt(key, iv, data)"),
                    "Deprecated: use `aes.encrypt(key, iv, data)` instead.",
                ),
                mfn(
                    "crypto",
                    "aes_encrypt_file",
                    "crypto.aes_encrypt_file",
                    vec![
                        ParamSpec::req("src", "str"),
                        ParamSpec::req("dst", "str"),
                        ParamSpec::req("key", "str"),
                    ],
                    "None",
                    Some("aes.encrypt_file(src, dst, key)"),
                    "Deprecated: use `aes.encrypt_file(src, dst, key)` instead.",
                ),
                mfn(
                    "crypto",
                    "decode_b64",
                    "crypto.decode_b64",
                    vec![
                        ParamSpec::req("content", "str"),
                        ParamSpec::opt("encode_type", "Optional[str]", "None"),
                    ],
                    "Bytes",
                    Some("base64.decode(content, encode_type)"),
                    "Deprecated: use `base64.decode(content, encode_type)` instead.",
                ),
                mfn(
                    "crypto",
                    "decode_utf16le",
                    "crypto.decode_utf16le",
                    vec![ParamSpec::req("data", "Bytes")],
                    "str",
                    Some("data.decode(\"utf16le\")"),
                    "Deprecated: use `bytes.decode(\"utf16le\")` instead.",
                ),
                mfn(
                    "crypto",
                    "encode_b64",
                    "crypto.encode_b64",
                    vec![
                        ParamSpec::req("content", "Bytes | str"),
                        ParamSpec::opt("encode_type", "Optional[str]", "None"),
                    ],
                    "str",
                    Some("base64.encode(content, encode_type)"),
                    "Deprecated: use `base64.encode(content, encode_type)` instead.",
                ),
                mfn(
                    "crypto",
                    "encode_utf16le",
                    "crypto.encode_utf16le",
                    vec![ParamSpec::req("data", "str")],
                    "Bytes",
                    Some("data.encode(\"utf16le\")"),
                    "Deprecated: use `str.encode(\"utf16le\")` instead.",
                ),
                mfn(
                    "crypto",
                    "from_json",
                    "crypto.from_json",
                    vec![ParamSpec::req("content", "Bytes | str")],
                    "Any",
                    Some("json.decode(content)"),
                    "Deprecated: use `json.decode(content)` instead.",
                ),
                mfn(
                    "crypto",
                    "hash_file",
                    "crypto.hash_file",
                    vec![ParamSpec::req("file", "str"), ParamSpec::req("algo", "str")],
                    "str",
                    Some("hash.file(file, algo)"),
                    "Deprecated: use `hash.file(file, algo)` instead.",
                ),
                mfn(
                    "crypto",
                    "is_json",
                    "crypto.is_json",
                    vec![ParamSpec::req("content", "Bytes | str")],
                    "bool",
                    Some("json.is_valid(content)"),
                    "Deprecated: use `json.is_valid(content)` instead.",
                ),
                mfn(
                    "crypto",
                    "md5",
                    "crypto.md5",
                    vec![ParamSpec::req("data", "Bytes | str")],
                    "str",
                    Some("hash.md5(data)"),
                    "Deprecated: use `hash.md5(data)` instead.",
                ),
                mfn(
                    "crypto",
                    "sha1",
                    "crypto.sha1",
                    vec![ParamSpec::req("data", "Bytes | str")],
                    "str",
                    Some("hash.sha1(data)"),
                    "Deprecated: use `hash.sha1(data)` instead.",
                ),
                mfn(
                    "crypto",
                    "sha256",
                    "crypto.sha256",
                    vec![ParamSpec::req("data", "Bytes | str")],
                    "str",
                    Some("hash.sha256(data)"),
                    "Deprecated: use `hash.sha256(data)` instead.",
                ),
                mfn(
                    "crypto",
                    "to_json",
                    "crypto.to_json",
                    vec![ParamSpec::req("content", "Any")],
                    "str",
                    Some("json.encode(content)"),
                    "Deprecated: use `json.encode(content)` instead.",
                ),
            ],
            vec![],
        );

        // 28. Deprecated `pivot` module
        add_module(
            "pivot",
            "Deprecated compatibility module. Use `scan`, `portal`, `net`, or `ssh` instead.",
            Some("Use `scan`, `portal`, `net`, or `ssh` instead."),
            vec![
                mfn(
                    "pivot",
                    "arp_scan",
                    "pivot.arp_scan",
                    vec![ParamSpec::req("target_cidrs", "List[str]")],
                    "List[Dict]",
                    Some("scan.arp(target_cidrs)"),
                    "Deprecated: use `scan.arp(target_cidrs)` instead.",
                ),
                mfn(
                    "pivot",
                    "create_portal",
                    "pivot.create_portal",
                    vec![],
                    "None",
                    Some("portal.create()"),
                    "Deprecated: use `portal.create()` instead.",
                ),
                mfn(
                    "pivot",
                    "ncat",
                    "pivot.ncat",
                    vec![
                        ParamSpec::req("address", "str"),
                        ParamSpec::req("port", "int"),
                        ParamSpec::req("data", "Bytes | str"),
                        ParamSpec::req("protocol", "str"),
                    ],
                    "str",
                    Some("net.tcp(address, port, data) or net.udp(address, port, data)"),
                    "Deprecated: use `net.tcp(...)` or `net.udp(...)` instead.",
                ),
                mfn(
                    "pivot",
                    "port_scan",
                    "pivot.port_scan",
                    vec![
                        ParamSpec::req("target_cidrs", "List[str]"),
                        ParamSpec::req("ports", "List[int]"),
                        ParamSpec::req("protocol", "str"),
                        ParamSpec::req("timeout", "int"),
                        ParamSpec::opt("fd_limit", "Optional[int]", "None"),
                    ],
                    "List[Dict]",
                    Some("scan.ports(target_cidrs, ports, protocol, timeout, fd_limit)"),
                    "Deprecated: use `scan.ports(...)` instead.",
                ),
                mfn(
                    "pivot",
                    "ssh_copy",
                    "pivot.ssh_copy",
                    vec![
                        ParamSpec::req("target", "str"),
                        ParamSpec::req("port", "int"),
                        ParamSpec::req("src", "str"),
                        ParamSpec::req("dst", "str"),
                        ParamSpec::req("username", "str"),
                        ParamSpec::opt("password", "Optional[str]", "None"),
                        ParamSpec::opt("key", "Optional[str]", "None"),
                        ParamSpec::opt("key_password", "Optional[str]", "None"),
                        ParamSpec::opt("timeout", "Optional[int]", "None"),
                    ],
                    "str",
                    Some("ssh.copy(...)"),
                    "Deprecated: use `ssh.copy(...)` instead.",
                ),
                mfn(
                    "pivot",
                    "ssh_deploy",
                    "pivot.ssh_deploy",
                    vec![
                        ParamSpec::req("ips", "List[str]"),
                        ParamSpec::req("credentials", "List[Dict]"),
                        ParamSpec::req("cmd", "str"),
                        ParamSpec::opt("privesc_cmd", "Optional[str]", "None"),
                        ParamSpec::opt("payload", "Optional[str]", "None"),
                        ParamSpec::opt("payload_dst", "Optional[str]", "None"),
                        ParamSpec::opt("timeout", "Optional[int]", "None"),
                        ParamSpec::opt("retries", "Optional[int]", "None"),
                    ],
                    "List[Dict]",
                    Some("ssh.deploy(...)"),
                    "Deprecated: use `ssh.deploy(...)` instead.",
                ),
                mfn(
                    "pivot",
                    "ssh_exec",
                    "pivot.ssh_exec",
                    vec![
                        ParamSpec::req("target", "str"),
                        ParamSpec::req("port", "int"),
                        ParamSpec::req("command", "str"),
                        ParamSpec::req("username", "str"),
                        ParamSpec::opt("password", "Optional[str]", "None"),
                        ParamSpec::opt("key", "Optional[str]", "None"),
                        ParamSpec::opt("key_password", "Optional[str]", "None"),
                        ParamSpec::opt("timeout", "Optional[int]", "None"),
                    ],
                    "Dict[str, Any]",
                    Some("ssh.exec(...)"),
                    "Deprecated: use `ssh.exec(...)` instead.",
                ),
            ],
            vec![
                (
                    "smb_exec",
                    "unimplemented proposal removed in Eldritch 0.4.1",
                ),
                (
                    "bind_proxy",
                    "unimplemented proposal removed in Eldritch 0.4.1; use portal.create()",
                ),
                (
                    "port_forward",
                    "unimplemented proposal removed in Eldritch 0.4.1; use portal.create()",
                ),
            ],
        );

        // Builtin functions (all 39 in eldritch-core)
        let mut builtins = BTreeMap::new();
        let mut add_builtin = |name: &'static str,
                               params: Vec<ParamSpec>,
                               return_type: &'static str,
                               varargs: bool,
                               kwargs: bool,
                               fallback_desc: &str| {
            builtins.insert(
                name.to_string(),
                FuncSpec {
                    name,
                    qualified_name: name,
                    module: None,
                    params,
                    return_type,
                    varargs,
                    kwargs,
                    deprecated_replacement: None,
                    description: doc_for(name, fallback_desc),
                },
            );
        };

        add_builtin(
            "abs",
            vec![ParamSpec::req("x", "int | float")],
            "int | float",
            false,
            false,
            "Returns the absolute value of a number.",
        );
        add_builtin(
            "all",
            vec![ParamSpec::req("iterable", "Iterable")],
            "bool",
            false,
            false,
            "Returns `True` if all elements of `iterable` are truthy.",
        );
        add_builtin(
            "any",
            vec![ParamSpec::req("iterable", "Iterable")],
            "bool",
            false,
            false,
            "Returns `True` if any element of `iterable` is truthy.",
        );
        add_builtin(
            "assert",
            vec![
                ParamSpec::req("condition", "Any"),
                ParamSpec::opt("message", "Optional[str]", "None"),
            ],
            "None",
            false,
            false,
            "Aborts execution if `condition` is falsy.",
        );
        add_builtin(
            "assert_eq",
            vec![ParamSpec::req("a", "Any"), ParamSpec::req("b", "Any")],
            "None",
            false,
            false,
            "Aborts execution if `a` is not equal to `b`.",
        );
        add_builtin(
            "bool",
            vec![ParamSpec::opt("x", "Any", "False")],
            "bool",
            false,
            false,
            "Converts a value to a boolean.",
        );
        add_builtin(
            "builtins",
            vec![],
            "List[str]",
            false,
            false,
            "Returns a list of all built-in function names.",
        );
        add_builtin(
            "bytes",
            vec![ParamSpec::opt("source", "Any", "b\"\"")],
            "Bytes",
            false,
            false,
            "Creates a `Bytes` object from an integer length, UTF-8 string, or iterable of byte ints.",
        );
        add_builtin(
            "chr",
            vec![ParamSpec::req("i", "int")],
            "str",
            false,
            false,
            "Returns the single-character string for Unicode code point `i`.",
        );
        add_builtin(
            "dict",
            vec![ParamSpec::opt("iterable", "Optional[Iterable]", "None")],
            "Dict",
            false,
            true,
            "Creates a new dictionary from an optional iterable and/or keyword arguments.",
        );
        add_builtin(
            "dir",
            vec![ParamSpec::opt("object", "Optional[Any]", "None")],
            "List[str]",
            false,
            false,
            "Returns a list of attributes/methods for `object`, or names in the current scope.",
        );
        add_builtin(
            "enumerate",
            vec![
                ParamSpec::req("iterable", "Iterable"),
                ParamSpec::opt("start", "int", "0"),
            ],
            "List[Tuple[int, Any]]",
            false,
            false,
            "Returns a list of `(index, item)` tuples for `iterable`.",
        );
        add_builtin(
            "eprint",
            vec![],
            "None",
            true,
            false,
            "Prints arguments to standard error (`stderr`), separated by spaces.",
        );
        add_builtin(
            "eval",
            vec![ParamSpec::req("source", "str")],
            "Any",
            false,
            false,
            "Evaluates an Eldritch code string in the current interpreter context.",
        );
        add_builtin(
            "fail",
            vec![ParamSpec::req("message", "str")],
            "None",
            false,
            false,
            "Aborts execution immediately with `message`.",
        );
        add_builtin(
            "filter",
            vec![
                ParamSpec::req("function", "Optional[Callable]"),
                ParamSpec::req("iterable", "Iterable"),
            ],
            "List",
            false,
            false,
            "Filters elements of `iterable` for which `function(item)` is truthy.",
        );
        add_builtin(
            "float",
            vec![ParamSpec::opt("x", "int | float | str", "0.0")],
            "float",
            false,
            false,
            "Converts a number or string to a floating-point number.",
        );
        add_builtin(
            "hex",
            vec![ParamSpec::req("x", "int")],
            "str",
            false,
            false,
            "Converts an integer to a lowercase hexadecimal string prefixed with `0x`.",
        );
        add_builtin(
            "int",
            vec![
                ParamSpec::opt("x", "int | float | str", "0"),
                ParamSpec::opt("base", "int", "10"),
            ],
            "int",
            false,
            false,
            "Converts a number or string to an integer.",
        );
        add_builtin(
            "len",
            vec![ParamSpec::req("s", "Sized")],
            "int",
            false,
            false,
            "Returns the number of items in a sequence or collection.",
        );
        add_builtin(
            "libs",
            vec![],
            "List[str]",
            false,
            false,
            "Returns a list of all registered standard library names.",
        );
        add_builtin(
            "list",
            vec![ParamSpec::opt("iterable", "Optional[Iterable]", "None")],
            "List",
            false,
            false,
            "Creates a list from an optional iterable.",
        );
        add_builtin(
            "map",
            vec![
                ParamSpec::req("function", "Callable"),
                ParamSpec::req("iterable", "Iterable"),
            ],
            "List",
            false,
            false,
            "Applies `function` to every item of `iterable` and returns a list of results.",
        );
        add_builtin(
            "max",
            vec![ParamSpec::req("iterable", "Any")],
            "Any",
            true,
            false,
            "Returns the largest item in an iterable or the largest of two or more arguments.",
        );
        add_builtin(
            "min",
            vec![ParamSpec::req("iterable", "Any")],
            "Any",
            true,
            false,
            "Returns the smallest item in an iterable or the smallest of two or more arguments.",
        );
        add_builtin(
            "ord",
            vec![ParamSpec::req("c", "str | Bytes")],
            "int",
            false,
            false,
            "Returns the Unicode code point of a single-character string or byte.",
        );
        add_builtin(
            "pprint",
            vec![
                ParamSpec::req("object", "Any"),
                ParamSpec::opt("indent", "int", "2"),
            ],
            "None",
            false,
            false,
            "Pretty-prints an object with indentation.",
        );
        add_builtin(
            "print",
            vec![],
            "None",
            true,
            false,
            "Prints arguments to standard output (`stdout`), separated by spaces.",
        );
        add_builtin(
            "range",
            vec![
                ParamSpec::req("start_or_stop", "int"),
                ParamSpec::opt("stop", "Optional[int]", "None"),
                ParamSpec::opt("step", "int", "1"),
            ],
            "List[int]",
            false,
            false,
            "Returns a list of integers in an arithmetic progression.",
        );
        add_builtin(
            "reduce",
            vec![
                ParamSpec::req("function", "Callable"),
                ParamSpec::req("iterable", "Iterable"),
                ParamSpec::opt("initializer", "Optional[Any]", "None"),
            ],
            "Any",
            false,
            false,
            "Applies a two-argument `function` cumulatively to the items of `iterable`.",
        );
        add_builtin(
            "repr",
            vec![ParamSpec::req("object", "Any")],
            "str",
            false,
            false,
            "Returns a printable representation of `object`.",
        );
        add_builtin(
            "reversed",
            vec![ParamSpec::req("seq", "Sequence")],
            "List",
            false,
            false,
            "Returns a new list with the elements of `seq` in reverse order.",
        );
        add_builtin(
            "set",
            vec![ParamSpec::opt("iterable", "Optional[Iterable]", "None")],
            "Set",
            false,
            false,
            "Creates a new set object.",
        );
        add_builtin(
            "sorted",
            vec![
                ParamSpec::req("iterable", "Iterable"),
                ParamSpec::opt("key", "Optional[Callable]", "None"),
                ParamSpec::opt("reverse", "bool", "False"),
            ],
            "List",
            false,
            true,
            "Returns a new sorted list from the items in `iterable`.",
        );
        add_builtin(
            "str",
            vec![ParamSpec::opt("object", "Any", "\"\"")],
            "str",
            false,
            false,
            "Converts `object` to a string (UTF-8 decodes `Bytes`).",
        );
        add_builtin(
            "tprint",
            vec![ParamSpec::req("list_of_dicts", "List[Dict]")],
            "None",
            false,
            false,
            "Prints a list of dictionaries formatted as a table.",
        );
        add_builtin(
            "tuple",
            vec![ParamSpec::opt("iterable", "Optional[Iterable]", "None")],
            "Tuple",
            false,
            false,
            "Creates a tuple from an optional iterable.",
        );
        add_builtin(
            "type",
            vec![ParamSpec::req("object", "Any")],
            "str",
            false,
            false,
            "Returns the type name of `object` (`\"int\"`, `\"string\"`, `\"list\"`, `\"dict\"`, etc.).",
        );
        add_builtin(
            "zip",
            vec![],
            "List[Tuple]",
            true,
            false,
            "Aggregates elements from each of the iterables into tuples.",
        );

        // Built-in value methods (`str`, `bytes`, `list`, `dict`, `set`, `tuple`)
        let mut methods: BTreeMap<String, Vec<MethodSpec>> = BTreeMap::new();
        let mut add_method = |receiver_type: &'static str,
                              name: &'static str,
                              params: Vec<ParamSpec>,
                              return_type: &'static str,
                              description: &'static str| {
            methods
                .entry(name.to_string())
                .or_default()
                .push(MethodSpec {
                    receiver_type,
                    name,
                    params,
                    return_type,
                    description,
                });
        };

        // str methods
        add_method(
            "str",
            "capitalize",
            vec![],
            "str",
            "Returns a copy of the string with its first character capitalized and the rest lowercased.",
        );
        add_method(
            "str",
            "codepoints",
            vec![],
            "List[int]",
            "Returns a list of Unicode codepoint integers for each character in the string.",
        );
        add_method(
            "str",
            "count",
            vec![
                ParamSpec::req("sub", "str"),
                ParamSpec::opt("start", "int", "0"),
                ParamSpec::opt("end", "Optional[int]", "None"),
            ],
            "int",
            "Returns the number of non-overlapping occurrences of substring `sub`.",
        );
        add_method(
            "str",
            "elems",
            vec![],
            "List[str]",
            "Returns a list of single-character strings.",
        );
        add_method(
            "str",
            "encode",
            vec![ParamSpec::opt("encoding", "str", "\"utf8\"")],
            "Bytes",
            "Encodes the string into `Bytes` using `\"utf8\"` (default) or `\"utf16le\"`.",
        );
        add_method(
            "str",
            "endswith",
            vec![ParamSpec::req("suffix", "str | Tuple[str, ...]")],
            "bool",
            "Returns `True` if the string ends with the specified `suffix`.",
        );
        add_method(
            "str",
            "find",
            vec![
                ParamSpec::req("sub", "str"),
                ParamSpec::opt("start", "int", "0"),
                ParamSpec::opt("end", "Optional[int]", "None"),
            ],
            "int",
            "Returns the lowest index where substring `sub` is found, or `-1` if not found.",
        );
        add_method(
            "str",
            "format",
            vec![ParamSpec::opt("*args", "Any", "")],
            "str",
            "Performs string formatting using `{}` placeholders.",
        );
        add_method(
            "str",
            "index",
            vec![
                ParamSpec::req("sub", "str"),
                ParamSpec::opt("start", "int", "0"),
                ParamSpec::opt("end", "Optional[int]", "None"),
            ],
            "int",
            "Like `find()`, but raises an error when the substring is not found.",
        );
        add_method(
            "str",
            "isalnum",
            vec![],
            "bool",
            "Returns `True` if all characters are alphanumeric and the string is non-empty.",
        );
        add_method(
            "str",
            "isalpha",
            vec![],
            "bool",
            "Returns `True` if all characters are alphabetic and the string is non-empty.",
        );
        add_method(
            "str",
            "isdigit",
            vec![],
            "bool",
            "Returns `True` if all characters are digits and the string is non-empty.",
        );
        add_method(
            "str",
            "islower",
            vec![],
            "bool",
            "Returns `True` if all cased characters are lowercase.",
        );
        add_method(
            "str",
            "isspace",
            vec![],
            "bool",
            "Returns `True` if all characters are whitespace and the string is non-empty.",
        );
        add_method(
            "str",
            "istitle",
            vec![],
            "bool",
            "Returns `True` if the string is titlecased.",
        );
        add_method(
            "str",
            "isupper",
            vec![],
            "bool",
            "Returns `True` if all cased characters are uppercase.",
        );
        add_method(
            "str",
            "join",
            vec![ParamSpec::req("iterable", "Iterable[str]")],
            "str",
            "Concatenates strings in `iterable` using this string as separator.",
        );
        add_method(
            "str",
            "lower",
            vec![],
            "str",
            "Returns a copy of the string converted to lowercase.",
        );
        add_method(
            "str",
            "lstrip",
            vec![ParamSpec::opt("chars", "Optional[str]", "None")],
            "str",
            "Returns a copy of the string with leading characters removed.",
        );
        add_method(
            "str",
            "partition",
            vec![ParamSpec::req("sep", "str")],
            "Tuple[str, str, str]",
            "Splits the string at the first occurrence of `sep` into a 3-tuple.",
        );
        add_method(
            "str",
            "removeprefix",
            vec![ParamSpec::req("prefix", "str")],
            "str",
            "If the string starts with `prefix`, returns `string[len(prefix):]`.",
        );
        add_method(
            "str",
            "removesuffix",
            vec![ParamSpec::req("suffix", "str")],
            "str",
            "If the string ends with `suffix`, returns `string[:-len(suffix)]`.",
        );
        add_method(
            "str",
            "replace",
            vec![
                ParamSpec::req("old", "str"),
                ParamSpec::req("new", "str"),
                ParamSpec::opt("count", "Optional[int]", "None"),
            ],
            "str",
            "Returns a copy of the string with occurrences of `old` replaced by `new`.",
        );
        add_method(
            "str",
            "rfind",
            vec![
                ParamSpec::req("sub", "str"),
                ParamSpec::opt("start", "int", "0"),
                ParamSpec::opt("end", "Optional[int]", "None"),
            ],
            "int",
            "Returns the highest index where substring `sub` is found, or `-1`.",
        );
        add_method(
            "str",
            "rindex",
            vec![
                ParamSpec::req("sub", "str"),
                ParamSpec::opt("start", "int", "0"),
                ParamSpec::opt("end", "Optional[int]", "None"),
            ],
            "int",
            "Like `rfind()`, but raises an error when the substring is not found.",
        );
        add_method(
            "str",
            "rpartition",
            vec![ParamSpec::req("sep", "str")],
            "Tuple[str, str, str]",
            "Splits the string at the last occurrence of `sep` into a 3-tuple.",
        );
        add_method(
            "str",
            "rsplit",
            vec![
                ParamSpec::opt("sep", "Optional[str]", "None"),
                ParamSpec::opt("maxsplit", "Optional[int]", "None"),
            ],
            "List[str]",
            "Splits the string from the right at `sep`.",
        );
        add_method(
            "str",
            "rstrip",
            vec![ParamSpec::opt("chars", "Optional[str]", "None")],
            "str",
            "Returns a copy of the string with trailing characters removed.",
        );
        add_method(
            "str",
            "split",
            vec![
                ParamSpec::opt("sep", "Optional[str]", "None"),
                ParamSpec::opt("maxsplit", "Optional[int]", "None"),
            ],
            "List[str]",
            "Splits the string into a list of substrings using `sep` as delimiter.",
        );
        add_method(
            "str",
            "splitlines",
            vec![ParamSpec::opt("keepends", "bool", "False")],
            "List[str]",
            "Returns a list of the lines in the string, breaking at line boundaries.",
        );
        add_method(
            "str",
            "startswith",
            vec![ParamSpec::req("prefix", "str | Tuple[str, ...]")],
            "bool",
            "Returns `True` if the string starts with the specified `prefix`.",
        );
        add_method(
            "str",
            "strip",
            vec![ParamSpec::opt("chars", "Optional[str]", "None")],
            "str",
            "Returns a copy of the string with leading and trailing characters removed.",
        );
        add_method(
            "str",
            "title",
            vec![],
            "str",
            "Returns a titlecased version of the string.",
        );
        add_method(
            "str",
            "upper",
            vec![],
            "str",
            "Returns a copy of the string converted to uppercase.",
        );

        // bytes methods
        add_method(
            "Bytes",
            "decode",
            vec![ParamSpec::opt("encoding", "str", "\"utf8\"")],
            "str",
            "Decodes `Bytes` into a string using `\"utf8\"` (default) or `\"utf16le\"`.",
        );

        // list methods
        add_method(
            "List",
            "append",
            vec![ParamSpec::req("x", "Any")],
            "None",
            "Appends `x` to the end of the list.",
        );
        add_method(
            "List",
            "clear",
            vec![],
            "None",
            "Removes all items from the list.",
        );
        add_method(
            "List",
            "count",
            vec![ParamSpec::req("x", "Any")],
            "int",
            "Returns the number of times `x` appears in the list.",
        );
        add_method(
            "List",
            "extend",
            vec![ParamSpec::req("iterable", "Iterable")],
            "None",
            "Extends the list by appending all items from `iterable`.",
        );
        add_method(
            "List",
            "index",
            vec![
                ParamSpec::req("x", "Any"),
                ParamSpec::opt("start", "int", "0"),
                ParamSpec::opt("end", "Optional[int]", "None"),
            ],
            "int",
            "Returns the zero-based index of the first item equal to `x`.",
        );
        add_method(
            "List",
            "insert",
            vec![ParamSpec::req("i", "int"), ParamSpec::req("x", "Any")],
            "None",
            "Inserts `x` at index `i`.",
        );
        add_method(
            "List",
            "pop",
            vec![ParamSpec::opt("i", "int", "-1")],
            "Any",
            "Removes and returns the item at index `i` (default last).",
        );
        add_method(
            "List",
            "remove",
            vec![ParamSpec::req("x", "Any")],
            "None",
            "Removes the first item from the list whose value is equal to `x`.",
        );
        add_method(
            "List",
            "sort",
            vec![],
            "None",
            "Sorts the items of the list in place.",
        );

        // dict methods
        add_method(
            "Dict",
            "clear",
            vec![],
            "None",
            "Removes all items from the dictionary.",
        );
        add_method(
            "Dict",
            "get",
            vec![
                ParamSpec::req("key", "Any"),
                ParamSpec::opt("default", "Any", "None"),
            ],
            "Any",
            "Returns the value for `key` if `key` is in the dictionary, else `default`.",
        );
        add_method(
            "Dict",
            "items",
            vec![],
            "List[Tuple[Any, Any]]",
            "Returns a list of `(key, value)` pairs in the dictionary.",
        );
        add_method(
            "Dict",
            "keys",
            vec![],
            "List[Any]",
            "Returns a list of all keys in the dictionary.",
        );
        add_method(
            "Dict",
            "pop",
            vec![
                ParamSpec::req("key", "Any"),
                ParamSpec::opt("default", "Optional[Any]", "None"),
            ],
            "Any",
            "Removes `key` and returns its value, or `default` if not present.",
        );
        add_method(
            "Dict",
            "popitem",
            vec![],
            "Tuple[Any, Any]",
            "Removes and returns a `(key, value)` pair from the dictionary.",
        );
        add_method(
            "Dict",
            "setdefault",
            vec![
                ParamSpec::req("key", "Any"),
                ParamSpec::opt("default", "Any", "None"),
            ],
            "Any",
            "If `key` is in the dictionary, returns its value; otherwise inserts `key` with `default`.",
        );
        add_method(
            "Dict",
            "update",
            vec![ParamSpec::req("other", "Dict | Iterable")],
            "None",
            "Updates the dictionary with key/value pairs from `other`.",
        );
        add_method(
            "Dict",
            "values",
            vec![],
            "List[Any]",
            "Returns a list of all values in the dictionary.",
        );

        // set methods
        add_method(
            "Set",
            "add",
            vec![ParamSpec::req("elem", "Any")],
            "None",
            "Adds `elem` to the set.",
        );
        add_method(
            "Set",
            "clear",
            vec![],
            "None",
            "Removes all elements from the set.",
        );
        add_method(
            "Set",
            "difference",
            vec![ParamSpec::req("other", "Set | Iterable")],
            "Set",
            "Returns a new set with elements in the set that are not in `other`.",
        );
        add_method(
            "Set",
            "discard",
            vec![ParamSpec::req("elem", "Any")],
            "None",
            "Removes `elem` from the set if it is present.",
        );
        add_method(
            "Set",
            "intersection",
            vec![ParamSpec::req("other", "Set | Iterable")],
            "Set",
            "Returns a new set with elements common to the set and `other`.",
        );
        add_method(
            "Set",
            "isdisjoint",
            vec![ParamSpec::req("other", "Set | Iterable")],
            "bool",
            "Returns `True` if the set has no elements in common with `other`.",
        );
        add_method(
            "Set",
            "issubset",
            vec![ParamSpec::req("other", "Set | Iterable")],
            "bool",
            "Returns `True` if every element in the set is in `other`.",
        );
        add_method(
            "Set",
            "issuperset",
            vec![ParamSpec::req("other", "Set | Iterable")],
            "bool",
            "Returns `True` if every element in `other` is in the set.",
        );
        add_method(
            "Set",
            "pop",
            vec![],
            "Any",
            "Removes and returns an arbitrary element from the set.",
        );
        add_method(
            "Set",
            "remove",
            vec![ParamSpec::req("elem", "Any")],
            "None",
            "Removes `elem` from the set; raises an error if not present.",
        );
        add_method(
            "Set",
            "symmetric_difference",
            vec![ParamSpec::req("other", "Set | Iterable")],
            "Set",
            "Returns a new set with elements in either the set or `other`, but not both.",
        );
        add_method(
            "Set",
            "union",
            vec![ParamSpec::req("other", "Set | Iterable")],
            "Set",
            "Returns a new set with elements from the set and `other`.",
        );
        add_method(
            "Set",
            "update",
            vec![ParamSpec::req("other", "Set | Iterable")],
            "None",
            "Updates the set, adding elements from `other`.",
        );

        Self {
            modules,
            builtins,
            methods,
        }
    }

    pub fn is_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    pub fn is_builtin(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
    }

    pub fn get_module(&self, name: &str) -> Option<&ModuleSpec> {
        self.modules.get(name)
    }

    pub fn get_module_func(&self, module: &str, func: &str) -> Option<&FuncSpec> {
        self.modules.get(module)?.functions.get(func)
    }

    pub fn get_builtin(&self, name: &str) -> Option<&FuncSpec> {
        self.builtins.get(name)
    }
}
