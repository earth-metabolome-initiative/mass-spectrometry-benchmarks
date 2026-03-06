from __future__ import annotations

import math
import random

DEFAULT_SEED: int = 0xDEAD_BEEF_CAFE_BABE


def generate_pairs(spectrum_ids: list[int]) -> list[tuple[int, int]]:
    pairs: list[tuple[int, int]] = []
    for index, left_id in enumerate(spectrum_ids):
        pairs.append((left_id, left_id))
        for right_id in spectrum_ids[index + 1 :]:
            pairs.append((left_id, right_id))
    return pairs


def _index_to_pair(
    ids: list[int], k: int
) -> tuple[int, int]:
    row = (math.isqrt(1 + 8 * k) - 1) // 2
    col = k - row * (row + 1) // 2
    return ids[col], ids[row]


def sample_pairs(
    spectrum_ids: list[int],
    num_pairs: int,
    seed: int = DEFAULT_SEED,
) -> list[tuple[int, int]]:
    n = len(spectrum_ids)
    total_pairs = n * (n + 1) // 2
    clamped = min(num_pairs, total_pairs)
    rng = random.Random(seed)
    indices = rng.sample(range(total_pairs), clamped)
    return [_index_to_pair(spectrum_ids, k) for k in indices]
