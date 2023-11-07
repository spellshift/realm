pub fn sleep(seconds: f64) {
    std::thread::sleep(std::time::Duration::from_secs_f64(seconds));
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn test_sleep() {
        let before = Local::now();
        sleep(5.0);
        let after = Local::now();
        assert!((after.signed_duration_since(before).num_milliseconds() - 5000).abs() <= 250);
    }
}