#[cfg(test)]
mod tests {
    extern crate alloc;
    use eldritch_core::{Interpreter, Value};

    fn run_code(code: &str) -> Result<Value, String> {
        let mut interp = Interpreter::new();
        interp.interpret(code)
    }

    #[test]
    fn test_len_no_args() {
        let res = run_code("len()");
        assert!(res.is_err(), "len() without args should error, not panic");
    }

    #[test]
    fn test_assert_no_args() {
        let res = run_code("assert()");
        assert!(
            res.is_err(),
            "assert() without args should error, not panic"
        );
    }

    #[test]
    fn test_assert_eq_no_args() {
        let res = run_code("assert_eq()");
        assert!(
            res.is_err(),
            "assert_eq() without args should error, not panic"
        );
    }

    #[test]
    fn test_assert_eq_one_arg() {
        let res = run_code("assert_eq(1)");
        assert!(
            res.is_err(),
            "assert_eq() with one arg should error, not panic"
        );
    }

    #[test]
    fn test_abs_invalid_args() {
        assert!(run_code("abs()").is_err());
        assert!(run_code("abs(1, 2)").is_err());
        assert!(run_code("abs('s')").is_err());
    }

    #[test]
    fn test_all_invalid_args() {
        assert!(run_code("all()").is_err());
        assert!(run_code("all(1, 2)").is_err());
        assert!(run_code("all(1)").is_err());
    }

    #[test]
    fn test_any_invalid_args() {
        assert!(run_code("any()").is_err());
        assert!(run_code("any(1, 2)").is_err());
        assert!(run_code("any(1)").is_err());
    }

    #[test]
    fn test_bool_invalid_args() {
        assert!(run_code("bool()").is_ok());
    }

    #[test]
    fn test_bytes_invalid_args() {
        assert!(run_code("bytes(1, 2)").is_err());
        assert!(run_code("bytes('a')").is_ok());
        assert!(run_code("bytes([1, 2])").is_ok());
        assert!(run_code("bytes([256])").is_err());
        assert!(run_code("bytes(-1)").is_err());
    }

    #[test]
    fn test_dict_invalid_args() {
        assert!(run_code("dict(1, 2)").is_err());
        assert!(run_code("dict(1)").is_err());
        assert!(run_code("dict(['a'])").is_err());
    }

    #[test]
    fn test_dir_invalid_args() {
        assert!(run_code("dir(1, 2)").is_ok());
    }

    #[test]
    fn test_enumerate_invalid_args() {
        assert!(run_code("enumerate()").is_err());
        assert!(run_code("enumerate(1)").is_err());
        assert!(run_code("enumerate([], 'a')").is_err());
    }

    #[test]
    fn test_float_invalid_args() {
        assert!(run_code("float(1, 2)").is_err());
        assert!(run_code("float('invalid')").is_err());
    }

    #[test]
    fn test_int_invalid_args() {
        assert!(run_code("int(1, 2)").is_err());
        assert!(run_code("int('invalid')").is_err());
    }

    #[test]
    fn test_len_invalid_args() {
        assert!(run_code("len(1)").is_err());
    }

    #[test]
    fn test_list_invalid_args() {
        assert!(run_code("list(1, 2)").is_err());
        assert!(run_code("list(1)").is_err());
    }

    #[test]
    fn test_max_invalid_args() {
        assert!(run_code("max()").is_err());
        assert!(run_code("max([])").is_err());
        assert!(run_code("max(1)").is_err());
    }

    #[test]
    fn test_min_invalid_args() {
        assert!(run_code("min()").is_err());
        assert!(run_code("min([])").is_err());
        assert!(run_code("min(1)").is_err());
    }

    #[test]
    fn test_pprint_invalid_args() {
        assert!(run_code("pprint()").is_err());
    }

    #[test]
    fn test_range_invalid_args() {
        assert!(run_code("range()").is_err());
        assert!(run_code("range(1, 2, 0)").is_err());
        assert!(run_code("range(1, 2, 3, 4)").is_err());
    }

    #[test]
    fn test_reversed_invalid_args() {
        assert!(run_code("reversed()").is_err());
        assert!(run_code("reversed(1)").is_err());
    }

    #[test]
    fn test_set_invalid_args() {
        assert!(run_code("set(1, 2)").is_err());
        assert!(run_code("set(1)").is_err());
    }

    #[test]
    fn test_sorted_invalid_args() {
        assert!(run_code("sorted()").is_err());
        assert!(run_code("sorted(1)").is_err());
    }

    #[test]
    fn test_tuple_invalid_args() {
        assert!(run_code("tuple(1, 2)").is_err());
        assert!(run_code("tuple(1)").is_err());
    }

    #[test]
    fn test_zip_invalid_args() {
        assert!(run_code("zip()").is_ok());
        assert!(run_code("zip(1)").is_err());
    }
}
