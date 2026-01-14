use anyhow::Result;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;

pub fn bool() -> Result<bool> {
    let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
    Ok(rng.gen::<bool>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() -> anyhow::Result<()> {
        bool()?;
        Ok(())
    }
}
