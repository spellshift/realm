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
