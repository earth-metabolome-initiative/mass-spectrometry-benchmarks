"""Compute missing Python reference similarities and timings in a single pass.

Reads experiments, spectra, and implementations from the SQLite DB, generates
spectrum pairs at runtime, computes Python-reference results for any missing
pairs, and writes back to the results table.

Usage: python_reference_compute.py <db_path> [batch_size] [max_spectra]
"""

from __future__ import annotations

import sqlite3
import sys

from python_ref import db_io
from python_ref import runner
from python_ref import workload
from python_ref.algorithms import cosine_greedy
from python_ref.algorithms import cosine_hungarian
from python_ref.algorithms import entropy_unweighted
from python_ref.algorithms import entropy_weighted
from python_ref.algorithms import modified_cosine
from python_ref.algorithms import modified_greedy_cosine

DB_PATH = sys.argv[1] if len(sys.argv) > 1 else "fixtures/benchmark.db"
BATCH_SIZE = int(sys.argv[2]) if len(sys.argv) > 2 else None
MAX_SPECTRA = int(sys.argv[3]) if len(sys.argv) > 3 else None

ALGORITHMS = [
    ("CosineHungarian", "matchms", cosine_hungarian.compute_once),
    ("CosineGreedy", "matchms", cosine_greedy.compute_once),
    ("ModifiedCosineApprox", "matchms", modified_cosine.compute_once),
    ("ModifiedGreedyCosine", "matchms", modified_greedy_cosine.compute_once),
    ("EntropySimilarityWeighted", "ms_entropy", entropy_weighted.compute_once),
    ("EntropySimilarityUnweighted", "ms_entropy", entropy_unweighted.compute_once),
]


def main() -> None:
    conn = sqlite3.connect(DB_PATH)
    try:
        cur = conn.cursor()
        experiments = db_io.load_experiments(cur)
        spectra = db_io.load_spectra(cur, max_spectra=MAX_SPECTRA)
        id_pairs = workload.generate_pairs(list(spectra.keys()))

        for algo_name, library_name, compute_once in ALGORITHMS:
            impl_id = db_io.get_implementation_id(cur, algo_name, library_name)
            runner.run_algorithm(
                conn=conn,
                algorithm_name=algo_name,
                library_name=library_name,
                implementation_id=impl_id,
                experiments=experiments,
                spectra=spectra,
                id_pairs=id_pairs,
                batch_size=BATCH_SIZE,
                compute_once=compute_once,
            )
    finally:
        conn.close()


if __name__ == "__main__":
    main()
