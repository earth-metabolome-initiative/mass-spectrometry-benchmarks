from __future__ import annotations

import math
import sqlite3
import time

from tqdm import tqdm

from python_ref import db_io
from python_ref.types import ComputeFn
from python_ref.types import ExperimentData
from python_ref.types import SpectrumData
from python_ref.types import WorkItem

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

    work: list[WorkItem] = []
    for left_id, right_id in id_pairs:
        for experiment in experiments:
            work.append(
                WorkItem(left_id=left_id, right_id=right_id, experiment=experiment)
            )

    if not work:
        return 0

    work_by_experiment: dict[int, list] = {experiment.id: [] for experiment in experiments}
    for item in work:
        work_by_experiment.setdefault(item.experiment.id, []).append(item)

    uncommitted = 0
    algo_label = f"{algorithm_name} ({library_name})"
    progress = tqdm(total=len(work), desc=f"[{algo_label}]", unit="pair", leave=False)
    try:
        for experiment in experiments:
            exp_work = work_by_experiment.get(experiment.id, [])
            if not exp_work:
                continue

            params = experiment.params
            n_warmup = int(params["n_warmup"])
            n_reps = int(params["n_reps"])

            # Warmup once per (implementation, experiment) using a representative subset.
            warmup_items = exp_work[:GLOBAL_WARMUP_PAIR_SAMPLE]
            for _ in range(n_warmup):
                for item in warmup_items:
                    left_spec = spectra[item.left_id]
                    right_spec = spectra[item.right_id]
                    compute_once(left_spec, right_spec, params)

            for item in exp_work:
                left_spec = spectra[item.left_id]
                right_spec = spectra[item.right_id]

                times: list[int] = []
                score = 0.0
                matches = 0
                for _ in range(n_reps):
                    t0 = time.perf_counter_ns()
                    raw_score, raw_matches = compute_once(left_spec, right_spec, params)
                    score = _validate_score(
                        raw_score,
                        algorithm_label=algo_label,
                        left_id=item.left_id,
                        right_id=item.right_id,
                        experiment_id=item.experiment.id,
                    )
                    matches = _validate_matches(raw_matches)
                    t1 = time.perf_counter_ns()
                    times.append(t1 - t0)

                median_ns = sorted(times)[n_reps // 2]
                median_us = median_ns / 1000.0

                db_io.insert_result(
                    cur=cur,
                    item=item,
                    implementation_id=implementation_id,
                    score=score,
                    matches=matches,
                    median_time_us=median_us,
                )

                uncommitted += 1
                progress.update(1)
                if uncommitted >= COMMIT_INTERVAL:
                    conn.commit()
                    uncommitted = 0
    finally:
        progress.close()

    conn.commit()
    return len(work)
