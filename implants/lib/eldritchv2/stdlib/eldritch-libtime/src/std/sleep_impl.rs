use std::{thread, time};

pub fn sleep(secs: i64) -> Result<(), String> {
    thread::sleep(time::Duration::from_secs(secs as u64));
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
        lib.sleep(1).unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed.as_secs() >= 1);
    }
}
