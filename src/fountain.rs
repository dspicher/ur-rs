#[must_use]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
pub fn fragment_length(data_length: usize, max_fragment_length: usize) -> usize {
    let fragment_count = data_length / max_fragment_length + 1;
    (data_length as f64 / fragment_count as f64).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_length() {
        assert_eq!(fragment_length(12345, 1955), 1764);
        assert_eq!(fragment_length(12345, 30000), 12345);
    }
}
