# Release asset selection and OS/arch normalization

This spec defines how `bd release` selects GitHub release assets and how OS/arch
values are normalized for template rendering and matching.

## Asset selection flow

Selection happens per target. When no target matrix is configured, a single
"universal" target is resolved using the global `artifact` settings.

Precedence order (highest first):
1) Target-specific `artifact.targets.<target>.asset_name`
2) Target-specific `artifact.targets.<target>.asset_template`
3) Global `artifact.asset_name`
4) Global `artifact.asset_template`
5) Auto-match (heuristics)

### Step 1: Load candidate assets
- Source: GitHub Release assets for the selected release (drafts ignored;
  prereleases only when `--include-prerelease` is set).
- Exclude checksum/signature artifacts from candidates. A candidate is ignored
  if its name ends with any of these suffixes (case-insensitive):
  `.sha256`, `.sha256sum`, `.sig`, `.asc`.

### Step 2: Resolve by explicit `asset_name`
- If an explicit `asset_name` is set (global or target-specific), require an
  exact case-sensitive match.
- If missing, error:
  - Message: `asset not found: <name>; available assets: <a>, <b>, ...`
  - Exit code: 3 (invalid input)

### Step 3: Resolve by `asset_template`
- Render the template using the normalized fields:
  - `{name}`: formula name
  - `{version}`: normalized version string (no `v` prefix)
  - `{os}` / `{arch}`: normalized target values
- Require an exact case-sensitive match for the rendered name.
- If missing, error:
  - Message: `no release asset matches template '<template>' for target '<target>'`
  - Exit code: 3

### Step 4: Auto-match heuristics (no explicit name/template)
When no name/template is provided, select a best match from candidates:
1) Filter to assets that include the exact version string (normalized, no `v`).
2) Prefer archive extensions in this order:
   - `.tar.gz`
   - `.zip`
   - any other archive (`.tgz`, `.tar.bz2`, `.tar.xz`, `.gz`)
3) Prefer names that contain the normalized `{os}` and `{arch}` when the target
   is OS/arch-specific; otherwise ignore OS/arch scoring.
4) If multiple remain, choose the shortest name.
5) If multiple still remain, fail with ambiguity.

Ambiguity error:
- Message: `multiple release assets match target '<target>'; specify --asset-name or --asset-template`
- Exit code: 3

No match error:
- Message: `no release assets match target '<target>'; available assets: <a>, <b>, ...`
- Exit code: 3

## Multi-target resolution

When `artifact.targets` is configured, resolve each target independently using
its overrides first, then global settings. All targets must resolve to a unique
asset and SHA.

If any target fails, the command fails and lists the missing or ambiguous
targets.

Example failure:
- Message: `no release assets match target 'darwin-arm64'; available assets: ...`
- Exit code: 3

## OS/arch normalization

Normalization is used for template rendering and auto-match scoring.

| Input (uname/Cargo) | Normalized os | Normalized arch |
| --- | --- | --- |
| `darwin` | `darwin` | `amd64` for `x86_64`, `arm64` for `aarch64` |
| `linux` | `linux` | `amd64` for `x86_64`, `arm64` for `aarch64` |
| other | (error) | (error) |

If an unknown OS or arch is detected, fail early with:
- Message: `unsupported target: os='<os>', arch='<arch>'`
- Exit code: 3
