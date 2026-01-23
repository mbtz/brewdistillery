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

## Target matrix configuration

Use `artifact.targets` to enable OS or OS/arch-specific assets. All target keys
must use the same shape:
- Per-OS: `<os>` (e.g., `darwin`, `macos`, `osx`, `linux`)
- Per-OS+arch: `<os>-<arch>` (e.g., `darwin-arm64`, `linux-x86_64`)

Mixing per-OS and per-OS+arch keys is invalid.

Canonical error:
- `target keys must be all <os> or all <os>-<arch>` (exit code 3)

Config example (per-OS):
```
[artifact.targets."darwin"]
asset_template = "brewtool-{version}-darwin-universal.tar.gz"

[artifact.targets."linux"]
asset_template = "brewtool-{version}-linux-x86_64.tar.gz"
```

Config example (per-OS+arch):
```
[artifact.targets."darwin-arm64"]
asset_template = "brewtool-{version}-darwin-arm64.tar.gz"

[artifact.targets."linux-amd64"]
asset_template = "brewtool-{version}-linux-amd64.tar.gz"
```

Validation rules:
- Duplicate target keys for the same OS (per-OS) or same OS/arch (per-target)
  are rejected.
- Per-OS templates should not include `{arch}`; use per-OS+arch keys instead.

## Multi-target resolution

When `artifact.targets` is configured, resolve each target independently using
its overrides first, then global settings. All targets must resolve to a unique
asset and SHA.

If any target fails, the command fails and lists the missing or ambiguous
targets.

Example failure:
- Message: `no release assets match target 'darwin-arm64'; available assets: ...`
- Exit code: 3

## Formula output mapping

Universal (no `artifact.targets`):
```
  url "https://github.com/acme/brewtool/releases/download/1.2.3/brewtool-1.2.3.tar.gz"
  sha256 "deadbeef"
```

Per-OS:
```
  on_macos do
    url "https://example.com/brewtool-1.2.3-darwin.tar.gz"
    sha256 "macsha"
  end
  on_linux do
    url "https://example.com/brewtool-1.2.3-linux.tar.gz"
    sha256 "linuxsha"
  end
```

Per-OS+arch:
```
  on_macos do
    if Hardware::CPU.arm?
      url "https://example.com/brewtool-1.2.3-darwin-arm64.tar.gz"
      sha256 "armsha"
    else
      url "https://example.com/brewtool-1.2.3-darwin-amd64.tar.gz"
      sha256 "amdsha"
    end
  end
  on_linux do
    if Hardware::CPU.arm?
      url "https://example.com/brewtool-1.2.3-linux-arm64.tar.gz"
      sha256 "linuxarm"
    else
      url "https://example.com/brewtool-1.2.3-linux-amd64.tar.gz"
      sha256 "linuxamd"
    end
  end
```

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
