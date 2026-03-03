"""Compute missing Python reference similarities and timings in a single pass.

Reads experiments, spectra, and implementations from the SQLite DB, generates
spectrum pairs at runtime, computes Python-reference results for any missing
pairs, and writes back to the results table.

Usage:
  python_reference_compute.py <db_path> [max_spectra]
  python_reference_compute.py <db_path> --max-spectra <max_spectra>
  python_reference_compute.py <db_path> --algorithm <algorithm_name>
"""

from __future__ import annotations

import sys
import sqlite3

from python_ref import db_io
from python_ref import runner
from python_ref import workload
from python_ref.algorithms import cosine_greedy
from python_ref.algorithms import cosine_hungarian
from python_ref.algorithms import entropy_unweighted
from python_ref.algorithms import entropy_weighted
from python_ref.algorithms import modified_greedy_cosine

def parse_cli_args(argv: list[str]) -> tuple[str, int | None, str | None]:
    db_path = argv[1] if len(argv) > 1 else "fixtures/benchmark.db"
    max_spectra: int | None = None
    selected_algorithm: str | None = None
    extra_positional: list[str] = []

    i = 2
    while i < len(argv):
        token = argv[i]
        if token == "--max-spectra":
            if i + 1 >= len(argv):
                raise SystemExit("missing value for --max-spectra")
            max_spectra = int(argv[i + 1])
            i += 2
            continue
        if token.startswith("--max-spectra="):
            max_spectra = int(token.split("=", 1)[1])
            i += 1
            continue
        if token == "--algorithm":
            if i + 1 >= len(argv):
                raise SystemExit("missing value for --algorithm")
            selected_algorithm = argv[i + 1]
            i += 2
            continue
        if token.startswith("--algorithm="):
            selected_algorithm = token.split("=", 1)[1]
            i += 1
            continue
        extra_positional.append(token)
        i += 1

    if max_spectra is None:
        if len(extra_positional) == 1:
            max_spectra = int(extra_positional[0])
        elif len(extra_positional) == 2:
            _legacy_batch = int(extra_positional[0])
            max_spectra = int(extra_positional[1])
            print(
                "[python_reference_compute] legacy batch_size argument is ignored",
                file=sys.stderr,
            )
        elif len(extra_positional) > 2:
            raise SystemExit("too many positional arguments")
    else:
        if len(extra_positional) == 1:
            _legacy_batch = int(extra_positional[0])
            print(
                "[python_reference_compute] legacy batch_size argument is ignored",
                file=sys.stderr,
            )
        elif len(extra_positional) > 1:
            raise SystemExit("too many positional arguments")

    return db_path, max_spectra, selected_algorithm


DB_PATH, MAX_SPECTRA, SELECTED_ALGORITHM = parse_cli_args(sys.argv)

ALGORITHMS = [
    ("CosineHungarian", "matchms", cosine_hungarian.compute_once),
    ("CosineGreedy", "matchms", cosine_greedy.compute_once),
    ("ModifiedGreedyCosine", "matchms", modified_greedy_cosine.compute_once),
    ("EntropySimilarityWeighted", "ms_entropy", entropy_weighted.compute_once),
    ("EntropySimilarityUnweighted", "ms_entropy", entropy_unweighted.compute_once),
]


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
    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA journal_mode = WAL")
    try:
        cur = conn.cursor()
        experiments = db_io.load_experiments(cur)
        spectra = db_io.load_spectra(cur, max_spectra=MAX_SPECTRA)
        id_pairs = workload.generate_pairs(list(spectra.keys()))

        for algo_name, library_name, compute_once in selected_algorithms(
            SELECTED_ALGORITHM
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
