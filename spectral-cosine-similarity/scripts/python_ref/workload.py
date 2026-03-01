from __future__ import annotations

from python_ref.types import ExperimentData
from python_ref.types import WorkItem


def generate_pairs(spectrum_ids: list[int]) -> list[tuple[int, int]]:
    pairs: list[tuple[int, int]] = []
    for index, left_id in enumerate(spectrum_ids):
        pairs.append((left_id, left_id))
        for right_id in spectrum_ids[index + 1 :]:
            pairs.append((left_id, right_id))
    return pairs


def select_missing_work(
    id_pairs: list[tuple[int, int]],
    experiments: list[ExperimentData],
    existing: set[tuple[int, int, int]],
    batch_size: int | None,
) -> list[WorkItem]:
    work: list[WorkItem] = []
    for left_id, right_id in id_pairs:
        for experiment in experiments:
            if (left_id, right_id, experiment.id) in existing:
                continue
            work.append(
                WorkItem(left_id=left_id, right_id=right_id, experiment=experiment)
            )
            if batch_size is not None and len(work) >= batch_size:
                return work
    return work
