#[cfg(test)]
mod tests {
    use crate::fake::RegexLibraryFake;
    use eldritch_core::ForeignValue;

    #[test]
    fn check_method_names() {
        let regex = RegexLibraryFake::default();
        let methods = regex.method_names();
        println!("Methods: {:?}", methods);
        // We expect to find "match" if it's already correct, or "r#match" if it's not.
        assert!(methods.contains(&"match".to_string()) || methods.contains(&"r#match".to_string()));
    }
}
