use regex::Regex;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

struct MethodDoc {
    name: String,
    signature: String,
    docs: String,
}

struct LibraryDoc {
    name: String, // "file", "agent", etc.
    trait_name: String, // "FileLibrary"
    docs: String,
    methods: Vec<MethodDoc>,
}

fn main() {
    println!("cargo:rerun-if-changed=../stdlib");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let stdlib_path = Path::new(&manifest_dir).join("../stdlib");
    let docs_path = Path::new(&manifest_dir).join("../../../../docs/_docs/user-guide/eldritch-stdlib.md");

    let libraries = parse_libraries(&stdlib_path);
    generate_markdown(&libraries, &docs_path);
}

fn parse_libraries(root: &Path) -> Vec<LibraryDoc> {
    let mut libraries = Vec::new();

    // Regex matches:
    // 1. Pre-attribute docs
    // 2. Library name
    // 3. Post-attribute docs
    // 4. Trait name
    let lib_re = Regex::new(r#"(?s)((?:[ \t]*///[^\r\n]*[\r\n]+)*)\s*#\[eldritch_library\("([^"]+)"\)\]\s*((?:[ \t]*///[^\r\n]*[\r\n]+)*)\s*pub trait (\w+)"#).unwrap();

    // Regex matches:
    // 1. Pre-attribute docs
    // 2. Attribute content (optional, e.g. ("match"))
    // 3. Post-attribute docs
    // 4. Function name (allows r#name)
    // 5. Args (everything until closing paren)
    // 6. Return type inner (Result<TYPE, E>) - attempts to capture TYPE, handles nested brackets poorly but sufficient for current use
    //    Ideally we match `Result<` then balance brackets, but regex is weak here.
    //    We will match `Result<(.+?), String>`.

    let method_re = Regex::new(r#"(?s)((?:[ \t]*///[^\r\n]*[\r\n]+)*)\s*#\[eldritch_method(\([^)]+\))?\]\s*((?:[ \t]*///[^\r\n]*[\r\n]+)*)\s*fn\s+([r#]*\w+)\s*\(([^)]*)\)\s*->\s*Result<(.+?),\s*String>"#).unwrap();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == "lib.rs" {
            let content = fs::read_to_string(entry.path()).unwrap();

            // Search for library definition
            if let Some(caps) = lib_re.captures(&content) {
                let doc_pre = caps.get(1).map_or("", |m| m.as_str());
                let lib_name = caps.get(2).map_or("", |m| m.as_str()).to_string();
                let doc_post = caps.get(3).map_or("", |m| m.as_str());
                let trait_name = caps.get(4).map_or("", |m| m.as_str()).to_string();

                let full_doc = format!("{}{}", clean_docs(doc_pre), clean_docs(doc_post));

                let mut methods = Vec::new();

                // Iterate over methods
                for m_caps in method_re.captures_iter(&content) {
                    let m_doc_pre = m_caps.get(1).map_or("", |m| m.as_str());
                    let m_attr_content = m_caps.get(2).map_or("", |m| m.as_str());
                    let m_doc_post = m_caps.get(3).map_or("", |m| m.as_str());
                    let fn_name = m_caps.get(4).map_or("", |m| m.as_str());
                    let args_str = m_caps.get(5).map_or("", |m| m.as_str());
                    let ret_type_str = m_caps.get(6).map_or("", |m| m.as_str());

                    let mut final_name = fn_name.to_string();
                    if !m_attr_content.is_empty() {
                         // Extract name from ("name")
                         let trimmed = m_attr_content.trim_matches(|c| c == '(' || c == ')' || c == '"');
                         if !trimmed.is_empty() {
                             final_name = trimmed.to_string();
                         }
                    }

                    final_name = final_name.replace("r#", "");
                    let full_name = format!("{}.{}", lib_name, final_name);

                    let sig = format_signature(&full_name, args_str, ret_type_str);
                    let m_docs = format!("{}{}", clean_docs(m_doc_pre), clean_docs(m_doc_post));

                    methods.push(MethodDoc {
                        name: full_name,
                        signature: sig,
                        docs: m_docs.trim().to_string(),
                    });
                }

                methods.sort_by(|a, b| a.name.cmp(&b.name));

                libraries.push(LibraryDoc {
                    name: lib_name,
                    trait_name,
                    docs: full_doc.trim().to_string(),
                    methods,
                });
            }
        }
    }

    libraries.sort_by(|a, b| a.name.cmp(&b.name));
    libraries
}

fn clean_docs(raw: &str) -> String {
    raw.lines()
       .filter_map(|l| {
           let trimmed = l.trim();
           if trimmed.starts_with("///") {
               let content = trimmed[3..].trim();
               // We keep empty lines (content is empty string)
               Some(content)
           } else {
               None
           }
       })
       .filter(|l| !l.contains("pub trait"))
       .collect::<Vec<_>>()
       .join("\n")
}

fn split_args(args_raw: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for c in args_raw.chars() {
        match c {
            '<' => {
                depth += 1;
                current.push(c);
            },
            '>' => {
                depth -= 1;
                current.push(c);
            },
            ',' => {
                if depth == 0 {
                    args.push(current.trim().to_string());
                    current.clear();
                } else {
                    current.push(c);
                }
            },
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        args.push(current.trim().to_string());
    }

    args
}

fn format_signature(name: &str, args_raw: &str, ret_raw: &str) -> String {
    let args: Vec<String> = split_args(args_raw).into_iter()
        .filter(|s| !s.is_empty() && s != "&self")
        .map(|s| {
            let parts: Vec<&str> = s.splitn(2, ':').collect();
            if parts.len() == 2 {
                format!("{}: {}", parts[0].trim(), map_type(parts[1].trim()))
            } else {
                s.to_string()
            }
        })
        .collect();

    let ret = map_type(ret_raw.trim());

    format!("{}({}) -> {}", name, args.join(", "), ret)
}

fn map_type(t: &str) -> String {
    let t = t.trim();
    // Basic cleanup
    let t = t.replace("alloc::string::", "")
             .replace("alloc::vec::", "")
             .replace("alloc::collections::", "")
             .replace("eldritch_core::", "")
             .replace("std::collections::", "")
             .replace("super::", ""); // remove super:: if present

    if t == "String" { "str".to_string() }
    else if t == "i64" { "int".to_string() }
    else if t == "bool" { "bool".to_string() }
    else if t == "Vec<u8>" { "Bytes".to_string() }
    else if t.starts_with("Vec<") {
        let inner = &t[4..t.len()-1];
        format!("List<{}>", map_type(inner))
    }
    else if t.starts_with("BTreeMap<") {
        if t.contains("Value") {
            "Dict".to_string() // Simplify generalized dicts
        } else {
             let content = &t[9..t.len()-1];
             let parts = split_args(content);
             if parts.len() == 2 {
                 format!("Dict<{}, {}>", map_type(&parts[0]), map_type(&parts[1]))
             } else {
                 "Dict".to_string()
             }
        }
    }
    else if t.starts_with("Option<") {
        let inner = &t[7..t.len()-1];
        format!("Option<{}>", map_type(inner))
    }
    else if t == "Value" { "Value".to_string() }
    else if t == "()" { "None".to_string() }
    else { t }
}

fn generate_markdown(libs: &[LibraryDoc], out_path: &Path) {
    let mut content = String::new();
    content.push_str("---\n");
    content.push_str("title: Eldritch Standard Library\n");
    content.push_str("tags:\n - User Guide\n");
    content.push_str("description: Eldritch Standard Library Documentation\n");
    content.push_str("permalink: user-guide/eldritch-stdlib\n");
    content.push_str("---\n\n");

    content.push_str("# Standard Library\n\n");
    content.push_str("The following libraries are available in Eldritch.\n\n");

    for lib in libs {
        let title_name = if let Some(c) = lib.name.chars().next() {
            c.to_uppercase().collect::<String>() + &lib.name[1..]
        } else {
            lib.name.clone()
        };

        content.push_str(&format!("## {} Library\n", title_name));
        if !lib.docs.is_empty() {
             content.push_str(&format!("{}\n\n", lib.docs));
        }

        for method in &lib.methods {
            content.push_str(&format!("### {}\n", method.name));
            content.push_str(&format!("`{}`\n", method.signature));
            if !method.docs.is_empty() {
                content.push_str(&format!("{}\n\n", method.docs));
            } else {
                content.push_str("\n");
            }
        }
    }

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    fs::write(out_path, content).expect("Unable to write documentation file");
}
