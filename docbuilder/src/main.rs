// Cargo.toml dependencies:
// [dependencies]
// regex = "1.10"

use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self}; // Import the env module to access command-line arguments

fn main() -> io::Result<()> {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if exactly one argument (the file path) is provided
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_pyi_file>", args[0]);
        return Ok(()); // Exit gracefully with a usage message
    }

    // The first argument (index 0) is the program name, so the file path is at index 1
    let pyi_file_path = &args[1];

    let pyi_content = fs::read_to_string(pyi_file_path)?;

    let markdown_output = convert_pyi_to_markdown(&pyi_content);

    println!("{}", markdown_output);
    Ok(())
}

/// Helper function to calculate minimum indentation and deindent lines.
/// This function now expects `docstring_raw` to include the triple quotes.
/// It calculates the base indentation from the first non-empty line (which could be the opening `"""`)
/// and removes that amount of whitespace from all lines.
fn deindent_docstring(docstring_raw: &str) -> String {
    let lines: Vec<&str> = docstring_raw.lines().collect();
    let mut base_indentation = usize::MAX;

    // Find the indentation of the first non-empty line

    for line in &lines {
        if !line.trim_start().is_empty() {
            base_indentation = line.chars().take_while(|&c| c.is_whitespace()).count();
            break; // Found the base indentation, no need to check further lines
        }
    }

    // If no non-empty lines were found (e.g., empty docstring), treat base indentation as 0
    if base_indentation == usize::MAX {
        base_indentation = 0;
    }

    // Process each line: remove the calculated base indentation
    let processed_lines: Vec<String> = lines
        .into_iter()
        .map(|line| {
            if line.len() >= base_indentation {
                line[base_indentation..].to_string()
            } else {
                // If a line is shorter than the base_indentation (e.g., an empty line),
                // keep it as is to preserve blank lines.
                line.to_string()
            }
        })
        .collect();

    let doc = processed_lines.join("\n");

    doc.strip_prefix("\"\"\"\n")
        .unwrap_or(&doc)
        .strip_suffix("\n\"\"\"")
        .unwrap_or(&doc)
        .trim()
        .to_string()
}

/// Converts the content of a .pyi file into a Markdown documentation string.
/// This function parses Python class and static method definitions,
/// extracting their names, signatures, and docstrings, and formats them
/// into Markdown
fn convert_pyi_to_markdown(pyi_content: &str) -> String {
    let mut markdown = String::new();

    // Regex to capture a class block:
    // (?P<class_name>...) : Captures the class name
    // (?P<docstring_full_block>...) : Captures the full docstring block including leading whitespace and triple quotes.
    let class_re = Regex::new(
        r#"(?s)class\s+(?P<class_name>\w+):\n(?P<docstring_full_block>\s+(?:[ \t]*)?"""[\s\S]*?""")(?:\s*@staticmethod[\s\S]*?|\s*)"#
    ).unwrap();

    let typeddict_re = Regex::new(r#"class (?P<class_name>\w+)\(TypedDict\):"#).unwrap();

    // Regex to capture a static method's signature and docstring:
    // (?P<method_name>...) : Captures the method name
    // (?P<params>...) : Captures the parameters
    // (?P<return_type>...) : Captures the return type
    // (?P<docstring_full_block>...) : Captures the full docstring block including leading whitespace and triple quotes.
    let method_re = Regex::new(
        r#"(?s)@staticmethod\s+def\s+(?P<method_name>\w+)\((?P<params>.*?)\)\s*->\s*(?P<return_type>.*?):\n(?P<docstring_full_block>\s+(?:[ \t]*)?"""[\s\S]*?""")\s+\.\.\."#
    ).unwrap();

    // Get all the TypedDicts. We want to replace these with Dict in the output
    let mut typed_dicts = HashSet::new();
    for td in typeddict_re.captures_iter(pyi_content) {
        typed_dicts.insert(td.name("class_name").unwrap().as_str());
    }

    // Library docstrings come from here:
    /*
    # Used for meta-style interactions with the agent itself.
    agent: Agent = ...
    assets: Assets = ...    # Used to interact with files stored natively in the agent.
    ....
    */
    let mut library_list_items = Vec::new();
    let toc_re =
        Regex::new(r"(#\s(?P<doc1>.*)\n)?(?P<name>[a-z]+):\s+\w+\s+=\s+...\s*#\s+(?P<doc2>.*)")
            .unwrap();
    for toc_v in toc_re.captures_iter(pyi_content) {
        let name = toc_v.name("name").unwrap().as_str();
        let mut doc = match toc_v.name("doc1") {
            Some(s) => s.as_str(),
            None => "",
        };
        if doc == "" {
            doc = match toc_v.name("doc2") {
                Some(s) => s.as_str(),
                None => "",
            };
        }
        library_list_items.push(format!("- `{}` - {}", name, doc,));
    }

    markdown.push_str("# Standard Library\n\n");
    markdown.push_str("The standard library is the default functionality that eldritch provides. It contains the following libraries:\n\n");
    markdown.push_str(&library_list_items.join("\n"));

    for class_cap in class_re.captures_iter(pyi_content) {
        markdown.push_str("---\n\n"); // Sepparator
        let class_name = class_cap.name("class_name").unwrap().as_str();
        let class_docstring_full_block = class_cap.name("docstring_full_block").unwrap().as_str(); // This now includes """ and leading whitespace
                                                                                                   // Apply de-indentation to the class docstring
        let formatted_class_docstring = deindent_docstring(class_docstring_full_block);

        // Add class header and docstring to markdown
        markdown.push_str(&format!("## {}\n\n", class_name));
        markdown.push_str(formatted_class_docstring.as_str());
        markdown.push_str("\n\n");

        // Extract the block of code belonging to this class to search for methods.
        // This is a simplified approach; a more robust parser would build an AST.
        // We assume methods are directly after the class docstring and before the next class or global variable.
        let class_block_start = class_cap.get(0).unwrap().end();
        let next_class_match = class_re.find_at(pyi_content, class_block_start);
        let next_global_instance_match = Regex::new(r#"(?s)^\w+:\s+\w+\s+=\s+\.\.\.\s*$"#)
            .unwrap()
            .find_at(pyi_content, class_block_start);

        let class_methods_end = if let Some(m) = next_class_match {
            m.start()
        } else if let Some(m) = next_global_instance_match {
            m.start()
        } else {
            pyi_content.len() // If no next class or global instance, assume methods extend to end of file
        };

        let class_methods_block =
            &pyi_content[class_cap.get(0).unwrap().start()..class_methods_end];

        // Iterate over each method definition within the current class's block
        for method_cap in method_re.captures_iter(class_methods_block) {
            let method_name = method_cap.name("method_name").unwrap().as_str();
            let params = method_cap.name("params").unwrap().as_str();
            let mut return_type = method_cap.name("return_type").unwrap().as_str().to_string();
            // Check if its a TypedDict
            for t in &typed_dicts {
                let rt = return_type.replace(t, "Dict");
                return_type = rt.clone();
            }
            let method_docstring_full_block =
                method_cap.name("docstring_full_block").unwrap().as_str();

            // Apply de-indentation to the method docstring, strips """ also
            let formatted_docstring = deindent_docstring(method_docstring_full_block);

            markdown.push_str(&format!(
                "### {}.{}\n\n",
                class_name.to_lowercase(),
                method_name
            ));

            let typedef = &format!(
                "`{}.{}({}) -> {}`",
                class_name.to_lowercase(),
                method_name,
                params.trim().replace(" = None", ""),
                return_type
            );

            // replace all the newlines in this
            let typedef = Regex::new(r"\n\s+")
                .unwrap()
                .replace_all(typedef, " ")
                .into_owned();

            markdown.push_str(typedef.as_str());
            markdown.push_str("\n\n");
            markdown.push_str(&formatted_docstring);
            markdown.push_str("\n\n");
        }
    }

    markdown
}
