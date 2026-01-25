# Testing matrix

This document outlines the minimum automated test coverage for v0. It focuses on
init/release/doctor flows and uses temporary repos plus mocked network/sha to
avoid external dependencies.

## Matrix

| Area | Scenario | Setup | Assertions | Notes |
| --- | --- | --- | --- | --- |
| init | non-interactive success | temp CLI repo + tap path | writes `.distill/config.toml`, formula exists | use tempdir; no git required |
| init | missing required fields | empty args in non-interactive | exit code 2 + missing fields list | assert AppError::MissingConfig |
| init | overwrite blocked | pre-existing config/formula | exit code 3 unless `--force` | confirm no writes without `--force` |
| init | import-formula | existing formula with desc/homepage/license/bin | config pulls formula fields; formula unchanged | ensure file is not re-rendered |
| init | tap path missing | tap path absent | creates `Formula/` + file | verify scaffolded tree |
| release | dry-run success | local tap path + explicit version/tag + asset selection | prints preview, no writes, SHA=DRY-RUN | no network calls |
| release | dry-run remote-only | tap.remote set without tap.path or absolute formula path | exit code 2 with dry-run tap path error | proves no clone attempt |
| release | dry-run missing version/tag | dry-run without version/tag | exit code 2 with missing version/tag error | fails before any network |
| release | asset selection missing | release with no matching assets | exit code 3 + list available assets | check error message |
| release | prerelease excluded | latest is prerelease, flag off | exit code 3 with prerelease note | include `--include-prerelease` case |
| release | version/tag mismatch | `--version` != normalized `--tag` | exit code 3 | validate exact error wording |
| doctor | missing config | no `.distill/config.toml` | exit code 2 | `config not found at <path>` |
| doctor | warnings | missing optional fields | warnings printed, exit 0 | verify `--strict` flips to error |
| doctor | audit gated | `--audit` with missing brew | exit code 6 | skip if brew not available |

## Harness outline

- Unit tests live beside modules (`src/commands/*.rs`) using `#[cfg(test)]`.
- Use `tempfile::TempDir` for isolated CLI/tap repo directories.
- Provide helper constructors (similar to `base_context` + `base_args`) to
  build `AppContext` and args without depending on the real filesystem layout.
- For release tests, inject a mock host client that returns deterministic
  releases/assets and SHA values. The mock should avoid network and allow
  error injection (missing assets, prerelease-only, HTTP failures).
- Use snapshot-style asserts for rendered formulas or preview output when
  comparing large strings (keep snapshots small and readable).
- Avoid invoking `git` or `brew` in unit tests; reserve those for opt-in
  integration tests gated by environment flags.

## Minimal integration suite (opt-in)

- `bd init --non-interactive` on a temp repo and tap directory.
- `bd release --dry-run` with a mocked host client and static formula.
- `bd doctor` with a valid config and missing fields to confirm warnings.

These should run in CI only when `BREWDISTILLERY_INTEGRATION=1` is set.
