use anyhow::Result;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;

pub fn int(min: i32, max: i32) -> Result<i32> {
    if min >= max {
        return Err(anyhow::anyhow!("Invalid range"));
    }
    let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
    Ok(rng.gen_range(min..max))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NUM_ITERATION: i32 = 1000;
    const MIN_VALUE: i32 = 0;
    const MAX_VALUE: i32 = 100;

    #[test]
    fn test_random_int() -> anyhow::Result<()> {
        let random_number = random_int(MIN_VALUE, MAX_VALUE)?;
        assert!(random_number >= MIN_VALUE && random_number < MAX_VALUE);
        Ok(())
    }

    #[test]
    fn test_random_int_uniform() -> anyhow::Result<()> {
        let mut counts = vec![0; MAX_VALUE as usize];
        for _ in 0..NUM_ITERATION {
            let random_number = random_int(MIN_VALUE, MAX_VALUE)?;
            counts[random_number as usize] += 1;
        }

        let lower_bound = 0.90 * NUM_ITERATION as f64 / MAX_VALUE as f64;
        let upper_bound = 1.10 * NUM_ITERATION as f64 / MAX_VALUE as f64;
        
        for count in counts {
            assert!(
                count as f64 >= lower_bound && count as f64 <= upper_bound,
                "Count {} is not within the acceptable bounds of ({},{})",
                count,
                lower_bound,
                upper_bound
            );
        }
        Ok(())
    }
}
