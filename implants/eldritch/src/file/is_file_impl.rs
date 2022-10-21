use anyhow::Result;

pub fn is_file(_path: String) -> Result<()> {
    unimplemented!("Method unimplemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile,tempdir};

    #[test]
    fn test_is_file_basic() -> anyhow::Result<()>{
        unimplemented!("Method unimplemented")
    }
}
