use anyhow::Result;
use rand::distributions::{Alphanumeric, DistString, Distribution, Uniform};
use rand_chacha::rand_core::SeedableRng;

pub fn string(length: u64, charset_opt: Option<String>) -> Result<String> {
    let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
    let res = match charset_opt {
        Some(charset) => {
            let strlen = charset.chars().count();
            let rand_dist = Uniform::from(0..strlen);
            let mut s = "".to_string();
            for _ in 0..length {
                let index = rand_dist.sample(&mut rng);
                s.push(charset.chars().nth(index).unwrap());
            }
            s
        }
        None => Alphanumeric.sample_string(&mut rng, length as usize),
    };

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NUM_ITERATION: i32 = 100;

    #[test]
    fn test_string() -> anyhow::Result<()> {
        let rand_string = string(5, None)?;
        assert_eq!(rand_string.chars().count(), 5);
        Ok(())
    }

    #[test]
    fn test_string_charset() -> anyhow::Result<()> {
        let rand_string = string(5, Some("a".to_string()))?;
        assert_eq!(rand_string.chars().count(), 5);
        assert_eq!(rand_string, "aaaaa");
        Ok(())
    }

    #[test]
    fn test_string_uniform() -> anyhow::Result<()> {
        use std::collections::HashSet;

        let mut result_str = HashSet::new();
        for _ in 0..=NUM_ITERATION {
            let new_str = string(16, None)?;
            assert_eq!(new_str.chars().count(), 16);
            assert!(result_str.insert(new_str), "test_string_uniform - failed");
        }
        Ok(())
    }

    #[test]
    fn test_string_length() -> anyhow::Result<()> {
        for i in [0, 1000, 8192000] {
            println!("Testing string length {}", i);
            let new_str = string(i as u64, None)?;
            assert_eq!(new_str.chars().count(), i as usize);
        }
        Ok(())
    }
}
