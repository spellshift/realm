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

    // To make it complete, we append an extra newline to signal block end, or we call input("") to end it
    let res = repl.input(text);
    println!("Whole block result: {}", res);
    let res = repl.input("");
    println!("Whole block result with extra newline: {}", res);
}
