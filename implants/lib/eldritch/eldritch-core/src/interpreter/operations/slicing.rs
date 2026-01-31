pub fn adjust_slice_indices(
    length: i64,
    start: &Option<i64>,
    stop: &Option<i64>,
    step: i64,
) -> (i64, i64) {
    let start_val = if let Some(s) = start {
        let mut s = *s;
        if s < 0 {
            s += length;
        }
        if step < 0 {
            if s >= length {
                length - 1
            } else if s < 0 {
                -1
            } else {
                s
            }
        } else if s < 0 {
            0
        } else if s > length {
            length
        } else {
            s
        }
    } else if step < 0 {
        length - 1
    } else {
        0
    };

    let stop_val = if let Some(s) = stop {
        let mut s = *s;
        if s < 0 {
            s += length;
        }
        if step < 0 {
            if s < -1 {
                -1
            } else if s >= length {
                length - 1
            } else {
                s
            }
        } else if s < 0 {
            0
        } else if s > length {
            length
        } else {
            s
        }
    } else if step < 0 {
        -1
    } else {
        length
    };

    (start_val, stop_val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjust_slice_indices_basic() {
        // len=10, [:]
        assert_eq!(adjust_slice_indices(10, &None, &None, 1), (0, 10));
        // len=10, [2:5]
        assert_eq!(adjust_slice_indices(10, &Some(2), &Some(5), 1), (2, 5));
        // len=10, [:5]
        assert_eq!(adjust_slice_indices(10, &None, &Some(5), 1), (0, 5));
        // len=10, [2:]
        assert_eq!(adjust_slice_indices(10, &Some(2), &None, 1), (2, 10));
    }

    #[test]
    fn test_adjust_slice_indices_negative_step() {
        // len=10, [::-1] -> start defaults to len-1 (9), stop defaults to -1
        assert_eq!(adjust_slice_indices(10, &None, &None, -1), (9, -1));
        // len=10, [5:2:-1]
        assert_eq!(adjust_slice_indices(10, &Some(5), &Some(2), -1), (5, 2));
    }

    #[test]
    fn test_adjust_slice_indices_negative_indices() {
        // len=10, [-5:] -> start=-5 -> 5
        assert_eq!(adjust_slice_indices(10, &Some(-5), &None, 1), (5, 10));
        // len=10, [:-2] -> stop=-2 -> 8
        assert_eq!(adjust_slice_indices(10, &None, &Some(-2), 1), (0, 8));
        // len=10, [-5:-2] -> 5, 8
        assert_eq!(adjust_slice_indices(10, &Some(-5), &Some(-2), 1), (5, 8));
    }

    #[test]
    fn test_adjust_slice_indices_out_of_bounds_positive_step() {
        // len=10, [-100:] -> start clamped to 0
        assert_eq!(adjust_slice_indices(10, &Some(-100), &None, 1), (0, 10));
        // len=10, [100:] -> start clamped to 10
        assert_eq!(adjust_slice_indices(10, &Some(100), &None, 1), (10, 10));
        // len=10, [:100] -> stop clamped to 10
        assert_eq!(adjust_slice_indices(10, &None, &Some(100), 1), (0, 10));
        // len=10, [:-100] -> stop clamped to 0
        assert_eq!(adjust_slice_indices(10, &None, &Some(-100), 1), (0, 0));
    }

    #[test]
    fn test_adjust_slice_indices_out_of_bounds_negative_step() {
        // len=10, [100::-1] -> start clamped to len-1 (9)
        assert_eq!(adjust_slice_indices(10, &Some(100), &None, -1), (9, -1));
        // len=10, [-100::-1] -> start clamped to -1 (empty slice)
        assert_eq!(adjust_slice_indices(10, &Some(-100), &None, -1), (-1, -1));

        // len=10, [: -100 : -1] -> stop clamped to -1
        // logic: if s < -1 { -1 }
        assert_eq!(adjust_slice_indices(10, &None, &Some(-100), -1), (9, -1));
    }

    #[test]
    fn test_adjust_slice_indices_complex_cases() {
        // Regression cases or complex combos
        // len=5, [::2]
        assert_eq!(adjust_slice_indices(5, &None, &None, 2), (0, 5));
        // len=5, [::-2]
        assert_eq!(adjust_slice_indices(5, &None, &None, -2), (4, -1));
    }
}
