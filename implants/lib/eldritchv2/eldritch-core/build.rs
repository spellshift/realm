use std::{fs::OpenOptions, io::Write};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/interpreter/builtins");
    println!("cargo:rerun-if-changed=src/docs");
    println!("cargo:rerun-if-changed=../stdlib");

    // We only want to run this if we are in the main repo build environment
    // and not when published to crates.io or minimal environments
    if !std::env::var("CARGO_FEATURE_NO_DOC_GEN").is_ok() {
        generate_docs().unwrap();
    }
}

fn generate_docs() -> std::io::Result<()> {
    // We will attempt to locate the user guide.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let user_guide_path = std::path::Path::new(&manifest_dir)
        .parent() // eldritchv2
        .and_then(|p| p.parent()) // lib
        .and_then(|p| p.parent()) // implants
        .and_then(|p| p.parent()) // root
        .map(|p: &std::path::Path| p.join("docs/_docs/user-guide/eldritchv2-core.md"));

    if let Some(path) = user_guide_path {
        // Truncate the file to 0 bytes
        std::fs::File::create(&path)?;

        generate_header_docs(&path)?;

        match generate_core_docs(&path, &manifest_dir) {
            Ok(_) => println!("cargo:warning=Documentation updated successfully."),
            Err(e) => println!("cargo:warning=Failed to update documentation: {}", e),
        }
        // let libraries = parse_libraries(&user_guide_path);
        // match generate_stdlib_docs(&libraries, &user_guide_path) {
        //     Ok(_) => println!("cargo:warning=Documentation updated successfully."),
        //     Err(e) => println!("cargo:warning=Failed to update documentation: {}", e),
        // }
    }
    Ok(())
}

fn generate_header_docs(user_guide_path: &std::path::Path) -> std::io::Result<()> {
    let header_content = r#"---
title: Eldritch V2 core
tags:
 - User Guide
description: EldritchV2 Core User Guide
permalink: user-guide/eldritchv2-core
---
{% comment %} Generated from implants/lib/eldritchv2/eldritch-core/build.rs {% endcomment %}

# Overview

Eldritch V2 is a Starlark-like domain specific language used for scripting implant behaviors. It is designed to be familiar to Python users while remaining simple, safe, and easily embeddable.

## Quick Start

You can try the language in the [interactive REPL demo](/assets/eldritch-repl/index.html).

### Hello World

```python
print("Hello, World!")
```

### Variables and Types

Eldritch V2 is dynamically typed.

```python
x = 10              # int
y = 3.14            # float
name = "Eldritch"   # string
is_active = True    # bool
data = b"\x00\x01"  # bytes
items = [1, 2, 3]   # list
config = {"a": 1}   # dict
point = (10, 20)    # tuple
unique = {1, 2, 3}  # set
```

## Language Reference

### Control Flow

**If Statements**

```python
if x > 10:
    print("Big")
elif x == 10:
    print("Ten")
else:
    print("Small")
```

**Loops**

```python
# For loop
for i in range(5):
    print(i)

# While loop
while x > 0:
    x -= 1
    if x == 5:
        break
```

**Ternary Operator**

```python
status = "Active" if is_running else "Inactive"
```

### Functions

Functions are defined using `def`. They support positional arguments, keyword arguments, default values, `*args`, and `**kwargs`.

```python
def greet(name, greeting="Hello"):
    return "%s, %s!" % (greeting, name)

print(greet("World"))
```

### Modules

Eldritch V2 supports loading modules. Standard library modules (like `file`, `sys`) are available globally or can be imported if configured. In the standard environment, they are pre-loaded global objects.

```python
file.list(".")
```


## Built-in Functions

Eldritch V2 provides a rich set of built-in functions available in the global scope.


"#;
    let mut file = OpenOptions::new().append(true).open(user_guide_path)?;
    let _ = file.write_all(header_content.as_bytes());

    Ok(())
}

fn generate_core_docs(path: &std::path::Path, manifest_dir: &str) -> std::io::Result<()> {
    let builtins_dir = std::path::Path::new(manifest_dir).join("src/interpreter/builtins");
    let builtins_doc = parse_builtins(&builtins_dir)?;

    let mut file = std::fs::OpenOptions::new().append(true).open(path)?;
    file.write_all(builtins_doc.as_bytes())?;
    Ok(())
}

fn extract_doc_comments(content: &str) -> String {
    let mut doc = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(comment) = trimmed.strip_prefix("///") {
            let comment = comment.strip_prefix(" ").unwrap_or(comment);
            doc.push_str(comment);
            doc.push('\n');
        } else if !doc.is_empty() && !trimmed.starts_with("#[") && !trimmed.is_empty() {
            // Stop extracting if we hit code, but allow attributes or blank lines
            if !trimmed.starts_with("pub fn") {
                // heuristic to stop
            }
            break;
        }
    }
    doc
}

fn extract_library_name(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.contains("#[eldritch_library(") {
            let start = line.find('"')? + 1;
            let end = line.rfind('"')?;
            return Some(line[start..end].to_string());
        }
    }
    None
}

fn parse_builtins(builtins_dir: &std::path::Path) -> std::io::Result<String> {
    let mut entries: Vec<_> = std::fs::read_dir(builtins_dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    entries.sort();

    let mut doc = String::new();

    // We can group them if we want, but for now flat list or simple grouping
    // Current user guide groups them: Core, Type Constructors, Math & Logic, Iteration
    // To do this automatically, we might need metadata in the files.
    // For now, I'll just list them alphabetically or try to infer.

    // Let's rely on a simple map for grouping based on filename
    let groups = [
        (
            "Core",
            vec![
                "print",
                "pprint",
                "len",
                "type_",
                "dir",
                "libs",
                "builtins",
                "fail",
                "assert",
                "assert_eq",
            ],
        ),
        (
            "Type Constructors & Conversion",
            vec![
                "bool", "int", "float", "str", "bytes", "list", "dict", "set", "tuple",
            ],
        ),
        ("Math & Logic", vec!["abs", "max", "min", "range"]),
        (
            "Iteration",
            vec!["all", "any", "enumerate", "reversed", "sorted", "zip"],
        ),
    ];

    for (group_name, files) in groups.iter() {
        doc.push_str(&format!("### {}\n\n", group_name));

        for name in files.iter() {
            let file_path: std::path::PathBuf = builtins_dir.join(format!("{}.rs", name));

            if file_path.exists() {
                let content = std::fs::read_to_string(&file_path)?;
                let parsed_doc = extract_doc_comments(&content);

                // If no doc, skip or put placeholder
                if !parsed_doc.is_empty() {
                    // I'll define a convention: First line is signature, rest is description.
                    let mut lines = parsed_doc.lines();
                    if let Some(sig) = lines.next() {
                        let desc: String = lines.collect::<Vec<&str>>().join("\n");
                        // Remove leading ` if present in signature for clean formatting
                        let clean_sig = sig.trim();
                        println!("cargo:warning=Processing builtin: {}", clean_sig);
                        println!("cargo:warning=Processing builtin: {}", desc);

                        doc.push_str(&format!("#### {}\n{}\n\n", clean_sig, desc));
                    }
                } else {
                    // If no comments, try to match existing manual docs or just name
                    let display_name = if *name == "type_" { "type" } else { name };
                    doc.push_str(&format!("####   **`{}`**\n\n", display_name));
                }
            }
        }
    }

    Ok(doc)
}

// fn generate_docs(user_guide_path: &std::path::Path, core_path: &str) -> std::io::Result<()> {
//     // 2. Generate Builtins Docs
//     let builtins_dir = std::path::Path::new(core_path).join("src/interpreter/builtins");
//     let builtins_doc = parse_builtins(&builtins_dir)?;
//     doc_content.push_str(&builtins_doc);

//     doc_content.push_str("\n---\n\n");

//     // 3. Generate Methods Docs
//     doc_content.push_str("## Type Methods\n\n");
//     doc_content.push_str("Methods available on built-in types.\n\n");
//     let methods_doc_path = std::path::Path::new(core_path).join("src/docs/methods.rs");
//     let methods_doc = parse_methods_doc(&methods_doc_path)?;
//     doc_content.push_str(&methods_doc);

//     doc_content.push_str("\n---\n\n");

//     // 4. Generate Standard Library Docs
//     doc_content.push_str("## Standard Library\n\n");
//     doc_content.push_str("The standard library provides powerful capabilities for interacting with the host system.\n\n");

//     let stdlib_dir = std::path::Path::new(core_path)
//         .parent()
//         .unwrap()
//         .join("stdlib");
//     let stdlib_doc = parse_stdlib(&stdlib_dir)?;
//     doc_content.push_str(&stdlib_doc);

//     // Write back to file
//     std::fs::write(user_guide_path, doc_content)?;

//     Ok(())
// }

// /// Formats a documentation entry with a function/method name and description.
// ///
// /// The description should be multi-line text that follows the pattern:
// /// - First line(s): Description text
// /// - **Parameters** section with indented parameter lines
// /// - **Returns** section with indented return info
// /// - **Errors** section (optional) with indented error info
// fn format_doc_entry(name: &str, description: &str) -> String {
//     let mut output = format!("*   **`{}`**\n", name);

//     for line in description.lines() {
//         output.push_str("    ");
//         output.push_str(line);
//         output.push('\n');
//     }
//     output.push('\n');

//     output
// }

// fn parse_methods_doc(path: &std::path::Path) -> std::io::Result<String> {
//     let content = std::fs::read_to_string(path)?;
//     let mut output = String::new();
//     let mut in_method = false;

//     for line in content.lines() {
//         let trimmed = line.trim();
//         if let Some(stripped) = trimmed.strip_prefix("//: ") {
//             if !stripped.is_empty() {
//                 // Check if this is a method name (contains a dot and doesn't start with a marker like -, **, etc.)
//                 if !in_method
//                     && stripped.contains('.')
//                     && !stripped.starts_with('-')
//                     && !stripped.starts_with('*')
//                 {
//                     // New entry start, e.g., "list.append"
//                     output.push_str(&format!("*   **`{}`**\n", stripped));
//                     in_method = true;
//                 } else {
//                     // Method description or details - indent it
//                     output.push_str("    ");
//                     output.push_str(stripped);
//                     output.push('\n');
//                 }
//             } else {
//                 // Empty line - preserve it
//                 output.push('\n');
//             }
//         } else if let Some(stripped) = trimmed.strip_prefix("// ") {
//             // Section headers
//             if stripped.chars().all(|c| c == '=') {
//                 // Separator line, ignore
//                 continue;
//             }
//             if stripped.ends_with(" methods") {
//                 output.push_str(&format!("### {}\n\n", stripped));
//             }
//             // Reset method state when hitting non-doc comment
//             in_method = false;
//         } else {
//             // Reset method state on non-comment lines
//             in_method = false;
//         }
//     }

//     Ok(output)
// }

// fn extract_library_doc(content: &str) -> String {
//     // Extract comments above `pub trait`
//     let mut doc = String::new();
//     let mut buffer = Vec::new();

//     for line in content.lines() {
//         let trimmed = line.trim();
//         if trimmed.starts_with("///") {
//             let comment = trimmed
//                 .strip_prefix("///")
//                 .unwrap()
//                 .strip_prefix(" ")
//                 .unwrap_or(trimmed.strip_prefix("///").unwrap());
//             buffer.push(comment.to_string());
//         } else if trimmed.contains("pub trait") && trimmed.contains("Library") {
//             // Found the trait, use the buffered comments
//             for line in &buffer {
//                 doc.push_str(line);
//                 doc.push('\n');
//             }
//             break;
//         } else if !trimmed.starts_with("#[") && !trimmed.is_empty() {
//             // Reset buffer if we hit other code
//             buffer.clear();
//         }
//     }
//     doc
// }

// fn extract_methods(content: &str) -> Vec<(String, String)> {
//     let mut methods = Vec::new();
//     let mut buffer = Vec::new();
//     let mut is_method = false;
//     let mut method_name = String::new();

//     for line in content.lines() {
//         let trimmed = line.trim();
//         if trimmed.starts_with("///") {
//             let comment = trimmed
//                 .strip_prefix("///")
//                 .unwrap()
//                 .strip_prefix(" ")
//                 .unwrap_or(trimmed.strip_prefix("///").unwrap());
//             buffer.push(comment.to_string());
//         } else if trimmed.starts_with("#[eldritch_method") {
//             is_method = true;
//             if trimmed.contains("(\"") {
//                 // extract alias
//                 let start = trimmed.find('"').unwrap() + 1;
//                 let end = trimmed.rfind('"').unwrap();
//                 method_name = trimmed[start..end].to_string();
//             }
//         } else if is_method && trimmed.starts_with("fn ") {
//             // extract function name if alias wasn't present
//             if method_name.is_empty() {
//                 let start = 3;
//                 let end = trimmed.find('(').unwrap_or(trimmed.len());
//                 method_name = trimmed[start..end].trim().to_string();
//                 // handle raw identifiers
//                 if method_name.starts_with("r#") {
//                     method_name = method_name[2..].to_string();
//                 }
//                 // handle trailing underscore convention (move_, type_)
//                 if method_name.ends_with('_') {
//                     method_name.pop();
//                 }
//             }

//             // Construct signature from arguments
//             // This is hard to do perfectly with regex/string parsing, so we'll rely on the doc comment
//             // or just output the description.
//             // My plan: The doc comment should contain the signature or at least description.

//             methods.push((method_name.clone(), buffer.join("\n")));

//             buffer.clear();
//             is_method = false;
//             method_name.clear();
//         } else if !trimmed.starts_with("#[") && !trimmed.is_empty() {
//             buffer.clear();
//             is_method = false;
//             method_name.clear();
//         }
//     }
//     methods
// }

// fn parse_libraries(root: &Path) -> Vec<LibraryDoc> {
//     let mut libraries = Vec::new();

//     // Regex matches:
//     // 1. Pre-attribute docs
//     // 2. Library name
//     // 3. Post-attribute docs
//     // 4. Trait name
//     let lib_re = Regex::new(r#"(?s)((?:[ \t]*///[^\r\n]*[\r\n]+)*)\s*#\[eldritch_library\("([^"]+)"\)\]\s*((?:[ \t]*///[^\r\n]*[\r\n]+)*)\s*pub trait (\w+)"#).unwrap();

//     // Regex matches:
//     // 1. Pre-attribute docs
//     // 2. Attribute content (optional, e.g. ("match"))
//     // 3. Post-attribute docs
//     // 4. Function name (allows r#name)
//     // 5. Args (everything until closing paren)
//     // 6. Return type inner (Result<TYPE, E>) - attempts to capture TYPE, handles nested brackets poorly but sufficient for current use
//     //    Ideally we match `Result<` then balance brackets, but regex is weak here.
//     //    We will match `Result<(.+?), String>`.

//     let method_re = Regex::new(r#"(?s)((?:[ \t]*///[^\r\n]*[\r\n]+)*)\s*#\[eldritch_method(\([^)]+\))?\]\s*((?:[ \t]*///[^\r\n]*[\r\n]+)*)\s*fn\s+([r#]*\w+)\s*\(([^)]*)\)\s*->\s*Result<(.+?),\s*String>"#).unwrap();

//     for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
//         if entry.file_name() == "lib.rs" {
//             let content = fs::read_to_string(entry.path()).unwrap();

//             // Search for library definition
//             if let Some(caps) = lib_re.captures(&content) {
//                 let doc_pre = caps.get(1).map_or("", |m| m.as_str());
//                 let lib_name = caps.get(2).map_or("", |m| m.as_str()).to_string();
//                 let doc_post = caps.get(3).map_or("", |m| m.as_str());
//                 let trait_name = caps.get(4).map_or("", |m| m.as_str()).to_string();

//                 let full_doc = format!("{}{}", clean_docs(doc_pre), clean_docs(doc_post));

//                 let mut methods = Vec::new();

//                 // Iterate over methods
//                 for m_caps in method_re.captures_iter(&content) {
//                     let m_doc_pre = m_caps.get(1).map_or("", |m| m.as_str());
//                     let m_attr_content = m_caps.get(2).map_or("", |m| m.as_str());
//                     let m_doc_post = m_caps.get(3).map_or("", |m| m.as_str());
//                     let fn_name = m_caps.get(4).map_or("", |m| m.as_str());
//                     let args_str = m_caps.get(5).map_or("", |m| m.as_str());
//                     let ret_type_str = m_caps.get(6).map_or("", |m| m.as_str());

//                     let mut final_name = fn_name.to_string();
//                     if !m_attr_content.is_empty() {
//                         // Extract name from ("name")
//                         let trimmed =
//                             m_attr_content.trim_matches(|c| c == '(' || c == ')' || c == '"');
//                         if !trimmed.is_empty() {
//                             final_name = trimmed.to_string();
//                         }
//                     }

//                     final_name = final_name.replace("r#", "");
//                     let full_name = format!("{}.{}", lib_name, final_name);

//                     let sig = format_signature(&full_name, args_str, ret_type_str);
//                     let m_docs = format!("{}{}", clean_docs(m_doc_pre), clean_docs(m_doc_post));

//                     methods.push(MethodDoc {
//                         name: full_name,
//                         signature: sig,
//                         docs: m_docs.trim().to_string(),
//                     });
//                 }

//                 methods.sort_by(|a, b| a.name.cmp(&b.name));

//                 libraries.push(LibraryDoc {
//                     name: lib_name,
//                     trait_name,
//                     docs: full_doc.trim().to_string(),
//                     methods,
//                 });
//             }
//         }
//     }

//     libraries.sort_by(|a, b| a.name.cmp(&b.name));
//     libraries
// }

// fn clean_docs(raw: &str) -> String {
//     raw.lines()
//         .filter_map(|l| {
//             let trimmed = l.trim();
//             trimmed.strip_prefix("///").map(|content| content.trim())
//         })
//         .filter(|l| !l.contains("pub trait"))
//         .collect::<Vec<_>>()
//         .join("\n")
// }

// fn split_args(args_raw: &str) -> Vec<String> {
//     let mut args = Vec::new();
//     let mut current = String::new();
//     let mut depth = 0;

//     for c in args_raw.chars() {
//         match c {
//             '<' => {
//                 depth += 1;
//                 current.push(c);
//             }
//             '>' => {
//                 depth -= 1;
//                 current.push(c);
//             }
//             ',' => {
//                 if depth == 0 {
//                     args.push(current.trim().to_string());
//                     current.clear();
//                 } else {
//                     current.push(c);
//                 }
//             }
//             _ => current.push(c),
//         }
//     }
//     if !current.trim().is_empty() {
//         args.push(current.trim().to_string());
//     }

//     args
// }

// fn format_signature(name: &str, args_raw: &str, ret_raw: &str) -> String {
//     let args: Vec<String> = split_args(args_raw)
//         .into_iter()
//         .filter(|s| !s.is_empty() && s != "&self")
//         .map(|s| {
//             let parts: Vec<&str> = s.splitn(2, ':').collect();
//             if parts.len() == 2 {
//                 format!("{}: {}", parts[0].trim(), map_type(parts[1].trim()))
//             } else {
//                 s.to_string()
//             }
//         })
//         .collect();

//     let ret = map_type(ret_raw.trim());

//     format!("{}({}) -> {}", name, args.join(", "), ret)
// }

// fn map_type(t: &str) -> String {
//     let t = t.trim();
//     // Basic cleanup
//     let t = t
//         .replace("alloc::string::", "")
//         .replace("alloc::vec::", "")
//         .replace("alloc::collections::", "")
//         .replace("eldritch_core::", "")
//         .replace("std::collections::", "")
//         .replace("super::", ""); // remove super:: if present

//     if t == "String" {
//         "str".to_string()
//     } else if t == "i64" {
//         "int".to_string()
//     } else if t == "bool" {
//         "bool".to_string()
//     } else if t == "Vec<u8>" {
//         "Bytes".to_string()
//     } else if t.starts_with("Vec<") {
//         let inner = &t[4..t.len() - 1];
//         format!("List<{}>", map_type(inner))
//     } else if t.starts_with("BTreeMap<") {
//         if t.contains("Value") {
//             "Dict".to_string() // Simplify generalized dicts
//         } else {
//             let content = &t[9..t.len() - 1];
//             let parts = split_args(content);
//             if parts.len() == 2 {
//                 format!("Dict<{}, {}>", map_type(&parts[0]), map_type(&parts[1]))
//             } else {
//                 "Dict".to_string()
//             }
//         }
//     } else if t.starts_with("Option<") {
//         let inner = &t[7..t.len() - 1];
//         format!("Option<{}>", map_type(inner))
//     } else if t == "Value" {
//         "Value".to_string()
//     } else if t == "()" {
//         "None".to_string()
//     } else {
//         t
//     }
// }

// fn generate_markdown(libs: &[LibraryDoc], out_path: &Path) {
//     let mut content = r#"---
// title: Eldritch V2
// tags:
//  - User Guide
// description: Eldritch V2 Standard Library Documentation
// permalink: user-guide/eldritchv2
// ---
// {% comment %} Generated from implants/lib/eldritchv2/eldritchv2/build.rs {% endcomment %}

// # Standard Library

// The following libraries are available in Eldritch.

// "#
//     .to_string();

//     for lib in libs {
//         let title_name = if let Some(c) = lib.name.chars().next() {
//             c.to_uppercase().collect::<String>() + &lib.name[1..]
//         } else {
//             lib.name.clone()
//         };

//         content.push_str(&format!("## {} Library\n", title_name));
//         if !lib.docs.is_empty() {
//             content.push_str(&format!("{}\n\n", lib.docs));
//         }

//         for method in &lib.methods {
//             content.push_str(&format!("### {}\n", method.name));
//             content.push_str(&format!("`{}`\n", method.signature));
//             if !method.docs.is_empty() {
//                 content.push_str(&format!("{}\n\n", method.docs));
//             } else {
//                 content.push('\n');
//             }
//         }
//     }

//     if let Some(parent) = out_path.parent() {
//         fs::create_dir_all(parent).unwrap();
//     }

//     fs::write(out_path, content).expect("Unable to write documentation file");
// }
