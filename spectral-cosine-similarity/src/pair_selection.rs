use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::seq::index;

const DEFAULT_SEED: u64 = 0xDEAD_BEEF_CAFE_BABE;

fn flat_index_to_pair(ids: &[i32], k: usize) -> (i32, i32) {
    let mut row = (((1.0 + 8.0 * k as f64).sqrt() - 1.0) / 2.0) as usize;
    if (row + 1) * (row + 2) / 2 <= k {
        row += 1;
    }
    let col = k - row * (row + 1) / 2;
    (ids[col], ids[row])
}

/// Sample `num_pairs` unique random pairs from the pair space.
pub fn sample_pairs(ids: &[i32], num_pairs: usize) -> Vec<(i32, i32)> {
    let n = ids.len();
    let total_pairs = n * (n + 1) / 2;
    let clamped = num_pairs.min(total_pairs);

    let mut rng = SmallRng::seed_from_u64(DEFAULT_SEED);
    let sampled = index::sample(&mut rng, total_pairs, clamped);

    sampled
        .into_iter()
        .map(|k| flat_index_to_pair(ids, k))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn flat_index_to_pair_covers_full_triangle() {
        let ids = [1, 2, 3];
        let expected = [
            (1, 1), // k=0
            (1, 2), // k=1
            (2, 2), // k=2
            (1, 3), // k=3
            (2, 3), // k=4
            (3, 3), // k=5
        ];
        for (k, expected_pair) in expected.iter().enumerate() {
            let pair = flat_index_to_pair(&ids, k);
            assert_eq!(pair, *expected_pair, "mismatch at k={k}");
        }
    }

    #[test]
    fn sample_pairs_deterministic() {
        let ids: Vec<i32> = (1..=10).collect();
        let pairs1 = sample_pairs(&ids, 20);
        let pairs2 = sample_pairs(&ids, 20);
        assert_eq!(pairs1, pairs2);
    }

    #[test]
    fn sample_pairs_no_duplicates() {
        let ids: Vec<i32> = (1..=20).collect();
        let pairs = sample_pairs(&ids, 100);
        let unique: HashSet<_> = pairs.iter().copied().collect();
        assert_eq!(pairs.len(), unique.len());
    }

    #[test]
    fn sample_pairs_canonical_ordering() {
        let ids: Vec<i32> = (1..=10).collect();
        for (left, right) in sample_pairs(&ids, 30) {
            assert!(left <= right, "pair ({left}, {right}) violates left <= right");
        }
    }

    #[test]
    fn sample_pairs_clamps_to_total() {
        let ids = [1, 2, 3]; // 6 total pairs
        let pairs = sample_pairs(&ids, 100);
        assert_eq!(pairs.len(), 6);
    }
}
