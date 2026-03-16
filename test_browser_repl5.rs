use eldritch_wasm::browser::BrowserRepl;

fn main() {
    let mut repl = BrowserRepl::new();
    let text = r#"def find_all_pam_deny_so():
    common_pam_dirs = ["/usr/lib/*/security/pam_deny.so"]

    found_paths = []
    for common_pam_dir in common_pam_dirs:
        found_execs = file.list(common_pam_dir)
        for found_exec in found_execs:
            found_paths.append(found_exec['absolute_path'])

    paths_count = 0
    for _ in found_paths:
        paths_count += 1

    if paths_count == 0:
        print("Couldn't find any pam_deny.so files. Aborting.")
        return []

    print("Found " + str(paths_count) + " pam_deny.so files.")
    return found_paths
"#;
    let mut input_buffer = String::new();
    let mut state_current_block = String::new();

    for line in text.lines() {
        let res = repl.input(line);
        println!("input: {:?} -> res: {}", line, res);
    }
}
