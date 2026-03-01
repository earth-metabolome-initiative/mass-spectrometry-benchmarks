# spectral-cosine-similarity

Benchmark pipeline comparing cosine-similarity implementations for mass spectra.

## Local development checks

From `spectral-cosine-similarity/`:

```bash
cargo fmt --all -- --check
cargo check --locked
cargo clippy --all-targets -- -D warnings
cargo test --no-run --locked
python3 -m compileall -q scripts
uv run python3 -c "import matchms"
```

## Pre-hooks (Rust-native with `prek`)

This repository keeps hook definitions in `.pre-commit-config.yaml`, executed by `prek`.

Install and activate:

```bash
uv tool install prek
prek install --hook-type pre-commit --hook-type pre-push
```

Run hooks manually:

```bash
prek run --all-files
prek run --hook-stage pre-push --all-files
```

## CI

GitHub Actions:

- `.github/workflows/ci.yml`: blocking Rust + lightweight Python quality checks on PRs and pushes to `main`.
- `.github/workflows/audit.yml`: scheduled/manual `cargo audit`.
