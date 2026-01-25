use alloc::string::ToString;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;

pub fn int(min: i64, max: i64) -> Result<i64, String> {
    if min >= max {
        return Err("Invalid range".to_string());
    }
    let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
    Ok(rng.gen_range(min..max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int() {
        let val = int(0, 10).unwrap();
        assert!((0..10).contains(&val));
    }

    #[test]
    fn test_int_invalid_range() {
        let val = int(10, 5);
        assert!(val.is_err());
        assert_eq!(val.err().unwrap(), "Invalid range");
    }

    #[test]
    fn test_int_equal_range() {
        let val = int(10, 10);
        assert!(val.is_err());
        assert_eq!(val.err().unwrap(), "Invalid range");
    }
}
