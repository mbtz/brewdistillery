# Release discovery policy (GitHub Release)

This document defines how `bd release` selects a GitHub Release when using
`artifact.strategy = "release-asset"`.

## Selection order

1) If `--tag` is provided, fetch the release by tag via
   `GET /repos/{owner}/{repo}/releases/tags/{tag}`.
2) Else if `--version` is provided, fetch the release by tag after applying
   version/tag normalization (leading `v` allowed by default).
3) Else, select the latest GitHub Release:
   - Without `--include-prerelease`: use `GET /repos/{owner}/{repo}/releases/latest`.
   - With `--include-prerelease`: use `GET /repos/{owner}/{repo}/releases` and
     pick the first non-draft entry in the list.

Notes:
- GitHub returns releases in descending creation order. When
  `--include-prerelease` is set, a prerelease will be selected if it appears
  first in the list.
- Draft releases are never selected.
- If `--version` is provided, the resolved release tag must match the requested
  version after normalization, or the command fails.
- In `--dry-run` mode, release discovery is skipped and `--version` or `--tag`
  is required.

## Draft and prerelease rules

- Draft releases are rejected in all modes.
- Prereleases are rejected unless `--include-prerelease` is set.

## Error messages (exact)

Missing or invalid releases:
- `GitHub release tag '<tag>' not found for <owner>/<repo>`
- `no GitHub releases found for <owner>/<repo>`
- `GitHub release '<tag>' is a draft; publish it before releasing`
- `GitHub release '<tag>' is a prerelease; re-run with --include-prerelease`

Version/tag mismatch:
- `GitHub release tag '<tag>' does not match requested version '<version>'`

Missing assets (release discovery feeds into asset selection):
- `no usable release assets found (checksum/signature assets are ignored)`
- `no release assets match target os=<os> arch=<arch>; available assets: <list>`
- `release asset '<name>' not found; available assets: <list>`
- `release asset (rendered from template) '<name>' not found; available assets: <list>`
- `multiple release assets match selection: <list>`

## Examples

Latest stable release (default):
```
bd release
```

Latest release including prereleases:
```
bd release --include-prerelease
```

Explicit tag (fails if draft/prerelease unless flag provided):
```
bd release --tag v1.2.3
```

Explicit version (tag normalization applies; mismatch fails):
```
bd release --version 1.2.3
```

Tag + version mismatch example (fails):
```
bd release --tag v1.2.4 --version 1.2.3
```
