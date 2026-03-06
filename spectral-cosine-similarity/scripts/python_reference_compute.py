"""Compute missing Python reference similarities and timings in a single pass.

Reads experiments, spectra, and selected pairs from the SQLite DB, computes
Python-reference results for any missing pairs, and writes back to the results table.

Usage:
  python_reference_compute.py <db_path> [--algorithm <name>]
"""

from __future__ import annotations

import argparse
import sqlite3

from python_ref import db_io
from python_ref import runner
from python_ref.algorithms import ALGORITHMS


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Python reference compute")
    parser.add_argument("db_path", nargs="?", default="fixtures/benchmark.db")
    parser.add_argument("--algorithm", default=None, help="Run only this algorithm")
    return parser.parse_args()


def selected_algorithms(algorithm_name: str | None):
    if algorithm_name is None:
        return ALGORITHMS

    selected = [entry for entry in ALGORITHMS if entry[0] == algorithm_name]
    if selected:
        return selected

    valid = ", ".join(name for name, _, _ in ALGORITHMS)
    raise SystemExit(
        f"unknown --algorithm '{algorithm_name}'. Valid algorithms: {valid}"
    )


def main() -> None:
    args = parse_args()
    conn = sqlite3.connect(args.db_path)
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA synchronous = NORMAL")
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute("PRAGMA ignore_check_constraints = OFF")
    conn.execute("PRAGMA busy_timeout = 5000")
    try:
        cur = conn.cursor()
        experiments = db_io.load_experiments(cur)
        id_pairs = db_io.load_selected_pairs(cur)
        spectra = db_io.load_spectra(cur)

        for algo_name, library_name, compute_once in selected_algorithms(
            args.algorithm
        ):
            impl_id = db_io.get_implementation_id(cur, algo_name, library_name)
            runner.run_algorithm(
                conn=conn,
                algorithm_name=algo_name,
                library_name=library_name,
                implementation_id=impl_id,
                experiments=experiments,
                spectra=spectra,
                id_pairs=id_pairs,
                compute_once=compute_once,
            )
    finally:
        conn.close()


if __name__ == "__main__":
    main()
