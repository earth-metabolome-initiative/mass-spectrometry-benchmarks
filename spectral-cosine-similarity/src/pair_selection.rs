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
