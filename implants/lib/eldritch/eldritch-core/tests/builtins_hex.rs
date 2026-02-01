extern crate alloc;
extern crate eldritch_core;

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use eldritch_core::Interpreter;
    use eldritch_core::Value;

    #[test]
    fn test_hex() {
        let mut interp = Interpreter::new();

        // Valid positive integers
        let res = interp.interpret("hex(255)");
        assert_eq!(res.unwrap(), Value::String(String::from("0xff")));

        let res = interp.interpret("hex(42)");
        assert_eq!(res.unwrap(), Value::String(String::from("0x2a")));

        // Valid negative integers
        let res = interp.interpret("hex(-42)");
        assert_eq!(res.unwrap(), Value::String(String::from("-0x2a")));

        // Zero
        let res = interp.interpret("hex(0)");
        assert_eq!(res.unwrap(), Value::String(String::from("0x0")));

        // Large integer
        // 123456789123456789 = 0x1b69b4bacd05f15
        let res = interp.interpret("hex(123456789123456789)");
        assert_eq!(
            res.unwrap(),
            Value::String(String::from("0x1b69b4bacd05f15"))
        );

        // Invalid types
        let res = interp.interpret("hex(1.2)");
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .contains("hex() argument must be an integer")
        );

        let res = interp.interpret("hex('s')");
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .contains("hex() argument must be an integer")
        );

        // Argument count
        let res = interp.interpret("hex()");
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .contains("hex() takes exactly one argument")
        );

        let res = interp.interpret("hex(1, 2)");
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .contains("hex() takes exactly one argument")
        );
    }
}
