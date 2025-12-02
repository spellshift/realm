#[cfg(test)]
mod tests {
    extern crate alloc;
    use eldritch_core::{Interpreter, Value};

    fn run_code(code: &str) -> Result<Value, String> {
        let mut interp = Interpreter::new();
        interp.interpret(code)
    }

    // List Methods
    #[test]
    fn test_list_append() {
        assert!(run_code("[].append()").is_err());
        assert!(run_code("[].append(1, 2)").is_err());
    }

    #[test]
    fn test_list_extend() {
        assert!(run_code("[].extend()").is_err());
        assert!(run_code("[].extend(1)").is_err());
    }

    #[test]
    fn test_list_insert() {
        assert!(run_code("[].insert()").is_err());
        assert!(run_code("[].insert(1)").is_err());
        assert!(run_code("[].insert('a', 1)").is_err());
    }

    #[test]
    fn test_list_remove() {
        assert!(run_code("[].remove()").is_err());
        assert!(run_code("[].remove(1)").is_err());
    }

    #[test]
    fn test_list_index() {
        assert!(run_code("[].index()").is_err());
        assert!(run_code("[].index(1)").is_err());
    }

    #[test]
    fn test_list_pop() {
        assert!(run_code("[].pop()").is_err()); // empty list
    }

    // Dictionary Methods
    #[test]
    fn test_dict_get() {
        assert!(run_code("{}.get()").is_err());
        assert!(run_code("{}.get(1, 2, 3)").is_err());
    }

    #[test]
    fn test_dict_update() {
        assert!(run_code("{}.update()").is_err());
        assert!(run_code("{}.update(1)").is_err());
    }

    #[test]
    fn test_dict_popitem() {
        assert!(run_code("{}.popitem()").is_err());
    }

    // Set Methods
    #[test]
    fn test_set_add() {
        assert!(run_code("set().add()").is_err());
    }

    #[test]
    fn test_set_remove() {
        assert!(run_code("set().remove(1)").is_err());
    }

    #[test]
    fn test_set_pop() {
        assert!(run_code("set().pop()").is_err());
    }

    #[test]
    fn test_set_union() {
        assert!(run_code("set().union()").is_err());
        assert!(run_code("set().union(1)").is_err());
    }

    // String Methods
    #[test]
    fn test_str_split() {
        assert!(run_code("''.split()").is_ok());
    }

    #[test]
    fn test_str_startswith() {
        assert!(run_code("''.startswith()").is_err());
    }

    #[test]
    fn test_str_endswith() {
        assert!(run_code("''.endswith()").is_err());
    }

    #[test]
    fn test_str_find() {
        assert!(run_code("''.find()").is_err());
    }

    #[test]
    fn test_str_replace() {
        assert!(run_code("''.replace()").is_err());
        assert!(run_code("''.replace('a')").is_err());
    }

    #[test]
    fn test_str_join() {
        assert!(run_code("''.join()").is_err());
        assert!(run_code("''.join(1)").is_err());
    }
}
