use std::{thread, time};

pub fn sleep(secs: f64) -> Result<(), String> {
    if secs < 0.0 {
        return Err("sleep length must be non-negative".to_string());
    }
    if !secs.is_finite() {
        return Err("sleep length must be a finite number".to_string());
    }
    thread::sleep(time::Duration::from_secs_f64(secs));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::StdTimeLibrary;
    use super::super::TimeLibrary;

    #[test]
    fn test_sleep() {
        let lib = StdTimeLibrary;
        let start = std::time::Instant::now();
        // Use a small sleep to avoid making tests slow
        lib.sleep(1.0).unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed.as_secs() >= 1);
    }

    #[test]
    fn test_sleep_fractional() {
        let lib = StdTimeLibrary;
        let start = std::time::Instant::now();
        lib.sleep(0.1).unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() >= 100);
    }
}
