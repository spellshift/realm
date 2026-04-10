#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_repl_input() {
        let mut repl = BrowserRepl::new();

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
    fn test_browser_repl_meta_help() {
        let mut repl = BrowserRepl::new();

        // Simple complete command
        let res = repl.input("help");
        assert!(res.contains("\"status\": \"meta\""));
        assert!(res.contains("\"type\": \"help\""));

        let res = repl.input("help()");
        assert!(res.contains("\"status\": \"meta\""));
        assert!(res.contains("\"type\": \"help\""));

        let res = repl.input("help(sys)");
        assert!(res.contains("\"status\": \"meta\""));
        assert!(res.contains("\"type\": \"help\""));
        assert!(res.contains("\"target\": \"sys\""));

        let res = repl.input("help(sys.shell)");
        assert!(res.contains("\"status\": \"meta\""));
        assert!(res.contains("\"type\": \"help\""));
        assert!(res.contains("\"target\": \"sys.shell\""));
    }

    #[test]
    fn test_browser_repl_meta_ssh() {
        let mut repl = BrowserRepl::new();

        let res = repl.input("ssh(\"test:pass@127.0.0.1:22\")");
        assert!(res.contains("\"status\": \"meta\""));
        assert!(res.contains("\"type\": \"ssh\""));
        assert!(res.contains("\"target\": \"test:pass@127.0.0.1:22\""));
    }

    #[test]
    fn test_browser_repl_complete() {
        let repl = BrowserRepl::new();

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
