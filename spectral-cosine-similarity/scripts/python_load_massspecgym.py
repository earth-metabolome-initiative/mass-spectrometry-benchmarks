"""Load Mass Spec Gym dataset into the benchmark database.

Parses spectra and molecules, computes ECFP fingerprints, and inserts them
into the SQLite DB. All qualifying spectra are loaded (no cap).

Usage:
  uv run python3 scripts/python_load_massspecgym.py <db_path>
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
import sys
from typing import Any

import numpy as np
from numpy.typing import NDArray
from skfp.fingerprints import (
    AtomPairFingerprint,
    ECFPFingerprint,
    MACCSFingerprint,
    MAPFingerprint,
    RDKitFingerprint,
)
from tqdm import tqdm

FINGERPRINT_CONFIGS: list[tuple[str, Any, dict[str, object]]] = [
    (
        "ecfp",
        ECFPFingerprint(fp_size=2048, radius=2, n_jobs=-1),
        {"fp_size": 2048, "radius": 2},
    ),
    (
        "fcfp",
        ECFPFingerprint(
            fp_size=2048,
            radius=2,
            use_pharmacophoric_invariants=True,
            n_jobs=-1,
        ),
        {"fp_size": 2048, "radius": 2, "pharmacophoric": True},
    ),
    (
        "maccs",
        MACCSFingerprint(n_jobs=-1),
        {"fp_size": 166},
    ),
    (
        "rdkit",
        RDKitFingerprint(fp_size=2048, n_jobs=-1),
        {"fp_size": 2048},
    ),
    (
        "atompair",
        AtomPairFingerprint(fp_size=2048, n_jobs=-1),
        {"fp_size": 2048},
    ),
    (
        "map",
        MAPFingerprint(fp_size=2048, n_jobs=-1),
        {"fp_size": 2048},
    ),
]


HASH_DECIMALS = 6
MIN_PEAKS = 5
MAX_PEAKS = 1000
INSERT_BATCH = 500


def canonicalize_component(value: float) -> str:
    if not np.isfinite(value):
        return str(value).lower()
    scale = 10.0**HASH_DECIMALS
    quantized = round(value * scale) / scale
    if quantized == 0.0:
        quantized = 0.0  # normalize -0.0
    return f"{quantized:.6f}"


def compute_spectrum_hash(
    precursor_mz: float, peaks: list[tuple[float, float]]
) -> str:
    sorted_peaks = sorted(peaks, key=lambda p: (p[0], p[1]))
    payload = f"pmz={canonicalize_component(precursor_mz)}"
    payload += f";n={len(sorted_peaks)}"
    payload += ";peaks="
    payload += "|".join(
        f"{canonicalize_component(mz)}:{canonicalize_component(intensity)}"
        for mz, intensity in sorted_peaks
    )
    return hashlib.sha256(payload.encode()).hexdigest()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Load Mass Spec Gym data into benchmark DB"
    )
    parser.add_argument("db_path", help="Path to SQLite database")
    return parser.parse_args()


def quality_filter_peaks(
    mzs: NDArray[np.float64], intensities: NDArray[np.float64]
) -> list[tuple[float, float]] | None:
    """Apply quality filters and return sorted peak list, or None if rejected."""
    if len(mzs) == 0:
        return None

    # Filter to positive intensities
    positive_mask = intensities > 0.0
    mzs = mzs[positive_mask]
    intensities = intensities[positive_mask]

    if len(mzs) < MIN_PEAKS or len(mzs) > MAX_PEAKS:
        return None

    # Sort by m/z
    order = np.argsort(mzs, kind="stable")
    mzs = mzs[order]
    intensities = intensities[order]

    # Check for duplicate m/z
    if len(mzs) > 1 and np.any(np.diff(mzs) == 0.0):
        return None

    return list(zip(mzs.tolist(), intensities.tolist(), strict=True))


def main() -> None:
    args = parse_args()
    conn = sqlite3.connect(args.db_path)
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA synchronous = NORMAL")
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute("PRAGMA ignore_check_constraints = OFF")
    conn.execute("PRAGMA busy_timeout = 5000")

    cur = conn.cursor()

    # Check existing spectra count
    cur.execute("SELECT COUNT(*) FROM spectra")
    existing_count: int = cur.fetchone()[0]
    if existing_count > 0:
        print(
            f"[load_massspecgym] {existing_count} spectra already in DB, skipping",
            file=sys.stderr,
        )
        conn.close()
        return

    # Load existing hashes
    cur.execute("SELECT spectrum_hash FROM spectra")
    seen_hashes: set[str] = {row[0] for row in cur.fetchall()}

    print("[load_massspecgym] Loading Mass Spec Gym dataset...", file=sys.stderr)
    import pandas as pd
    from huggingface_hub import hf_hub_download

    tsv_path: str = hf_hub_download(
        repo_id="roman-bushuiev/MassSpecGym",
        filename="data/MassSpecGym.tsv",
        repo_type="dataset",
    )
    df = pd.read_csv(tsv_path, sep="\t")
    df = df.set_index("identifier")
    df["mzs"] = df["mzs"].apply(
        lambda s: np.array(list(map(float, s.split(","))), dtype=np.float64)
    )
    df["intensities"] = df["intensities"].apply(
        lambda s: np.array(list(map(float, s.split(","))), dtype=np.float64)
    )
    print(
        f"[load_massspecgym] Loaded {len(df)} rows from massspecgym",
        file=sys.stderr,
    )

    # Filter rows with valid SMILES
    df = df[df["smiles"].notna() & (df["smiles"].str.strip() != "")]
    print(
        f"[load_massspecgym] {len(df)} rows with valid SMILES", file=sys.stderr
    )

    # Collect unique molecules by inchikey
    molecule_cache: dict[str, int] = {}  # inchikey -> molecule_id
    cur.execute("SELECT id, inchikey FROM molecules")
    for row in cur.fetchall():
        molecule_cache[row[1]] = row[0]

    # Prepare molecule data (grouped by inchikey)
    molecule_smiles: dict[str, str] = {}  # inchikey -> smiles
    for _, row in df.drop_duplicates(subset=["inchikey"]).iterrows():
        ik = str(row["inchikey"]).strip()
        smi = str(row["smiles"]).strip()
        if ik and smi and ik not in molecule_cache:
            molecule_smiles[ik] = smi

    # Insert new molecules (without fingerprints)
    if molecule_smiles:
        print(
            f"[load_massspecgym] Inserting {len(molecule_smiles)} new molecules...",
            file=sys.stderr,
        )
        inchikeys_list = list(molecule_smiles.keys())
        smiles_list = [molecule_smiles[ik] for ik in inchikeys_list]

        for ik in tqdm(inchikeys_list, desc="Inserting molecules"):
            cur.execute(
                "INSERT OR IGNORE INTO molecules (smiles, inchikey) VALUES (?, ?)",
                (molecule_smiles[ik], ik),
            )
            if cur.lastrowid and cur.lastrowid > 0:
                molecule_cache[ik] = cur.lastrowid
            else:
                cur.execute(
                    "SELECT id FROM molecules WHERE inchikey = ?", (ik,)
                )
                row = cur.fetchone()
                if row:
                    molecule_cache[ik] = row[0]
        conn.commit()

        # Insert fingerprint algorithms and compute fingerprints
        fp_algo_ids: dict[str, int] = {}
        for name, _calc, params in FINGERPRINT_CONFIGS:
            cur.execute(
                "INSERT OR IGNORE INTO fingerprint_algorithms "
                "(name, params) VALUES (?, ?)",
                (name, json.dumps(params, sort_keys=True)),
            )
            cur.execute(
                "SELECT id FROM fingerprint_algorithms WHERE name = ?", (name,)
            )
            fp_algo_ids[name] = cur.fetchone()[0]
        conn.commit()

        for name, fp_calculator, _params in FINGERPRINT_CONFIGS:
            algo_id = fp_algo_ids[name]
            print(
                f"[load_massspecgym] Computing {name} fingerprints for "
                f"{len(smiles_list)} molecules...",
                file=sys.stderr,
            )
            fp_matrix = fp_calculator.transform(smiles_list)

            for i, ik in enumerate(
                tqdm(inchikeys_list, desc=f"Inserting {name} fingerprints")
            ):
                fingerprint_molecule_id = molecule_cache.get(ik)
                if fingerprint_molecule_id is None:
                    continue
                fp_bits: NDArray[np.uint8] = fp_matrix[i].astype(np.uint8)
                fp_bytes = np.packbits(fp_bits).tobytes()
                cur.execute(
                    "INSERT OR IGNORE INTO fingerprints "
                    "(molecule_id, fingerprint_algorithm_id, fingerprint) "
                    "VALUES (?, ?, ?)",
                    (fingerprint_molecule_id, algo_id, fp_bytes),
                )
            conn.commit()

    # Process spectra
    print("[load_massspecgym] Processing spectra...", file=sys.stderr)
    inserted = 0
    skipped_hash = 0
    skipped_quality = 0

    for _, row in tqdm(df.iterrows(), total=len(df), desc="Loading spectra"):
        # Parse peaks from massspecgym format
        try:
            mzs_raw = row["mzs"]
            ints_raw = row["intensities"]
            if isinstance(mzs_raw, str):
                mzs = np.array(
                    [float(x) for x in mzs_raw.split(",") if x.strip()],
                    dtype=np.float64,
                )
                intensities = np.array(
                    [float(x) for x in ints_raw.split(",") if x.strip()],
                    dtype=np.float64,
                )
            else:
                mzs = np.asarray(mzs_raw, dtype=np.float64)
                intensities = np.asarray(ints_raw, dtype=np.float64)
        except (ValueError, TypeError):
            skipped_quality += 1
            continue

        peaks = quality_filter_peaks(mzs, intensities)
        if peaks is None:
            skipped_quality += 1
            continue

        precursor_mz = float(row["precursor_mz"])
        spectrum_hash = compute_spectrum_hash(precursor_mz, peaks)

        if spectrum_hash in seen_hashes:
            skipped_hash += 1
            continue
        seen_hashes.add(spectrum_hash)

        # Look up molecule (skip spectra without a known molecule)
        ik = str(row["inchikey"]).strip() if "inchikey" in row.index else ""
        molecule_id: int | None = molecule_cache.get(ik) if ik else None
        if molecule_id is None:
            skipped_quality += 1
            continue

        # Determine name
        identifier = str(
            row.get("identifier", row.get("spectrum_id", ""))
        ).strip()
        if not identifier:
            identifier = f"spectrum_{spectrum_hash[:12]}"
        name = identifier

        peaks_json = json.dumps(peaks)

        cur.execute(
            "INSERT OR IGNORE INTO spectra "
            "(name, raw_name, source_file, spectrum_hash, precursor_mz, "
            "num_peaks, peaks, molecule_id) "
            "VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (
                name,
                identifier,
                "massspecgym",
                spectrum_hash,
                precursor_mz,
                len(peaks),
                peaks_json,
                molecule_id,
            ),
        )
        if cur.rowcount > 0:
            inserted += 1

        if inserted > 0 and inserted % INSERT_BATCH == 0:
            conn.commit()

    conn.commit()
    conn.close()

    print(
        f"[load_massspecgym] Done: inserted={inserted}, "
        f"skipped_hash={skipped_hash}, skipped_quality={skipped_quality}",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
