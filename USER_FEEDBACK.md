## Decisions needed
1) Release discovery: how should drafts/prereleases be handled when selecting assets?
   - Please choose: (A) ignore drafts/prereleases by default; allow `--include-prerelease` to opt-in, (B) allow prereleases by default when no stable exists, (C) always allow prereleases (no opt-in).
    - A
2) Tag/version source when `--tag` and `--version` are omitted.
   - Please choose: (A) require explicit input in v0 (no auto-latest), or (B) default to latest GitHub Release tag when available.
    - B
3) Semver strictness for `--version` (after `v` normalization for tags).
   - Please choose: (A) strict `X.Y.Z` only, (B) allow prerelease/build metadata (`X.Y.Z-rc.1`, `+build.1`), (C) accept any string and only validate non-empty.
    - B
4) `bd release` idempotency when formula already targets the requested version.
   - Please choose: (A) treat as no-op success with a warning, (B) fail with a dedicated exit code, (C) require `--force` to re-apply and continue.
    - C
5) `bd init --import-formula` merge policy for conflicting fields.
   - Please choose: (A) formula is source-of-truth for formula fields; config only fills gaps, (B) config is source-of-truth unless `--prefer-formula` is set, (C) always prompt per field in interactive mode and fail in non-interactive unless `--prefer-formula`.
   - A