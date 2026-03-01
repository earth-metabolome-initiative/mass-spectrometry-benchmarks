"""Compute missing matchms similarities and timings in a single pass.

Reads experiments, spectra, and implementations from the SQLite DB,
generates spectrum pairs at runtime, computes matchms results for any
missing pairs, and writes back to the results table.

Usage: matchms_compute.py <db_path> [batch_size]
"""
import json
import sqlite3
import sys
import time

import numpy as np
from matchms import Spectrum
from matchms.similarity import CosineGreedy, CosineHungarian, ModifiedCosine
from tqdm import tqdm

DB_PATH = sys.argv[1] if len(sys.argv) > 1 else "fixtures/benchmark.db"
BATCH_SIZE = int(sys.argv[2]) if len(sys.argv) > 2 else None


def build_spectrum(peaks_json: str, precursor_mz: float) -> Spectrum:
    peaks = json.loads(peaks_json)
    mz = np.array([p[0] for p in peaks], dtype=np.float32).astype(np.float64)
    intensities = np.array([p[1] for p in peaks], dtype=np.float32).astype(np.float64)
    return Spectrum(mz=mz, intensities=intensities, metadata={"precursor_mz": precursor_mz})


def get_implementation_id(cur: sqlite3.Cursor, algo_name: str, lib_name: str) -> int:
    cur.execute(
        """
        SELECT i.id FROM implementations i
        JOIN algorithms a ON i.algorithm_id = a.id
        JOIN libraries l ON i.library_id = l.id
        WHERE a.name = ? AND l.name = ?
        """,
        (algo_name, lib_name),
    )
    row = cur.fetchone()
    if row is None:
        print(f"Implementation '{algo_name}' in '{lib_name}' not found in DB", file=sys.stderr)
        sys.exit(1)
    return row[0]


def load_experiments(cur: sqlite3.Cursor) -> list[dict]:
    cur.execute("SELECT id, params FROM experiments ORDER BY id ASC")
    return [{"id": r[0], "params": json.loads(r[1])} for r in cur.fetchall()]


def load_spectra(cur: sqlite3.Cursor) -> dict[int, tuple]:
    """Load all spectra from DB: id -> (Spectrum, num_peaks)."""
    cur.execute("SELECT id, peaks, precursor_mz, num_peaks FROM spectra ORDER BY id ASC")
    spectra = {}
    for row in cur.fetchall():
        spec_id, peaks_json, precursor_mz, num_peaks = row
        spectra[spec_id] = (build_spectrum(peaks_json, precursor_mz), num_peaks)
    return spectra


def generate_pairs(spectra: dict[int, tuple]) -> list[tuple[int, int]]:
    """Generate all spectrum ID pairs (including self-pairs)."""
    ids = list(spectra.keys())
    pairs = []
    for i, a in enumerate(ids):
        pairs.append((a, a))
        for b in ids[i + 1 :]:
            pairs.append((a, b))
    return pairs


def get_existing_keys(cur: sqlite3.Cursor, impl_id: int) -> set[tuple[int, int, int]]:
    """Get existing (left_id, right_id, experiment_id) for this implementation."""
    cur.execute(
        """
        SELECT left_id, right_id, experiment_id
        FROM results
        WHERE implementation_id = ?
        ORDER BY left_id ASC, right_id ASC, experiment_id ASC
        """,
        (impl_id,),
    )
    return {(r[0], r[1], r[2]) for r in cur.fetchall()}


def compute_algorithm(algo_name: str, algo_class):
    conn = sqlite3.connect(DB_PATH)
    cur = conn.cursor()

    impl_id = get_implementation_id(cur, algo_name, "matchms")
    experiments = load_experiments(cur)
    spectra = load_spectra(cur)
    id_pairs = generate_pairs(spectra)
    existing = get_existing_keys(cur, impl_id)

    # Work items: pairs not yet in results
    work = []
    for left_id, right_id in id_pairs:
        for exp in experiments:
            if (left_id, right_id, exp["id"]) not in existing:
                work.append((left_id, right_id, exp))

    if not work:
        return

    # Limit to batch size if specified
    if BATCH_SIZE is not None and len(work) > BATCH_SIZE:
        work = work[:BATCH_SIZE]

    uncommitted = 0
    for left_id, right_id, exp in tqdm(work, desc=f"[{algo_name}]", unit="pair", leave=False):
        left_spec = spectra[left_id][0]
        right_spec = spectra[right_id][0]
        params = exp["params"]
        n_warmup = params["n_warmup"]
        n_reps = params["n_reps"]

        cosine = algo_class(
            tolerance=params["tolerance"],
            mz_power=params["mz_power"],
            intensity_power=params["intensity_power"],
        )

        # Warmup
        for _ in range(n_warmup):
            cosine.pair(left_spec, right_spec)

        # Timed runs
        times = []
        for _ in range(n_reps):
            t0 = time.perf_counter_ns()
            result = cosine.pair(left_spec, right_spec)
            t1 = time.perf_counter_ns()
            times.append(t1 - t0)

        score = float(result["score"])
        matches = int(result["matches"])
        median_ns = sorted(times)[n_reps // 2]
        median_us = median_ns / 1000.0

        cur.execute(
            "INSERT INTO results (left_id, right_id, experiment_id, implementation_id, score, matches, median_time_us) VALUES (?, ?, ?, ?, ?, ?, ?)",
            (left_id, right_id, exp["id"], impl_id, score, matches, median_us),
        )

        uncommitted += 1
        if uncommitted >= 500:
            conn.commit()
            uncommitted = 0

    conn.commit()
    conn.close()


def main():
    compute_algorithm("CosineHungarian", CosineHungarian)
    compute_algorithm("CosineGreedy", CosineGreedy)
    compute_algorithm("ModifiedCosine", ModifiedCosine)


if __name__ == "__main__":
    main()
