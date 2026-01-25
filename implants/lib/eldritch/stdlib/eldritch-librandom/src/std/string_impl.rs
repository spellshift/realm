use alloc::string::{String, ToString};
use rand::distributions::{Alphanumeric, DistString, Distribution, Uniform};
use rand_chacha::rand_core::SeedableRng;

pub fn string(len: i64, charset: Option<String>) -> Result<String, String> {
    if len < 0 {
        return Err("Length cannot be negative".to_string());
    }
    let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
    let res = match charset {
        Some(charset) => {
            let strlen = charset.chars().count();
            if strlen == 0 {
                return Err("Charset cannot be empty".to_string());
            }
            let rand_dist = Uniform::from(0..strlen);
            let mut s = String::new();
            for _ in 0..len {
                let index = rand_dist.sample(&mut rng);
                s.push(charset.chars().nth(index).unwrap());
            }
            s
        }
        None => Alphanumeric.sample_string(&mut rng, len as usize),
    };

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let s = string(10, None).unwrap();
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_string_negative() {
        let s = string(-1, None);
        assert!(s.is_err());
        assert_eq!(s.err().unwrap(), "Length cannot be negative");
    }

    #[test]
    fn test_string_charset() {
        let s = string(5, Some("a".to_string())).unwrap();
        assert_eq!(s, "aaaaa");
    }

    #[test]
    fn test_string_charset_empty() {
        let s = string(5, Some("".to_string()));
        assert!(s.is_err());
        assert_eq!(s.err().unwrap(), "Charset cannot be empty");
    }

    #[test]
    fn test_string_charset_unicode() {
        let charset = "🦀";
        let s = string(5, Some(charset.to_string())).unwrap();
        assert_eq!(s.chars().count(), 5);
        assert!(s.chars().all(|c| c == '🦀'));
    }
}
