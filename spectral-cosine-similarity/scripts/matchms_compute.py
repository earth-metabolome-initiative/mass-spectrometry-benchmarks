"""Compute missing Python similarities and timings in a single pass.

Reads experiments, spectra, and implementations from the SQLite DB,
generates spectrum pairs at runtime, computes Python-reference results for any
missing pairs, and writes back to the results table.

Usage: matchms_compute.py <db_path> [batch_size]
"""

import json
import sqlite3
import sys
import time
from dataclasses import dataclass

import numpy as np
from tqdm import tqdm

try:
    from matchms import Spectrum
    from matchms.similarity import CosineGreedy, CosineHungarian, ModifiedCosine
except Exception as exc:  # pragma: no cover - runtime dependency guard
    print(
        "ERROR: matchms is required for benchmarking. Run `uv sync` in "
        "spectral-cosine-similarity/.",
        file=sys.stderr,
    )
    print(f"Import error: {exc}", file=sys.stderr)
    raise SystemExit(2)

try:
    import ms_entropy  # type: ignore
except Exception as exc:  # pragma: no cover - runtime dependency guard
    print(
        "ERROR: ms_entropy is required for entropy benchmarking. Run `uv sync` "
        "in spectral-cosine-similarity/.",
        file=sys.stderr,
    )
    print(f"Import error: {exc}", file=sys.stderr)
    raise SystemExit(2)

DB_PATH = sys.argv[1] if len(sys.argv) > 1 else "fixtures/benchmark.db"
BATCH_SIZE = int(sys.argv[2]) if len(sys.argv) > 2 else None


@dataclass
class SpectrumData:
    spectrum: Spectrum
    mz: np.ndarray
    intensities: np.ndarray


def parse_peaks(peaks_json: str) -> tuple[np.ndarray, np.ndarray]:
    peaks = json.loads(peaks_json)
    mz = np.array([p[0] for p in peaks], dtype=np.float64)
    intensities = np.array([p[1] for p in peaks], dtype=np.float64)
    if mz.size > 1:
        order = np.argsort(mz, kind="stable")
        mz = mz[order]
        intensities = intensities[order]
    return mz, intensities


def build_spectrum(mz: np.ndarray, intensities: np.ndarray, precursor_mz: float) -> Spectrum:
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


def load_spectra(cur: sqlite3.Cursor) -> dict[int, SpectrumData]:
    """Load all spectra from DB."""
    cur.execute("SELECT id, peaks, precursor_mz, num_peaks FROM spectra ORDER BY id ASC")
    spectra: dict[int, SpectrumData] = {}
    for row in cur.fetchall():
        spec_id, peaks_json, precursor_mz, _num_peaks = row
        mz, intensities = parse_peaks(peaks_json)
        spectra[spec_id] = SpectrumData(
            spectrum=build_spectrum(mz, intensities, precursor_mz),
            mz=mz,
            intensities=intensities,
        )
    return spectra


def generate_pairs(spectra: dict[int, SpectrumData]) -> list[tuple[int, int]]:
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


def compute_matchms_algorithm(algo_name: str, algo_class):
    conn = sqlite3.connect(DB_PATH)
    cur = conn.cursor()

    impl_id = get_implementation_id(cur, algo_name, "matchms")
    experiments = load_experiments(cur)
    spectra = load_spectra(cur)
    id_pairs = generate_pairs(spectra)
    existing = get_existing_keys(cur, impl_id)

    work = []
    for left_id, right_id in id_pairs:
        for exp in experiments:
            if (left_id, right_id, exp["id"]) not in existing:
                work.append((left_id, right_id, exp))

    if not work:
        return

    if BATCH_SIZE is not None and len(work) > BATCH_SIZE:
        work = work[:BATCH_SIZE]

    uncommitted = 0
    for left_id, right_id, exp in tqdm(work, desc=f"[{algo_name}]", unit="pair", leave=False):
        left_spec = spectra[left_id].spectrum
        right_spec = spectra[right_id].spectrum
        params = exp["params"]
        n_warmup = params["n_warmup"]
        n_reps = params["n_reps"]

        scorer = algo_class(
            tolerance=params["tolerance"],
            mz_power=params["mz_power"],
            intensity_power=params["intensity_power"],
        )

        for _ in range(n_warmup):
            scorer.pair(left_spec, right_spec)

        times = []
        for _ in range(n_reps):
            t0 = time.perf_counter_ns()
            result = scorer.pair(left_spec, right_spec)
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


def entropy_similarity_ms_entropy(
    left: SpectrumData, right: SpectrumData, tolerance: float, weighted: bool
) -> float | None:
    peaks_a = np.column_stack((left.mz, left.intensities))
    peaks_b = np.column_stack((right.mz, right.intensities))

    def try_call(func, **kwargs):
        try:
            return float(func(**kwargs))
        except TypeError:
            return None
        except Exception:
            return None

    if weighted:
        func = getattr(ms_entropy, "calculate_entropy_similarity", None)
        if func is None:
            return None
        for kwargs in (
            {
                "spectrum_a": peaks_a,
                "spectrum_b": peaks_b,
                "ms2_tolerance_in_da": tolerance,
                "clean_spectra": False,
            },
            {
                "spectrum_a": peaks_a,
                "spectrum_b": peaks_b,
                "ms2_tolerance": tolerance,
                "clean_spectra": False,
            },
            {
                "spectra_a": peaks_a,
                "spectra_b": peaks_b,
                "ms2_tolerance_in_da": tolerance,
                "clean_spectra": False,
            },
        ):
            result = try_call(func, **kwargs)
            if result is not None:
                return result
        for args in ((peaks_a, peaks_b), (peaks_a, peaks_b, tolerance)):
            try:
                return float(func(*args))
            except Exception:
                pass
    else:
        func = getattr(ms_entropy, "calculate_unweighted_entropy_similarity", None)
        if func is not None:
            for kwargs in (
                {
                    "spectrum_a": peaks_a,
                    "spectrum_b": peaks_b,
                    "ms2_tolerance_in_da": tolerance,
                    "clean_spectra": False,
                },
                {
                    "spectrum_a": peaks_a,
                    "spectrum_b": peaks_b,
                    "ms2_tolerance": tolerance,
                    "clean_spectra": False,
                },
            ):
                result = try_call(func, **kwargs)
                if result is not None:
                    return result
            for args in ((peaks_a, peaks_b), (peaks_a, peaks_b, tolerance)):
                try:
                    return float(func(*args))
                except Exception:
                    pass

        func = getattr(ms_entropy, "calculate_entropy_similarity", None)
        if func is not None:
            for kwargs in (
                {
                    "spectrum_a": peaks_a,
                    "spectrum_b": peaks_b,
                    "ms2_tolerance_in_da": tolerance,
                    "clean_spectra": False,
                    "weighted": False,
                },
                {
                    "spectrum_a": peaks_a,
                    "spectrum_b": peaks_b,
                    "ms2_tolerance_in_da": tolerance,
                    "clean_spectra": False,
                    "use_weighted_intensity": False,
                },
            ):
                result = try_call(func, **kwargs)
                if result is not None:
                    return result

    return None


def compute_entropy_algorithm(algo_name: str, weighted: bool):
    conn = sqlite3.connect(DB_PATH)
    cur = conn.cursor()

    impl_id = get_implementation_id(cur, algo_name, "ms_entropy")
    experiments = load_experiments(cur)
    spectra = load_spectra(cur)
    id_pairs = generate_pairs(spectra)
    existing = get_existing_keys(cur, impl_id)

    work = []
    for left_id, right_id in id_pairs:
        for exp in experiments:
            if (left_id, right_id, exp["id"]) not in existing:
                work.append((left_id, right_id, exp))

    if not work:
        return

    if BATCH_SIZE is not None and len(work) > BATCH_SIZE:
        work = work[:BATCH_SIZE]

    uncommitted = 0
    for left_id, right_id, exp in tqdm(work, desc=f"[{algo_name}]", unit="pair", leave=False):
        left_spec = spectra[left_id]
        right_spec = spectra[right_id]
        params = exp["params"]
        n_warmup = params["n_warmup"]
        n_reps = params["n_reps"]
        tolerance = float(params["tolerance"])

        for _ in range(n_warmup):
            score = entropy_similarity_ms_entropy(left_spec, right_spec, tolerance, weighted)
            if score is None:
                raise RuntimeError(
                    "Unsupported ms_entropy API for entropy similarity. "
                    "Please install a compatible ms_entropy version."
                )

        times = []
        score = 0.0
        for _ in range(n_reps):
            t0 = time.perf_counter_ns()
            maybe_score = entropy_similarity_ms_entropy(left_spec, right_spec, tolerance, weighted)
            if maybe_score is None:
                raise RuntimeError(
                    "Unsupported ms_entropy API for entropy similarity. "
                    "Please install a compatible ms_entropy version."
                )
            score = maybe_score
            t1 = time.perf_counter_ns()
            times.append(t1 - t0)

        median_ns = sorted(times)[n_reps // 2]
        median_us = median_ns / 1000.0

        # Entropy parity is score-only; store a sentinel for matches.
        cur.execute(
            "INSERT INTO results (left_id, right_id, experiment_id, implementation_id, score, matches, median_time_us) VALUES (?, ?, ?, ?, ?, ?, ?)",
            (left_id, right_id, exp["id"], impl_id, score, -1, median_us),
        )

        uncommitted += 1
        if uncommitted >= 500:
            conn.commit()
            uncommitted = 0

    conn.commit()
    conn.close()


def main():
    compute_matchms_algorithm("CosineHungarian", CosineHungarian)
    compute_matchms_algorithm("CosineGreedy", CosineGreedy)
    compute_matchms_algorithm("ModifiedCosine", ModifiedCosine)
    compute_entropy_algorithm("EntropySimilarityWeighted", weighted=True)
    compute_entropy_algorithm("EntropySimilarityUnweighted", weighted=False)


if __name__ == "__main__":
    main()
