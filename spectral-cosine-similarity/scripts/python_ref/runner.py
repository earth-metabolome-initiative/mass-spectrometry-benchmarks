from __future__ import annotations

import math
import sqlite3
import time

from tqdm import tqdm

from python_ref import db_io
from python_ref.types import ComputeFn, ExperimentData, SpectrumData

GLOBAL_WARMUP_PAIR_SAMPLE = 100


def _validate_score(
    score: float,
    *,
    algorithm_label: str,
    left_id: int,
    right_id: int,
    experiment_id: int,
) -> float:
    if not math.isfinite(score):
        raise RuntimeError(
            f"[python_ref] {algorithm_label}: non-finite score {score} for "
            f"(left_id={left_id}, right_id={right_id}, experiment_id={experiment_id})"
        )
    if score < 0.0 or score > 1.000001:
        raise RuntimeError(
            f"[python_ref] {algorithm_label}: score out of range for "
            f"(left_id={left_id}, right_id={right_id}, "
            f"experiment_id={experiment_id}): {score}"
        )
    if score > 1.0:
        score = 1.0
    return score


def _validate_matches(matches: int) -> int:
    if matches < -1:
        raise RuntimeError(
            f"[python_ref] invalid matches value {matches}; expected >= -1"
        )
    return matches


def run_algorithm(
    conn: sqlite3.Connection,
    algorithm_name: str,
    library_name: str,
    implementation_id: int,
    experiments: list[ExperimentData],
    spectra: dict[int, SpectrumData],
    id_pairs: list[tuple[int, int]],
    compute_once: ComputeFn,
) -> int:
    cur = conn.cursor()

    total_work = len(id_pairs) * len(experiments)
    if total_work == 0:
        return 0

    algo_label = f"{algorithm_name} ({library_name})"

    # Per-implementation skip check.
    cur.execute(
        "SELECT COUNT(*) FROM results WHERE implementation_id = ?",
        (implementation_id,),
    )
    existing = cur.fetchone()[0]
    if existing == total_work:
        print(f"[python_ref] {algo_label}: {existing} results cached, skipping")
        return 0
    if existing > 0:
        print(
            f"[python_ref] {algo_label}: partial results ({existing}/{total_work}), "
            "clearing and recomputing"
        )
        cur.execute(
            "DELETE FROM results WHERE implementation_id = ?",
            (implementation_id,),
        )
        conn.commit()

    with tqdm(total=total_work, desc=algo_label, unit="pair") as bar:
        for experiment in experiments:
            params = experiment.params
            n_warmup = int(params["n_warmup"])
            n_reps = int(params["n_reps"])

            # Warmup once per (implementation, experiment) using a representative subset.
            warmup_pairs = id_pairs[:GLOBAL_WARMUP_PAIR_SAMPLE]
            for _ in range(n_warmup):
                for left_id, right_id in warmup_pairs:
                    left_spec = spectra[left_id]
                    right_spec = spectra[right_id]
                    compute_once(left_spec, right_spec, params)

            for left_id, right_id in id_pairs:
                left_spec = spectra[left_id]
                right_spec = spectra[right_id]

                times: list[int] = []
                score = 0.0
                matches = 0
                for _ in range(n_reps):
                    t0 = time.perf_counter_ns()
                    raw_score, raw_matches = compute_once(left_spec, right_spec, params)
                    score = _validate_score(
                        raw_score,
                        algorithm_label=algo_label,
                        left_id=left_id,
                        right_id=right_id,
                        experiment_id=experiment.id,
                    )
                    matches = _validate_matches(raw_matches)
                    t1 = time.perf_counter_ns()
                    times.append(t1 - t0)

                median_ns = sorted(times)[n_reps // 2]
                median_us = median_ns / 1000.0

                db_io.insert_result(
                    cur=cur,
                    left_id=left_id,
                    right_id=right_id,
                    experiment_id=experiment.id,
                    implementation_id=implementation_id,
                    score=score,
                    matches=matches,
                    median_time_us=median_us,
                )
                bar.update(1)

    conn.commit()
    return total_work
