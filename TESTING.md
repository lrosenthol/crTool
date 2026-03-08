# Testing

Overview of how testing works in crTool. For test layout and detailed commands, see [tests/README.md](tests/README.md).

## Running tests

From the repository root:

```bash
# All tests (library, CLI integration, extraction, validation)
cargo test

# Safer for integration tests (avoids file-access races)
cargo test -- --test-threads=1
```

Run a specific test:

```bash
cargo test test_dog_jpg_simple_manifest
cargo test test_dog_png
```

## Test layout

- **tests/integration_tests.rs** — Sign test images (Dog.jpg, Dog.png, Dog.webp) with simple/full manifests, verify output.
- **tests/test_crjson_extraction.rs**, **test_validation.rs** — Extraction and schema validation.
- **tests/common/mod.rs** — Helpers and fixture checks (e.g. `test_fixtures_exist`, `test_certs_exist`).

Successful integration tests write signed images to `target/test_output/` (e.g. `Dog_simple.jpg`, `Dog_full.png`).

## Certificates

Integration tests use **Ed25519** certificates from the c2pa-rs test infrastructure:

- `tests/fixtures/certs/ed25519.pub` — certificate chain  
- `tests/fixtures/certs/ed25519.pem` — private key  

These work with the c2pa path dependency and the test signer setup; no CA-signed certs are required for the test suite.

For **manual CLI** testing you can also use:

- `./generate_test_certs.sh` → `examples/certs/` (self-signed; use with `--allow-self-signed` when supported), or  
- The same `tests/fixtures/certs/ed25519.*` with `--allow-self-signed` for signing from the CLI.

## Verifying signed output

After tests (or after running the CLI), you can inspect manifests with c2pa-tool:

```bash
cargo install c2pa-tool
c2pa target/test_output/Dog_simple.jpg
```

## CI

Tests run in CI (e.g. `.github/workflows/ci.yml`) with c2pa-rs checked out as a sibling. Use `--test-threads=1` in CI if your workflow does so locally to avoid flakiness.

## More detail

- [tests/README.md](tests/README.md) — Test structure, certificates, known issues, c2pa-rs notes.  
- [SETUP.md](SETUP.md) — First-time setup and `verify_setup.sh`.  
- [DEVELOPMENT.md](DEVELOPMENT.md) — Day-to-day workflow and scripts.
