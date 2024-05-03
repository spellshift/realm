use anyhow::Result;
use rand::{distributions::Alphanumeric, Rng}; // 0.8

pub fn string(length: u64) -> Result<String> {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect();
    return Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NUM_ITERATION: i32 = 1000000;

    #[test]
    fn test_string() -> anyhow::Result<()> {
        let rand_string = string(5)?;
        assert_eq!(rand_string.chars().count(), 5);
        Ok(())
    }

    #[test]
    fn test_string_uniform() -> anyhow::Result<()> {
        use std::collections::HashSet;

        let mut result_str = HashSet::new();
        for _ in 0..=NUM_ITERATION {
            let new_str = string(16)?;
            assert_eq!(new_str.chars().count(), 16);
            if !result_str.insert(new_str){
                assert!(false);
            }
        }
        Ok(())
    }

    #[test]
    fn test_string_length() -> anyhow::Result<()> {
        for i in 0..=1000 {
            let new_str = string(i as u64)?;
            assert_eq!(new_str.chars().count(), i as usize);
        }
        Ok(())
    }
}