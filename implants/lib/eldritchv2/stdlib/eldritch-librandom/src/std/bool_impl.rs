use rand::Rng;
use rand_chacha::rand_core::SeedableRng;

pub fn bool() -> Result<bool, String> {
    let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
    Ok(rng.r#gen::<bool>())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NUM_ITERATION: i32 = 1000;

    #[test]
    fn test_bool() {
        assert!(bool().is_ok());
    }

    #[test]
    fn test_bool_uniform() {
        let mut num_true = 0;
        for _ in 0..NUM_ITERATION {
            let b = bool().unwrap();
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
}
