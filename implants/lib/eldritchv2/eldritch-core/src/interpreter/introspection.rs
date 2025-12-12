use super::super::ast::Value;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::None => false,
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Bytes(b) => !b.is_empty(),
        Value::List(l) => !l.read().is_empty(),
        Value::Dictionary(d) => !d.read().is_empty(),
        Value::Set(s) => !s.read().is_empty(),
        Value::Tuple(t) => !t.is_empty(),
        Value::Function(_)
        | Value::NativeFunction(_, _)
        | Value::NativeFunctionWithKwargs(_, _)
        | Value::BoundMethod(_, _)
        | Value::Foreign(_) => true,
    }
}

pub fn get_type_name(value: &Value) -> String {
    match value {
        Value::None => "NoneType".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Int(_) => "int".to_string(),
        Value::Float(_) => "float".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Bytes(_) => "bytes".to_string(),
        Value::List(_) => "list".to_string(),
        Value::Dictionary(_) => "dict".to_string(),
        Value::Set(_) => "set".to_string(),
        Value::Tuple(_) => "tuple".to_string(),
        Value::Function(_)
        | Value::NativeFunction(_, _)
        | Value::NativeFunctionWithKwargs(_, _)
        | Value::BoundMethod(_, _) => "function".to_string(),
        Value::Foreign(f) => f.type_name().to_string(),
    }
}

pub fn get_dir_attributes(value: &Value) -> Vec<String> {
    let mut attrs = match value {
        Value::List(_) => vec![
            "append".to_string(),
            "extend".to_string(),
            "index".to_string(),
            "insert".to_string(),
            "pop".to_string(),
            "remove".to_string(),
            "sort".to_string(),
        ],
        Value::Dictionary(_) => vec![
            "get".to_string(),
            "items".to_string(),
            "keys".to_string(),
            "popitem".to_string(),
            "update".to_string(),
            "values".to_string(),
        ],
        Value::Set(_) => vec![
            "add".to_string(),
            "clear".to_string(),
            "contains".to_string(), // not standard python but useful
            "difference".to_string(),
            "discard".to_string(),
            "intersection".to_string(),
            "isdisjoint".to_string(),
            "issubset".to_string(),
            "issuperset".to_string(),
            "pop".to_string(),
            "remove".to_string(),
            "symmetric_difference".to_string(),
            "union".to_string(),
            "update".to_string(),
        ],
        Value::String(_) => vec![
            "endswith".to_string(),
            "find".to_string(),
            "format".to_string(),
            "join".to_string(),
            "lower".to_string(),
            "replace".to_string(),
            "split".to_string(),
            "startswith".to_string(),
            "strip".to_string(),
            "upper".to_string(),
        ],
        Value::Foreign(f) => f.method_names(),
        _ => Vec::new(),
    };
    attrs.sort();
    attrs
}

// Basic Levenshtein distance implementation
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let m = s1_chars.len();
    let n = s2_chars.len();

    let mut dp = vec![vec![0; n + 1]; m + 1];

    for (i, item) in dp.iter_mut().enumerate().take(m + 1) {
        item[0] = i;
    }
    for (j, item) in dp[0].iter_mut().enumerate().take(n + 1) {
        *item = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = core::cmp::min(
                core::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1), // insertion, deletion
                dp[i - 1][j - 1] + cost,                            // substitution
            );
        }
    }

    dp[m][n]
}

pub fn find_best_match(target: &str, candidates: &[String]) -> Option<String> {
    let mut best_match: Option<String> = None;
    let mut min_dist = usize::MAX;

    // Threshold logic:
    // Allow a distance of up to 4, or half the string length + 1.
    // This allows "config" (6) -> "get_config" (10) (dist 4, threshold 4)
    // "apend" (5) -> "append" (6) (dist 1, threshold 3)
    let threshold = (target.len() / 2 + 1).clamp(1, 4);

    for candidate in candidates {
        // Optimization: Skip if lengths differ too much
        let len_diff = (candidate.len() as isize - target.len() as isize).unsigned_abs();
        if len_diff > threshold {
            continue;
        }

        let dist = levenshtein_distance(target, candidate);
        if dist <= threshold && dist < min_dist {
            min_dist = dist;
            best_match = Some(candidate.clone());
        }
    }

    best_match
}
