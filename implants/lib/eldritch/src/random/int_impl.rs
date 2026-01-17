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

    /*
     * Statistics of a Uniform Distribution from a to b
     * mean = (a+b)/2
     * standard deviation = sqrt(((b-a)^2)/12)
     * a = 0, b = 1000
     * mean = 500, std dev = 288.675
     * 99% Confidence Interval where n = 50000 = (496.675, 503.325)
     */
    const MIN_VALUE: i32 = 0;
    const MAX_VALUE: i32 = 1000;

    #[test]
    fn test_random_int() -> anyhow::Result<()> {
        for _ in 0..5 {
            let random_number = int(MIN_VALUE, MAX_VALUE)?;
            assert!((MIN_VALUE..MAX_VALUE).contains(&random_number));
        }

        Ok(())
    }
}
