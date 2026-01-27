# Error strategy and exit codes

`brewdistillery` uses structured error categories (`AppError`) that map to
stable exit codes suitable for CI.

## Exit code taxonomy

| Exit code | Category | Meaning |
| --- | --- | --- |
| 0 | success | Command completed successfully |
| 1 | other/io | Unexpected or internal failures |
| 2 | missing config | Missing config or required inputs |
| 3 | invalid input | Invalid user input or blocked overwrite/idempotency |
| 4 | git state | Git problems (dirty repo, tag exists, remote ambiguity) |
| 5 | network | Network/API failures and download errors |
| 6 | audit | `bd doctor --audit` failures |

The mapping is defined in `src/errors.rs`.

## Canonical error messages by category

The following messages are intended to be stable and testable.

### Exit code 2: missing config or required inputs

- Missing config:
  - `config not found at <path>`
- Missing required fields (non-interactive):
  - `missing required fields for --non-interactive: <comma-separated list>`
- Missing required fields (import mode):
  - `missing required fields for --import-formula: <comma-separated list>`
- Missing required fields (create tap):
  - `missing required fields for --create-tap: tap-owner, tap-repo`
- Missing required fields (dry-run):
  - `missing required fields for --dry-run: version or tag`
- Missing required fields (source-tarball):
  - `source-tarball requires --version (or --tag)`
- Missing required fields (create-release):
  - `create-release requires --version or --tag when no GitHub release exists`
- Dry-run tap requirements:
  - `dry-run requires tap.path or an absolute tap.formula_path; tap.remote cannot be auto-cloned`
- Tap requirements guidance (release):
  - Missing-field lists should include: `tap.path, tap.remote, or tap.formula_path`

### Exit code 3: invalid input / overwrite blocked

- Formula naming and validation:
  - `formula name cannot be empty`
  - `formula name must not start with 'homebrew-'`
- Version/tag validation:
  - `invalid --version '<value>': expected semver (e.g. 1.2.3)`
  - `invalid --tag '<value>': expected semver (e.g. 1.2.3)`
  - `--version '<version>' does not match --tag '<tag>'`
- Overwrite guards:
  - `config already exists at <path>; re-run with --force or --yes to overwrite`
  - `formula already exists at <path>; re-run with --force or --yes to overwrite`
- Release idempotency:
  - `formula already targets version <version>; re-run with --force to re-apply`
- Tap creation conflicts:
  - `--create-tap cannot be used with --tap-new`
  - `--create-tap cannot be used with --tap-remote`
- Create-release conflicts:
  - `--create-release cannot be used with --skip-tag`
- Asset selection errors:
  - `no usable release assets found (checksum/signature assets are ignored)`
  - `no release asset matches template '<template>' for target '<target>'`
  - `no release assets match target '<target>'; available assets: <list>`
  - `multiple release assets match target '<target>'; specify --asset-name or --asset-template`
- Target matrix shape errors:
  - `target keys must be all <os> or all <os>-<arch>`

### Exit code 4: git state failures

- Dirty working tree:
  - `<label> has uncommitted changes; re-run with --allow-dirty to continue`
- Tag already exists:
  - `tag '<tag>' already exists; re-run with --skip-tag or choose a new version`
- Remote ambiguity:
  - `multiple git remotes found; configure origin or set a matching remote URL`
  - `multiple git remotes found; specify --host-owner/--host-repo`
  - `no GitHub remote found; specify --host-owner/--host-repo`
  - `unable to parse GitHub remote URL; specify --host-owner/--host-repo`
- Create-release push requirement:
  - `--create-release requires pushing the tag; re-run without --no-push`

### Exit code 5: network and API failures

- GitHub API failures:
  - `GitHub API request failed: <details>`
  - `GitHub token missing; set GITHUB_TOKEN or GH_TOKEN to create the tap repo`
- Release discovery failures:
  - `GitHub release tag '<tag>' not found for <owner>/<repo>`
  - `no GitHub releases found for <owner>/<repo>; create a GitHub Release or set artifact.strategy=source-tarball`
  - `GitHub release '<tag>' is a draft; publish it before releasing`
  - `GitHub release '<tag>' is a prerelease; re-run with --include-prerelease`
- Download/checksum failures:
  - `download exceeded size limit (<bytes> bytes) for <url>`
  - `failed to download asset`

### Exit code 6: audit failures

- `brew audit requires a resolved formula path`
- `failed to run brew audit: <err>`
- `brew audit failed` (stdout/stderr appended when present)

## References

- `docs/non-interactive.md`
- `docs/asset-selection.md`
- `docs/release-discovery.md`
- `docs/release-orchestration.md`
