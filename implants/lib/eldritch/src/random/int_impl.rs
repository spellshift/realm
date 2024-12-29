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
    const NUM_ITERATION: i32 = 50000;
    const MIN_VALUE: i32 = 0;
    const MAX_VALUE: i32 = 1000;
    const CI_99_MIN: f32 = 496.675;
    const CI_99_MAX: f32 = 503.325;
    const CHI_SQUARED_EXPECTED: f32 = NUM_ITERATION as f32 / MAX_VALUE as f32;
    const CHI_SQUARED_MIN: f32 = 914.3;
    const CHI_SQUARED_MAX: f32 = 1090.0;

    #[test]
    fn test_random_int() -> anyhow::Result<()> {
        let random_number = int(MIN_VALUE, MAX_VALUE)?;
        assert!(random_number >= MIN_VALUE && random_number < MAX_VALUE);
        Ok(())
    }

    #[test]
    fn test_random_int_uniform_average() -> anyhow::Result<()> {
        let mut total = 0;
        for _ in 0..NUM_ITERATION {
            let random_number = int(MIN_VALUE, MAX_VALUE)?;
            total += random_number;
        }

        let avg = total as f32 / NUM_ITERATION as f32;

        assert!(
            avg >= CI_99_MIN && avg <= CI_99_MAX,
            "Average of {} Random Numbers not within 99% Confidence Interval",
            NUM_ITERATION
        );

        Ok(())
    }

    #[test]
    fn test_random_int_uniform_chi_square() -> anyhow::Result<()> {
        let mut counts = [0.0; MAX_VALUE as usize];
        for _ in 0..NUM_ITERATION {
            let random_number = int(MIN_VALUE, MAX_VALUE)?;
            counts[random_number as usize] += 1.0;
        }

        let mut chi_square = 0.0;

        for count in counts {
            chi_square += (count - CHI_SQUARED_EXPECTED).powf(2.0) / CHI_SQUARED_EXPECTED
        }
        assert!(
            chi_square >= CHI_SQUARED_MIN && chi_square <= CHI_SQUARED_MAX,
            "Chi-Squared Goodness of Fit Failed. {} not in interval ({}, {})",
            chi_square,
            CHI_SQUARED_MIN,
            CHI_SQUARED_MAX
        );
        Ok(())
    }
}
