from __future__ import annotations

import math
import sqlite3
import sys
import time

from python_ref import db_io
from python_ref.types import ComputeFn
from python_ref.types import ExperimentData
from python_ref.types import SpectrumData

COMMIT_INTERVAL = 500
GLOBAL_WARMUP_PAIR_SAMPLE = 100


def _validate_score(
    score: object,
    *,
    algorithm_label: str,
    left_id: int,
    right_id: int,
    experiment_id: int,
) -> float:
    score = float(score)
    if not math.isfinite(score):
        raise RuntimeError(
            f"[python_ref] {algorithm_label}: non-finite score {score} for "
            f"(left_id={left_id}, right_id={right_id}, experiment_id={experiment_id})"
        )
    if score < 0.0 or score > 1.000001:
        raise RuntimeError(
            f"[python_ref] {algorithm_label}: score out of range for "
            f"(left_id={left_id}, right_id={right_id}, experiment_id={experiment_id}): {score}"
        )
    if score > 1.0:
        score = 1.0
    return score


def _validate_matches(matches: object) -> int:
    matches = int(matches)
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

    uncommitted = 0
    total_done = 0
    algo_label = f"{algorithm_name} ({library_name})"
    print(f"[{algo_label}] 0/{total_work}", file=sys.stderr, end="\r")

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

            uncommitted += 1
            total_done += 1
            if uncommitted >= COMMIT_INTERVAL:
                conn.commit()
                uncommitted = 0

            if total_done == total_work or total_done % 5000 == 0:
                print(f"[{algo_label}] {total_done}/{total_work}", file=sys.stderr, end="\r")

    conn.commit()
    print(f"[{algo_label}] {total_done}/{total_work} done", file=sys.stderr)
    return total_work
