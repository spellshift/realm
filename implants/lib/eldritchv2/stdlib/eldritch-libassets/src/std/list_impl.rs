use crate::RustEmbed;
use alloc::string::String;
use alloc::vec::Vec;

pub fn list<A: RustEmbed>(remote_assets: &[String]) -> Result<Vec<String>, String> {
    let mut files: Vec<String> = A::iter().map(|s| s.to_string()).collect();
    // Append remote assets to the list if they are not already there
    for remote in remote_assets {
        if !files.contains(remote) {
            files.push(remote.clone());
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list() {
        use super::super::read_binary_impl::tests::TestAsset;
        let remote_files = vec!["remote1.txt".to_string(), "remote2.txt".to_string()];
        let list_result = list::<TestAsset>(&remote_files).unwrap();
        assert!(
            list_result
                .iter()
                .any(|f| f.contains("print/main.eldritch"))
        );
        assert!(list_result.contains(&"remote1.txt".to_string()));
        assert!(list_result.contains(&"remote2.txt".to_string()));
    }
}
