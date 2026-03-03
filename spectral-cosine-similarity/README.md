# spectral-cosine-similarity

[![CI](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/ci.yml)
[![Security audit](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/audit.yml/badge.svg?branch=main)](https://github.com/earth-metabolome-initiative/mass-spectrometry-benchmarks/actions/workflows/audit.yml)

Benchmark pipeline comparing cosine-similarity implementations for mass spectra.

## IMPORTANT BENCHMARK SCOPE DISCLAIMER

This benchmark currently does **not** apply spectral normalization or spectral sanitization workflows.

- No denoising, windowed top-k filtering, precursor peak removal, or intensity normalization is performed.
- Ingest filtering is currently limited to structural validity checks, removal of nonpositive-intensity peaks, and peak-count bounds (`min_peaks=5`, `max_peaks=1000`).

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
