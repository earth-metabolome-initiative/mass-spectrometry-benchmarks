# spectral-cosine-similarity

[![CI](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/ci.yml)
[![Security audit](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/audit.yml/badge.svg?branch=main)](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/audit.yml)

Benchmark pipeline comparing cosine-similarity implementations for mass spectra.

## IMPORTANT BENCHMARK SCOPE DISCLAIMER

This benchmark currently does **not** apply spectral normalization or spectral sanitization workflows.

- No denoising, windowed top-k filtering, precursor peak removal, or intensity normalization is performed.
- Ingest filtering is currently limited to structural validity checks, removal of nonpositive-intensity peaks, and peak-count bounds (`min_peaks=5`, `max_peaks=1000`).

## Benchmark Parameterization

This benchmark currently runs a **single** experiment configuration (one reference benchmark setup), using matchms default parameters:

- `tolerance=0.01`
- `mz_power=0.0`
- `intensity_power=1.0`

## Current Plots (Preliminary)

The plots below are generated from the current `output/` artifacts:

- Timing: `output/timing.svg`
- RMSE vs reference: `output/rmse.svg`

![Timing by peak count](output/timing.svg)

![RMSE vs reference by peak count](output/rmse.svg)

## Performance Notes (Preliminary)

Current results are **preliminary** and should be treated as directional, not definitive.

- Rust implementations are consistently faster than the Python reference implementations across most peak-count buckets.
- Greedy variants tend to be faster than exact/Hungarian-style variants, especially as peak counts increase.
- Accuracy agreement is **not** uniformly tight across implementations.
- Greedy approximations can show non-trivial RMSE deltas (including around `1e-3` in some settings), so implementations should not be treated as numerically interchangeable without checking the current RMSE outputs.

Observed acceleration ranges from the current run (`output/tables.md`, spectra used: `1000`):

- `CosineGreedy` (Rust vs `matchms`): about `1.9x` to `3.2x` faster, depending on peak-count bucket.
- `CosineHungarian` (Rust vs `matchms`): about `3.1x` to `8.4x` faster (`~3.1x` to `3.5x` in the denser buckets).
- `ModifiedGreedyCosine` (Rust vs `matchms`): about `2.2x` to `4.0x` faster.
- `EntropySimilarityWeighted` (Rust vs `ms_entropy`): about `1.8x` to `4.8x` faster.
- `EntropySimilarityUnweighted` (Rust vs `ms_entropy`): about `2.3x` to `10.6x` faster.

Important caveats for interpreting these plots:

- No spectral normalization/sanitization pipeline is applied yet.
- Results are tied to one dataset slice (`max-spectra=1000`) and one parameterization.
- Some edge buckets have small sample sizes (for example `n=36` for `513–1023`), which can amplify apparent speedup swings.
- Absolute timings depend on hardware, runtime environment, and dependency versions.

Dataset source for benchmark runs:

- pinned snapshot: Zenodo record `11193898`, file `ALL_GNPS_cleaned.mgf`
- local cache path: `fixtures/ALL_GNPS_cleaned.mgf`

```bash
cargo run --release -- --max-spectra 100
```

Enable ntfy notifications (random topic per run):

```bash
cargo run --release -- --max-spectra 100 --ntfy
```

Pre-commit hooks (with `prek`):

```bash
uv tool install prek
prek validate-config ../prek.toml
prek install
prek run --all-files
prek auto-update
```
