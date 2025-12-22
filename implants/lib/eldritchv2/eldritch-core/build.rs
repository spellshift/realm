fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/interpreter/builtins");
    println!("cargo:rerun-if-changed=src/docs");
    println!("cargo:rerun-if-changed=../stdlib");

    // We only want to run this if we are in the main repo build environment
    // and not when published to crates.io or minimal environments
    if std::env::var("CARGO_FEATURE_NO_DOC_GEN").is_ok() {
        return;
    }

    // Since we can't easily modify files outside the build directory in a reliable way across all environments,
    // and build scripts shouldn't modify source code generally, we will generate the documentation
    // and print it to stdout (which is hidden) or write to a file in OUT_DIR.
    // However, the user request specifically asked to "output to our user guide".
    // In a local dev environment, this is acceptable.

    // We will attempt to locate the user guide.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let user_guide_path = std::path::Path::new(&manifest_dir)
        .parent() // eldritchv2
        .and_then(|p| p.parent()) // lib
        .and_then(|p| p.parent()) // implants
        .and_then(|p| p.parent()) // root
        .map(|p| p.join("docs/_docs/user-guide/eldritchv2-core.md"));

    if let Some(path) = user_guide_path {
        match generate_docs(&path, &manifest_dir) {
            Ok(_) => println!("cargo:warning=Documentation updated successfully."),
            Err(e) => println!("cargo:warning=Failed to update documentation: {}", e),
        }
    }
}

fn generate_docs(user_guide_path: &std::path::Path, core_path: &str) -> std::io::Result<()> {
    let mut doc_content = r#"---
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


"#.to_string();

    // 2. Generate Builtins Docs
    let builtins_dir = std::path::Path::new(core_path).join("src/interpreter/builtins");
    let builtins_doc = parse_builtins(&builtins_dir)?;
    doc_content.push_str(&builtins_doc);

    doc_content.push_str("\n---\n\n");

    // 3. Generate Methods Docs
    doc_content.push_str("## Type Methods\n\n");
    doc_content.push_str("Methods available on built-in types.\n\n");
    let methods_doc_path = std::path::Path::new(core_path).join("src/docs/methods.rs");
    let methods_doc = parse_methods_doc(&methods_doc_path)?;
    doc_content.push_str(&methods_doc);

    doc_content.push_str("\n---\n\n");

    // 4. Generate Standard Library Docs
    doc_content.push_str("## Standard Library\n\n");
    doc_content.push_str("The standard library provides powerful capabilities for interacting with the host system.\n\n");

    let stdlib_dir = std::path::Path::new(core_path)
        .parent()
        .unwrap()
        .join("stdlib");
    let stdlib_doc = parse_stdlib(&stdlib_dir)?;
    doc_content.push_str(&stdlib_doc);

    // Write back to file
    std::fs::write(user_guide_path, doc_content)?;

    Ok(())
}

fn parse_builtins(dir: &std::path::Path) -> std::io::Result<String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
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
            let filename = if *name == "type_" {
                "type_.rs"
            } else {
                // Handle cases where filename matches
                // We need to construct the path
                // This is a bit hacky, but works for the known set
                // Ideally we scan files and read their docs
                ""
            };

            let file_path = if filename.is_empty() {
                dir.join(format!("{}.rs", name))
            } else {
                dir.join(filename)
            };

            if file_path.exists() {
                let content = std::fs::read_to_string(&file_path)?;
                let parsed_doc = extract_doc_comments(&content);

                // If no doc, skip or put placeholder
                if !parsed_doc.is_empty() {
                    // Format: * **`name(...)`**: Description
                    // We need to extract the signature and description
                    // Let's assume the doc comments are formatted like:
                    // /// `name(args)`: Description
                    // or just Description.

                    // I'll define a convention: First line is signature, rest is description.
                    let mut lines = parsed_doc.lines();
                    if let Some(sig) = lines.next() {
                        let desc: String = lines.collect::<Vec<&str>>().join("\n    ");
                        // Remove leading ` if present in signature for clean formatting
                        let clean_sig = sig.trim().trim_matches('`');

                        doc.push_str(&format!("*   **`{}`**\n    {}\n\n", clean_sig, desc));
                    }
                } else {
                    // If no comments, try to match existing manual docs or just name
                    let display_name = if *name == "type_" { "type" } else { name };
                    doc.push_str(&format!("*   **`{}`**\n\n", display_name));
                }
            }
        }
    }

    Ok(doc)
}

fn parse_methods_doc(path: &std::path::Path) -> std::io::Result<String> {
    let content = std::fs::read_to_string(path)?;
    let mut doc = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(content) = trimmed.strip_prefix("//: ") {
            doc.push_str(content);
            doc.push('\n');
        } else if let Some(text) = trimmed.strip_prefix("// ") {
            // Treat as markdown header or text if it looks like it
            // Actually, my `methods.rs` structure uses `// Header` and `//: content`

            // Check if it's a header
            if text.chars().all(|c| c == '=') {
                // It's a separator line, ignore
            } else if text.chars().any(|c| c.is_alphanumeric()) {
                // Likely a header
                if !doc.ends_with("## ") {
                    // Avoid double headers
                    doc.push_str(&format!("### {}\n\n", text));
                }
            }
        }
    }

    // Post-process to format entries
    // My format in methods.rs is:
    // //: list.append
    // //: Description...

    // I want to convert `list.append` to `* **`list.append(...)`**`
    // But I don't have the signature in the first line of my `methods.rs` I wrote.
    // I wrote: `//: list.append`

    // Let's improve the parser to be smarter.

    let mut output = String::new();
    let mut current_entry = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("//: ") {
            if !stripped.is_empty() {
                if current_entry.is_empty() && stripped.contains('.') {
                    // New entry start, e.g., "list.append"
                    current_entry = stripped.to_string();
                    output.push_str(&format!("*   **`{}`**\n", current_entry));
                } else {
                    output.push_str("    ");
                    output.push_str(stripped);
                    output.push('\n');
                }
            } else {
                output.push('\n');
            }
        } else if let Some(stripped) = trimmed.strip_prefix("// ") {
            if stripped.chars().all(|c| c == '=') {
                continue;
            }
            if stripped.ends_with(" methods") {
                output.push_str(&format!("### {}\n\n", stripped));
            }
            // Reset current entry when hitting non-doc comment
            current_entry.clear();
        } else {
            current_entry.clear();
        }
    }

    Ok(output)
}

fn parse_stdlib(dir: &std::path::Path) -> std::io::Result<String> {
    let mut doc = String::new();

    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    entries.sort();

    for path in entries {
        if path.is_dir() {
            let lib_rs = path.join("src/lib.rs");
            if lib_rs.exists() {
                let content = std::fs::read_to_string(&lib_rs)?;

                // Parse the library name and doc
                // Look for `#[eldritch_library("name")]`
                // and doc comments above `pub trait ...`

                if let Some(lib_name) = extract_library_name(&content) {
                    // Capitalize first letter
                    let title = if lib_name == "http" || lib_name == "ssh" {
                        lib_name.to_uppercase()
                    } else {
                        let mut c = lib_name.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    };

                    doc.push_str(&format!("### {}\n\n", title));

                    // Extract library level docs
                    let lib_doc = extract_library_doc(&content);
                    doc.push_str(&lib_doc);
                    doc.push_str("\n\n");

                    // Extract methods
                    // Look for `#[eldritch_method...]`
                    let methods = extract_methods(&content);
                    for (name, sig_doc) in methods {
                        doc.push_str(&format!("*   **`{}.{}`**\n", lib_name, name));
                        for line in sig_doc.lines() {
                            doc.push_str("    ");
                            doc.push_str(line);
                            doc.push('\n');
                        }
                        doc.push('\n');
                    }
                }
            }
        }
    }

    Ok(doc)
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

fn extract_library_doc(content: &str) -> String {
    // Extract comments above `pub trait`
    let mut doc = String::new();
    let mut buffer = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("///") {
            let comment = trimmed
                .strip_prefix("///")
                .unwrap()
                .strip_prefix(" ")
                .unwrap_or(trimmed.strip_prefix("///").unwrap());
            buffer.push(comment.to_string());
        } else if trimmed.contains("pub trait") && trimmed.contains("Library") {
            // Found the trait, use the buffered comments
            for line in &buffer {
                doc.push_str(line);
                doc.push('\n');
            }
            break;
        } else if !trimmed.starts_with("#[") && !trimmed.is_empty() {
            // Reset buffer if we hit other code
            buffer.clear();
        }
    }
    doc
}

fn extract_methods(content: &str) -> Vec<(String, String)> {
    let mut methods = Vec::new();
    let mut buffer = Vec::new();
    let mut is_method = false;
    let mut method_name = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("///") {
            let comment = trimmed
                .strip_prefix("///")
                .unwrap()
                .strip_prefix(" ")
                .unwrap_or(trimmed.strip_prefix("///").unwrap());
            buffer.push(comment.to_string());
        } else if trimmed.starts_with("#[eldritch_method") {
            is_method = true;
            if trimmed.contains("(\"") {
                // extract alias
                let start = trimmed.find('"').unwrap() + 1;
                let end = trimmed.rfind('"').unwrap();
                method_name = trimmed[start..end].to_string();
            }
        } else if is_method && trimmed.starts_with("fn ") {
            // extract function name if alias wasn't present
            if method_name.is_empty() {
                let start = 3;
                let end = trimmed.find('(').unwrap_or(trimmed.len());
                method_name = trimmed[start..end].trim().to_string();
                // handle raw identifiers
                if method_name.starts_with("r#") {
                    method_name = method_name[2..].to_string();
                }
                // handle trailing underscore convention (move_, type_)
                if method_name.ends_with('_') {
                    method_name.pop();
                }
            }

            // Construct signature from arguments
            // This is hard to do perfectly with regex/string parsing, so we'll rely on the doc comment
            // or just output the description.
            // My plan: The doc comment should contain the signature or at least description.

            methods.push((method_name.clone(), buffer.join("\n")));

            buffer.clear();
            is_method = false;
            method_name.clear();
        } else if !trimmed.starts_with("#[") && !trimmed.is_empty() {
            buffer.clear();
            is_method = false;
            method_name.clear();
        }
    }
    methods
}
