/// Generate all spectrum ID pairs (including self-pairs).
pub fn generate_pairs(ids: &[i32]) -> Vec<(i32, i32)> {
    let mut pairs = Vec::with_capacity(ids.len() * (ids.len() + 1) / 2);

    for (i, &a) in ids.iter().enumerate() {
        // Self-pair
        pairs.push((a, a));
        // Cross-pairs
        for &b in &ids[i + 1..] {
            pairs.push((a, b));
        }
    }

    pairs
}

#[cfg(test)]
mod tests {
    use super::generate_pairs;

    #[test]
    fn handles_empty_and_singleton_inputs() {
        assert!(generate_pairs(&[]).is_empty());
        assert_eq!(generate_pairs(&[7]), vec![(7, 7)]);
    }

    #[test]
    fn generates_expected_pairs_without_mirrored_duplicates() {
        let ids = [10, 20, 30];
        let pairs = generate_pairs(&ids);

        assert_eq!(
            pairs,
            vec![(10, 10), (10, 20), (10, 30), (20, 20), (20, 30), (30, 30)]
        );
        assert_eq!(pairs.len(), ids.len() * (ids.len() + 1) / 2);
        assert!(!pairs.contains(&(20, 10)));
        assert!(!pairs.contains(&(30, 10)));
    }
}
