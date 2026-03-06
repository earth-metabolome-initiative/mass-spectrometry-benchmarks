# spectral-cosine-similarity

[![CI](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/ci.yml)
[![Security audit](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/audit.yml/badge.svg?branch=main)](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/audit.yml)

Benchmark pipeline comparing spectral similarity implementations for mass spectra.

## Prerequisites

- **Rust** (nightly toolchain)
- **[uv](https://docs.astral.sh/uv/)** for managing the Python environment
- Python packages `matchms` and `ms_entropy` (install with `uv sync` in this directory)

## Running the pipeline

The pipeline has five stages: **Initialize DB** &rarr; **Download** &rarr; **Prepare** &rarr; **Compute** &rarr; **Report**. A single command runs all of them. Results are stored in a SQLite database and the pipeline is fully resumable &mdash; re-running the same command skips already-computed work.

### Full run (all spectra, all pairs)

```bash
cargo run --release
```

This downloads the GNPS dataset, loads all spectra, generates every N&times;(N+1)/2 pair, benchmarks all Rust and Python implementations, and writes report artifacts to `output/`.

### Quick test run

Limit to 100 spectra and sample 500 random pairs:

```bash
cargo run --release -- --max-spectra 100 --num-pairs 500
```

### Options

| Flag | Description |
|------|-------------|
| `--max-spectra N` | Load only the first N spectra (useful for quick tests) |
| `--num-pairs N` | Sample N random pairs instead of all N&times;(N+1)/2 (deterministic) |
| `--ntfy` | Send push notifications via [ntfy](https://ntfy.sh) on stage completion |

### Examples

```bash
# Small smoke test (~6 pairs from 3 spectra)
cargo run --release -- --max-spectra 3

# Medium run with sampled pairs
cargo run --release -- --max-spectra 500 --num-pairs 1000

# Full dataset, all pairs, with push notifications
cargo run --release -- --ntfy
```

### Output

Reports are written to `output/`:

- `timing.svg` &mdash; mean timing by peak-count bucket (faceted line chart with error bars)
- `rmse.svg` &mdash; RMSE vs reference by peak-count bucket
- `tables.md` &mdash; summary statistics in Markdown
