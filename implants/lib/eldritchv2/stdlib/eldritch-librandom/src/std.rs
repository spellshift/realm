use super::RandomLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;
use rand::Rng;
use rand::distributions::{Alphanumeric, DistString, Distribution, Uniform};
use rand_chacha::rand_core::SeedableRng;

#[derive(Default, Debug)]
#[eldritch_library_impl(RandomLibrary)]
pub struct StdRandomLibrary;

impl RandomLibrary for StdRandomLibrary {
    fn bool(&self) -> Result<bool, String> {
        let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
        Ok(rng.r#gen::<bool>())
    }

    fn bytes(&self, len: i64) -> Result<Vec<u8>, String> {
        if len < 0 {
            return Err("Length cannot be negative".to_string());
        }
        let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
        let mut bytes = vec![0u8; len as usize];
        rng.fill(&mut bytes[..]);
        Ok(bytes)
    }

    fn int(&self, min: i64, max: i64) -> Result<i64, String> {
        if min >= max {
            return Err("Invalid range".to_string());
        }
        let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
        Ok(rng.gen_range(min..max))
    }

    fn string(&self, len: i64, charset: Option<String>) -> Result<String, String> {
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

    fn uuid(&self) -> Result<String, String> {
        Ok(uuid::Uuid::new_v4().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NUM_ITERATION: i32 = 1000;

    #[test]
    fn test_bool() {
        let lib = StdRandomLibrary;
        assert!(lib.bool().is_ok());
    }

    #[test]
    fn test_bool_uniform() {
        let lib = StdRandomLibrary;
        let mut num_true = 0;
        for _ in 0..NUM_ITERATION {
            let b = lib.bool().unwrap();
            if b {
                num_true += 1;
            }
        }

        let lower_bound = 0.40 * NUM_ITERATION as f64;
        let upper_bound = 0.60 * NUM_ITERATION as f64;
        let high_enough = lower_bound < num_true as f64;
        let low_enough = upper_bound > num_true as f64;
        assert!(
            high_enough && low_enough,
            "{num_true} was not between the acceptable bounds of ({lower_bound},{upper_bound})"
        );
    }

    #[test]
    fn test_bytes() {
        let lib = StdRandomLibrary;
        let b = lib.bytes(10).unwrap();
        assert_eq!(b.len(), 10);
    }

    #[test]
    fn test_bytes_negative() {
        let lib = StdRandomLibrary;
        let b = lib.bytes(-1);
        assert!(b.is_err());
        assert_eq!(b.err().unwrap(), "Length cannot be negative");
    }

    #[test]
    fn test_int() {
        let lib = StdRandomLibrary;
        let val = lib.int(0, 10).unwrap();
        assert!((0..10).contains(&val));
    }

    #[test]
    fn test_int_invalid_range() {
        let lib = StdRandomLibrary;
        let val = lib.int(10, 5);
        assert!(val.is_err());
        assert_eq!(val.err().unwrap(), "Invalid range");
    }

    #[test]
    fn test_int_equal_range() {
        let lib = StdRandomLibrary;
        let val = lib.int(10, 10);
        assert!(val.is_err());
        assert_eq!(val.err().unwrap(), "Invalid range");
    }

    #[test]
    fn test_string() {
        let lib = StdRandomLibrary;
        let s = lib.string(10, None).unwrap();
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_string_negative() {
        let lib = StdRandomLibrary;
        let s = lib.string(-1, None);
        assert!(s.is_err());
        assert_eq!(s.err().unwrap(), "Length cannot be negative");
    }

    #[test]
    fn test_string_charset() {
        let lib = StdRandomLibrary;
        let s = lib.string(5, Some("a".to_string())).unwrap();
        assert_eq!(s, "aaaaa");
    }

    #[test]
    fn test_string_charset_empty() {
        let lib = StdRandomLibrary;
        let s = lib.string(5, Some("".to_string()));
        assert!(s.is_err());
        assert_eq!(s.err().unwrap(), "Charset cannot be empty");
    }

    #[test]
    fn test_string_charset_unicode() {
        let lib = StdRandomLibrary;
        let charset = "🦀";
        let s = lib.string(5, Some(charset.to_string())).unwrap();
        assert_eq!(s.chars().count(), 5);
        assert!(s.chars().all(|c| c == '🦀'));
    }

    #[test]
    fn test_uuid() {
        let lib = StdRandomLibrary;
        let u = lib.uuid().unwrap();
        assert!(uuid::Uuid::parse_str(&u).is_ok());
    }
}
