extern crate alloc;
extern crate eldritch_core;

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use eldritch_core::Interpreter;
    use eldritch_core::Value;

    #[test]
    fn test_chr() {
        let mut interp = Interpreter::new();

        // Valid integers
        let res = interp.interpret("chr(65)");
        assert_eq!(res.unwrap(), Value::String(String::from("A")));

        let res = interp.interpret("chr(97)");
        assert_eq!(res.unwrap(), Value::String(String::from("a")));

        let res = interp.interpret("chr(8364)");
        assert_eq!(res.unwrap(), Value::String(String::from("€")));

        // Edge cases
        let res = interp.interpret("chr(0)");
        assert_eq!(res.unwrap(), Value::String(String::from("\0")));

        let res = interp.interpret("chr(1114111)"); // 0x10FFFF
        match res {
            Ok(_) => {}
            Err(e) => panic!("Should accept 0x10FFFF: {}", e),
        }

        // Invalid integer (out of range)
        let res = interp.interpret("chr(1114112)");
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .contains("chr() arg not in range(0x110000)")
        );

        let res = interp.interpret("chr(-1)");
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .contains("chr() arg not in range(0x110000)")
        );

        // Type error
        let res = interp.interpret("chr('A')");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("TypeError"));
    }

    #[test]
    fn test_ord() {
        let mut interp = Interpreter::new();

        // Valid strings
        let res = interp.interpret("ord('A')");
        assert_eq!(res.unwrap(), Value::Int(65));

        let res = interp.interpret("ord('a')");
        assert_eq!(res.unwrap(), Value::Int(97));

        let res = interp.interpret("ord('€')");
        assert_eq!(res.unwrap(), Value::Int(8364));

        // Valid bytes
        let res = interp.interpret("ord(bytes([65]))");
        assert_eq!(res.unwrap(), Value::Int(65));

        // Invalid strings (length != 1)
        let res = interp.interpret("ord('AB')");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("expected string of length 1"));

        let res = interp.interpret("ord('')");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("expected string of length 1"));

        // Invalid bytes (length != 1)
        let res = interp.interpret("ord(bytes([65, 66]))");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("expected bytes of length 1"));

        // Type error
        let res = interp.interpret("ord(1)");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("TypeError"));
    }
}
