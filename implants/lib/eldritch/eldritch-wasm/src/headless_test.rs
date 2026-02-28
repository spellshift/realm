#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_repl_input() {
        let mut repl = HeadlessRepl::new();

        // Simple complete command
        let res = repl.input("print('hello')");
        assert!(res.contains("\"status\": \"complete\""));
        assert!(res.contains("\"payload\": \"print('hello')\""));

        // Incomplete command (def)
        let res = repl.input("def foo():");
        assert!(res.contains("\"status\": \"incomplete\""));

        // Continue incomplete command
        let res = repl.input("  pass");
        assert!(res.contains("\"status\": \"incomplete\"")); // Needs empty line

        // Finish incomplete command
        let res = repl.input("");
        assert!(res.contains("\"status\": \"complete\""));
        // Payload should contain full block
        assert!(res.contains("def foo():\\n  pass\\n"));
    }

    #[test]
    fn test_headless_repl_complete() {
        let repl = HeadlessRepl::new();

        // Check completions for global 'print'
        let res = repl.complete("pri", 3);
        assert!(res.contains("print"));

        // Check filtering of keywords
        let res = repl.complete("def", 3);
        assert!(!res.contains("def"));

        let res = repl.complete("pa", 2);
        assert!(!res.contains("pass"));
    }
}
