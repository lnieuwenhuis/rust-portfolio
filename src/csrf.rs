use crate::error::AppError;

pub fn validate(expected: &str, received: &str) -> Result<(), AppError> {
    if constant_time_eq(expected.as_bytes(), received.as_bytes()) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.iter()
        .zip(right.iter())
        .fold(0_u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn token_comparison_matches_equal_values_only() {
        assert!(constant_time_eq(b"same", b"same"));
        assert!(!constant_time_eq(b"same", b"nope"));
        assert!(!constant_time_eq(b"same", b"same-but-longer"));
    }
}
