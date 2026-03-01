from __future__ import annotations

import json
import sqlite3

import numpy as np
from matchms import Spectrum

from python_ref.types import ExperimentData
from python_ref.types import SpectrumData
from python_ref.types import WorkItem


def parse_peaks(peaks_json: str) -> tuple[np.ndarray, np.ndarray]:
    peaks = json.loads(peaks_json)
    mz = np.array([peak[0] for peak in peaks], dtype=np.float64)
    intensities = np.array([peak[1] for peak in peaks], dtype=np.float64)
    if mz.size > 1:
        order = np.argsort(mz, kind="stable")
        mz = mz[order]
        intensities = intensities[order]
    return mz, intensities


def build_spectrum(
    mz: np.ndarray, intensities: np.ndarray, precursor_mz: float
) -> Spectrum:
    return Spectrum(
        mz=mz, intensities=intensities, metadata={"precursor_mz": precursor_mz}
    )


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
        raise RuntimeError(f"Implementation '{algo_name}' in '{lib_name}' not found in DB")
    return int(row[0])


def load_experiments(cur: sqlite3.Cursor) -> list[ExperimentData]:
    cur.execute("SELECT id, params FROM experiments ORDER BY id ASC")
    return [ExperimentData(id=int(row[0]), params=json.loads(row[1])) for row in cur.fetchall()]


def load_spectra(cur: sqlite3.Cursor) -> dict[int, SpectrumData]:
    cur.execute(
        "SELECT id, peaks, precursor_mz, num_peaks FROM spectra ORDER BY id ASC"
    )
    spectra: dict[int, SpectrumData] = {}
    for row in cur.fetchall():
        spec_id, peaks_json, precursor_mz, _num_peaks = row
        mz, intensities = parse_peaks(peaks_json)
        spectra[int(spec_id)] = SpectrumData(
            spectrum=build_spectrum(mz, intensities, float(precursor_mz)),
            mz=mz,
            intensities=intensities,
        )
    return spectra


def get_existing_keys(cur: sqlite3.Cursor, implementation_id: int) -> set[tuple[int, int, int]]:
    cur.execute(
        """
        SELECT left_id, right_id, experiment_id
        FROM results
        WHERE implementation_id = ?
        ORDER BY left_id ASC, right_id ASC, experiment_id ASC
        """,
        (implementation_id,),
    )
    return {(int(row[0]), int(row[1]), int(row[2])) for row in cur.fetchall()}


def insert_result(
    cur: sqlite3.Cursor,
    item: WorkItem,
    implementation_id: int,
    score: float,
    matches: int,
    median_time_us: float,
) -> None:
    cur.execute(
        """
        INSERT INTO results
            (left_id, right_id, experiment_id, implementation_id, score, matches, median_time_us)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        (
            item.left_id,
            item.right_id,
            item.experiment.id,
            implementation_id,
            score,
            matches,
            median_time_us,
        ),
    )
