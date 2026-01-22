# brewdistillery PRD (planning)

## Status and critique of existing notes
Status: in_progress (updated 2026-01-22; USER_FEEDBACK integrated)

What is now solid:
- Mission and scope are clear (Rust CLI, Homebrew init + release workflow, GitHub-only v0).
- Core commands are set (`bd init`, `bd release` + alias `bd ship`, `bd doctor`).
- Out-of-scope items are explicit (no CI workflow generation, no private repos, no bottle pipeline).
- GitHub API tap creation is in scope; release automation lives inside `bd` (no helper script).
- Config lives in the main repo under `.distill/` (path now fixed).
- Tap layout and artifact strategy defaults are now aligned with user feedback.

Needs strengthening / missing detail:
- Task list needs tighter definition of done (examples + acceptance criteria + exact error text/exit codes).
- Metadata detection needs concrete edge cases: Rust workspaces, multi-bin naming, missing license/homepage, plus non-Rust preset mappings and conflict resolution.
- Formula generation needs canonical examples: name/class normalization and Ruby output for multi-bin and per-OS/arch assets.
- Release idempotency rules are incomplete (tag exists, missing assets, re-run behavior for `--force`).
- `bd init --import-formula` still needs field-level merge rules and non-interactive overwrite policy.
- Asset selection still needs explicit override behavior (`--asset-name`), deterministic matching/tiebreakers, checksum/sig exclusion, and per-target resolution rules.
- Tap repo local path resolution needs default path rules, temp dir naming, cleanup, and user messaging.
- Non-interactive contract should include explicit required fields for asset selection and tap path/remote ambiguity.
- GitHub tap creation flow ordering + rollback/cleanup steps not detailed.
- Exit code taxonomy lacks mappings to specific failures + message wording.
- Version update strategies need workspace handling rules and config examples.
- `brew tap-new` dependency handling (missing brew, offline) needs fallback/error behavior.
- Cross-platform formula assets need a concrete config shape and Homebrew DSL mapping for per-OS/arch sections.
- Preview/diff UX needs a concrete spec (what gets diffed, how multi-repo changes are presented, dry-run output contract).
- Checksum acquisition details are missing (size/time limits, retry behavior, dry-run policy).
- Git remote selection and ambiguity resolution rules are not fully specified (origin vs upstream, multi-remote non-interactive failures).

Improvements added in this revision:
- Integrated USER_FEEDBACK decisions for prerelease handling, latest tag defaulting, semver strictness, idempotency, and import-formula merge policy.
- Reframed non-Rust support as a first-class requirement (Node/Go/Python manifest presets and bin detection).
- Folded decisions into release discovery, version validation, idempotency, and import-formula sections; updated related tasks.

Remaining execution gaps before implementation:
- Git integration choice (libgit2 vs shell) and remote selection behavior.
- Per-OS/arch asset config shape + formula DSL mapping and validation (decision locked, spec missing).
- Checksum download limits/retry/backoff and dry-run behavior.
- Release discovery selection order + error messaging for prerelease/draft handling.
- Release idempotency outcomes (tag exists, `--force` path) + exit code mapping.
- Non-Rust metadata detection presets and scope (Node/Go/etc) for v0.
- Cargo workspace handling for version detection/update (explicit package selection vs defaults).
- `bd init --import-formula` merge details (field-level map, non-interactive overwrite policy).
- Preview/diff UX contract for multi-repo updates.

This PRD is now structured to convert decisions into discrete, testable tasks. Any additional product decisions needed should be appended to USER_TODO.md.

## Mission
Help CLI authors initialize and maintain Homebrew tap formulae quickly and safely, with strong defaults and clear escape hatches.

## Product principles
- Guided but flexible: great defaults, with overrides for every critical field.
- Safe by default: dry-run, previews, and explicit confirmations before destructive actions.
- Deterministic outputs: re-running commands should be idempotent or clearly explain differences.
- Minimal coupling: support multiple repo/host setups via configuration and host abstraction.

## Non-functional requirements (v0)
- Runs on macOS and Linux (Homebrew supported platforms).
- Works in offline mode for init and template rendering; release may require network for checksums.
- Clear, actionable errors with exit codes that CI can use.
- No background services; single binary with minimal runtime deps.

## Problem statement
Publishing a Homebrew formula requires duplicating metadata, handling version bumps, tagging, tarball hashing, and syncing a separate tap repository. These steps are error-prone and vary across teams. A guided CLI should standardize the workflow while remaining flexible for different hosts and repo layouts.

## Target users
- CLI authors who want to publish on Homebrew without learning tap structure.
- Maintainers who already have a formula and want repeatable release automation.
- Teams that need consistent release practices across multiple repos.

## Success metrics (v0)
- Time-to-first-formula under 5 minutes for a new project.
- Release workflow succeeds on first try in >90% of runs (no manual fixes).
- Generated tap passes `brew audit` and installs via `brew install` on macOS.

## Constraints and assumptions
- The release artifact already exists (we do not build binaries in v0).
- Git is required; Ruby/Homebrew are required only for optional audit/validation steps.
- Default host is GitHub only (v0); other hosts are deferred.
- The CLI repository is a Git repo and has an accessible remote.
- `bd init` may create the tap repo on GitHub via API (token via env, not persisted).
- Private repos are out-of-scope for v0.
- Release assets/tarballs must be publicly downloadable (no auth in v0).
- Version files in the CLI repo are not updated by default during release (configurable).

## Risks and mitigations (v0)
- Homebrew conventions may change; mitigate by pinning template versions and offering `bd doctor --audit`.
- Dual-repo git operations are error-prone; mitigate with previews, explicit repo labels, and `--no-push`.
- Artifact downloads can fail or be slow; mitigate with clear errors and optional retry/backoff.
- Some users lack Homebrew/Ruby; keep audit opt-in and make init/release work without brew.

## Decisions locked (v0)
- Release command: `bd release` with alias `bd ship`.
- Hosting: GitHub only.
- Default push behavior: push tags/commits by default; `--no-push` disables.
- Tap creation: `bd init` supports GitHub API tap repo creation (env token).
- Private repos: not supported in v0.
- Default formula name: repo name (override allowed).
- CI workflow generation: not in v0.
- Version update: default is no update; configurable updater supports `cargo` and `regex` modes.
- Config location: `.distill/config.toml` in the CLI repo; `.distill/` holds related local files.
- Commit message conventions: configurable templates with a simple default (e.g. `feature: Updated formula for version 'X.Y.Z'`).
- Tag format: default semver `X.Y.Z`; tags are optional (release can update formula only).
- `brew tap-new`: preferred scaffold but must be optional via flag.
- `bd doctor`: `brew audit` is opt-in.
- `bd init`: detect existing setup (tap/formula) and avoid overwriting without confirmation.
- Release automation lives in `bd` (no helper script generated).
- Tap layout: separate `homebrew-<repo>` tap repo only in v0.
- Release artifact strategy: default GitHub release assets for binaries; source tarball is a separate explicit strategy (no implicit fallback without config).
- Multi-binary formulae supported in v0.
- Formula `version` stanza is always explicit.
- Formula class naming uses simple CamelCase of segments (no acronym preservation).
- `bd release` auto-clones tap repo to a temp dir when tap remote is known and path is missing.
- Release discovery: GitHub Release required for release-asset strategy; drafts ignored; prereleases ignored unless `--include-prerelease`; when `--tag`/`--version` are omitted, default to latest GitHub Release tag (stable by default).
- Cross-platform formula assets: support per-OS/arch sections in v0 (single universal asset is still allowed).
- Checksum acquisition: download the asset and compute SHA256 locally; checksum assets are ignored in v0.
- Version/tag precedence: normalize a leading `v` in tags and require `--version` to match the normalized tag; accept semver with prerelease/build metadata.
- `bd release` idempotency: when formula already targets the requested version, require `--force` to re-apply and continue.
- `bd init --import-formula`: formula is source-of-truth for formula fields; config only fills gaps.

## In-scope (v0)
- Interactive `bd init` that scaffolds a tap repository and a formula.
- Non-interactive mode for CI or power users (flags or config file).
- A release command that bumps version, tags, updates formula URL/SHA, and pushes.
- Template-based formula generation with safe defaults and local overrides.
- Explicit support for GitHub-hosted repos; extensible architecture for other hosts.
- Optional GitHub API flow to create the tap repo during `bd init`.
- `bd doctor` for validation (config, repo state, formula shape).

## Out-of-scope (v0)
- Homebrew casks.
- Building binaries.
- Bottles and multi-arch packaging pipelines.
- Private repos and auth-protected downloads.
- CI workflow generation.
- Telemetry or analytics.

## Core concepts and terminology
- Tap repository: Homebrew tap repo (usually `homebrew-<name>`).
- Formula name: Homebrew formula Ruby class name and file name.
- CLI repo: the repository where the CLI source lives.
- Host: provider for release artifacts and tarball URLs (GitHub v0).
- Release artifact: a tarball or archive referenced by the formula.

## Repository identity model (v0)
To avoid ambiguity, treat these as distinct identities even when they default to the same repo:
- CLI repo: where `bd` runs and where `.distill/config.toml` lives.
- Tap repo: `homebrew-<name>` repo containing `Formula/<formula>.rb`.
- Artifact repo: repo that hosts release assets or source tarballs (default: same as CLI repo).

Defaults and overrides:
- Default owner/repo: inferred from `origin` remote; if multiple remotes, prompt unless overridden.
- If no GitHub remote is detected, require explicit `--host-owner/--host-repo` (and `--tap-owner/--tap-repo` when creating a tap).
- Allow explicit overrides in config and flags for all three identities (owner, repo, remote URL, local path).

## Repository metadata detection (Rust-first, v0)
- Primary source: `Cargo.toml` `package` fields for name, description, homepage, license, version.
- Binary names: use `[[bin]]` names if present; otherwise default to `package.name`.
- License fallback: SPDX from `Cargo.toml`; otherwise detect from common `LICENSE*` filenames if unambiguous, else prompt.
- Homepage fallback: GitHub remote URL if `package.homepage` is missing.
- Non-Rust fallback (v0 target): probe `package.json`, `go.mod`, and `pyproject.toml` for name/description/version/bin (incl. multi-bin where supported); prompt on conflicts.
- If no known manifest is found, fall back to git remote + prompts; detection stays best-effort and always confirm in interactive flow.

## Core user journeys
1) New project, new tap
- Run `bd init` in the CLI repo.
- Provide metadata (name, description, homepage, license, version, bin name).
- Tool scaffolds or configures `homebrew-<repo>` tap, optionally creates it on GitHub, writes formula, stores config.
- Tool prints next steps (`brew tap`, `brew install`, and release guidance).

2) Existing project, new tap
- Run `bd init --tap-path <path>` or `--tap-remote <url>`.
- Tool detects repo metadata, prompts for missing fields.
- Tool writes formula and config, optionally commits to tap.

2b) Existing tap, existing formula
- Run `bd init --tap-path <path> --import-formula` (or similar).
- Tool detects and validates existing formula/tap setup.
- Tool fills missing config fields and refuses to overwrite without explicit confirmation.

3) Release
- Run `bd release` (or `bd ship`).
- Tool validates clean working tree and config.
- Tool computes tarball URL + SHA256, updates formula, commits, pushes (unless `--no-push`).

4) Validation
- Run `bd doctor` to detect config gaps, formula mismatches, or repo state issues.

## UX and interaction contract
- Interactive flow shows defaults and allows corrections before writing.
- Provide a preview step before writing formula/config (and before pushing).
- Preview should show a unified diff when overwriting existing files and explicitly list which repos will be modified.
- `--non-interactive` requires all required fields via flags or config.
- `--force` (or `--yes` when safe) is required to overwrite existing config/formula in non-interactive mode.
- Input precedence: flags > config file > repo autodetect > prompts.
- Idempotency: re-running `bd init` should not duplicate files or overwrite without confirmation.
- Cancellation must exit cleanly without partial writes (use temp files).
- All writes are atomic; partial scaffolds are cleaned up on failure.
- Output is concise by default with `--verbose` for diagnostics.
- Allow a "back" or "edit" step for key fields in interactive flows.
- Validate inputs early (license identifier, semver, formula/class naming rules).
- Non-interactive contract must be explicit: list required fields and which prompts they replace.

## Non-interactive required fields (draft)
These are required only when they cannot be inferred from config or repo autodetect.

`bd init --non-interactive`:
- CLI repo identity: `--host-owner/--host-repo` or a detectable GitHub remote.
- Tap repo identity: `--tap-owner/--tap-repo` or `--tap-remote` or `--tap-path`.
- Formula fields: `--formula-name` (or repo name), `--description`, `--homepage`, `--license` unless detected from a supported manifest or config.
- Binary name(s): `--bin-name` unless detected from a supported manifest or config list.
- Version: required unless inferred from a supported manifest or config.

`bd release --non-interactive`:
- Resolved config file (or explicit flags) for tap path/remote, formula name, artifact strategy, and asset selection (name/template or per-target overrides when OS/arch matrix is enabled).
- Version input via `--version` or `--tag`; if omitted, use latest GitHub Release tag (non-draft; prereleases only with `--include-prerelease`).

`bd doctor --non-interactive`:
- Config path or defaults; optional `--audit` still opt-in.

## Non-interactive contract table (draft)
Command | Required inputs if not resolved from config/autodetect | Autodetect sources | Failure mode
---|---|---|---
`bd init --non-interactive` | CLI repo identity (host owner/repo), tap identity (owner/repo/remote/path), formula metadata (desc/homepage/license), bin name(s), version (unless inferred) | Git remote, supported manifests, `LICENSE*`, existing formula | Exit with "missing required fields" list; no writes performed.
`bd release --non-interactive` | Tap repo path/remote, formula name/path, artifact strategy, asset name/template (or per-target overrides if OS/arch matrix is enabled), version/tag input (optional if latest Release tag is available) | Config file + GitHub Release API (stable by default; prerelease with flag) + existing formula | Exit with "missing/ambiguous" list; no writes performed.
`bd doctor --non-interactive` | Config path or defaults only | Config file | Exit if config invalid; `--audit` opt-in only.

## Idempotency and overwrite matrix (draft)
Scenario | Expected behavior | Notes
---|---|---
`bd init` with existing `.distill/config.toml` | Show diff; require confirmation or `--force` to overwrite | Default is no overwrite.
`bd init` with existing formula file | If identical: no-op; if different: show diff + confirm | Refuse without explicit consent.
`bd init --import-formula` with conflicts | Treat formula as source-of-truth for formula fields; config fills gaps | Define which config fields can be overwritten and when prompts are required.
`bd release` with existing tag | Fail with guidance (`--skip-tag` or new version) | Idempotency decision required for `--force` in v1.
`bd release` when formula already targets version | Fail unless `--force` is provided; with `--force`, re-apply and continue | Define exit code + exact warning message.
`bd release` with missing tap path but known tap remote | Auto-clone to a temp dir and warn/show path used | Fail if no tap remote is configured.
`bd release` with no matching asset | Fail with list and require `--asset-name` or `--asset-template` | No fallback unless configured.

## CLI surface (proposed)
- `bd init` (interactive by default)
  - Flags: `--non-interactive`, `--config <file>`, `--tap-path`, `--tap-remote`, `--tap-owner`, `--tap-repo`, `--formula-name`, `--repo-name`, `--license`, `--homepage`, `--description`, `--bin-name`, `--version`, `--artifact-strategy`, `--asset-template`, `--host-owner`, `--host-repo`, `--yes`, `--force`, `--dry-run`, `--tap-new` (use `brew tap-new`), `--import-formula`
- `bd release`
  - Alias: `bd ship`
  - Flags: `--version`, `--tag`, `--skip-tag`, `--no-push`, `--tap-path`, `--tap-remote`, `--artifact-strategy`, `--asset-template`, `--asset-name`, `--include-prerelease`, `--force`, `--host-owner`, `--host-repo`, `--dry-run`, `--allow-dirty`
- `bd doctor`
  - Flags: `--config <file>`, `--tap-path`, `--strict`, `--audit`
- `bd template` (optional)
  - Prints or validates the formula template used for generation.

## Configuration
Config location and structure:
- Default path: `.distill/config.toml` in the CLI repo.
- `.distill/` may also hold generated assets (templates, cached metadata).
- Config should be repo-local and committed by default (unless user opts out).

Suggested structure (subject to decisions):
- schema_version: integer for migrations
- project: name, description, homepage, license, bin(s)
- cli: owner, repo, remote, local path (derived; override allowed)
- tap: owner, repo, remote, path, formula name, formula path
- artifact: owner, repo, strategy (release asset vs source tarball), asset template, asset name (optional override)
- release: tag format, tarball URL template, commit message templates
- version_update: optional file path/regex or language strategy for bumping the CLI repo version (off by default)
- host: `github` (v0), optional API base and future provider settings
- template: override path, custom install block

Config rules:
- Precedence: flags > config file > repo autodetect > prompts.
- Validation: fail fast on missing required fields for non-interactive mode.
- Migration: preserve unknown fields and bump `schema_version` only when breaking.

Draft example (shape only, not final):
```
schema_version = 1

[project]
name = "brewdistillery"
description = "Homebrew release helper"
homepage = "https://github.com/acme/brewdistillery"
license = "MIT"
bin = ["bd"]

[cli]
owner = "acme"
repo = "brewdistillery"

[tap]
owner = "acme"
repo = "homebrew-brewdistillery"
path = "../homebrew-brewdistillery"
formula = "brewdistillery"

[artifact]
strategy = "release-asset"
asset_template = "{name}-{version}-{os}-{arch}.tar.gz"
# asset_name = "brewdistillery-1.2.3-darwin-arm64.tar.gz"

[release]
tag_format = "X.Y.Z"
commit_message_template = "feature: Updated formula for version '{version}'"
```

Per-target asset sketch (tentative):
```
[artifact.targets."darwin-arm64"]
asset_name = "brewdistillery-1.2.3-darwin-arm64.tar.gz"

[artifact.targets."linux-amd64"]
asset_template = "{name}-{version}-linux-x86_64.tar.gz"
```

## Tap layout and file structure (v0)
- Tap repo root contains `Formula/<formula>.rb`.
- Optional: README and basic metadata to explain tap usage.
- Enforce separate `homebrew-<repo>` tap repo on GitHub (v0).
- If using `brew tap-new`, ensure layout matches Homebrew expectations.

## Formula name normalization (draft)
- Input source: `--formula-name` or repo name; prompt if empty or invalid.
- Normalize: lowercase, replace spaces/underscores with dashes, collapse multiple dashes, trim leading/trailing dashes.
- Validation: allow only `a-z`, `0-9`, and `-`; reject `homebrew-` prefix; require non-empty.
- Class name: split on `-`/`_`, capitalize each segment, keep digits as-is (e.g., `foo2-bar` -> `Foo2Bar`, `http-server` -> `HttpServer`).

## Formula rules (v0)
- File naming: `<formula>.rb` where formula name is lowercase with dashes.
- Class naming: CamelCase derived from formula name (Homebrew convention).
- Required fields: desc, homepage, url, sha256, license, version.
- Per-OS/arch assets: when configured, generate conditional URL/SHA blocks (exact Homebrew DSL to be confirmed).
- Install block: support one or many binaries; generate `bin.install` with a list when multiple bins are configured.
- Optional blocks: `depends_on`, `livecheck`, and `post_install` are opt-in via template overrides.
- Enforce consistent class naming for acronyms and digits (document the exact transform).

## Release artifact strategy (v0)
- Default: GitHub release asset URL for binaries; source tarball is a separate explicit strategy (no implicit fallback).
- Artifact repo defaults to CLI repo; allow override to a different owner/repo.
- Support explicit asset name template (e.g. `{name}-{version}-{os}-{arch}.tar.gz`) and exact asset name override; allow per-target overrides when OS/arch matrix is configured.
- Checksum policy: download selected asset and compute SHA256 locally; checksum assets are ignored in v0; failures block release.
- Private repos are out-of-scope in v0 (public downloads only).
- Define an OS/arch normalization table (darwin/linux, amd64/arm64) for template expansion.
- Always include an explicit `version` stanza in the generated formula.

### Cross-platform formula assets (v0 decision)
Decision:
- Support per-OS/arch formula sections in v0 (macOS + Linux; optional arch split when multiple assets are provided). Single universal asset remains valid.

Spec details still needed:
- Config shape for multi-target assets (matrix vs explicit map) and precedence between global `asset_template` and per-target overrides.
- Validation rules for missing targets and partial matrices (e.g., macOS-only assets).
- Homebrew DSL mapping for OS/arch sections (confirm exact syntax with docs).

### Release discovery (v0 decision)
Decision for v0:
- `bd release` requires a GitHub Release when `artifact.strategy = "release-asset"`; tag-only flows fail with a clear error.
- Source tarball usage must be explicitly configured (no implicit fallback when a Release is missing).
- Draft releases are always ignored.
- Prereleases are ignored by default; `--include-prerelease` opts in.
- If `--tag` and `--version` are omitted, select the latest GitHub Release tag (stable by default; include prereleases only when opted in).
- If an explicit tag resolves only to a draft or prerelease without opt-in, fail with a clear error.

Spec details still needed:
- Selection order when `--include-prerelease` is set and stable releases also exist (prefer stable vs latest overall).
- Exact error wording/exit codes for missing releases and disallowed prereleases/drafts.

### Version update strategies (draft)
We reference optional version updates but need a concrete, minimal v0 scope:
- `version_update.mode = "none"` (default): no repo version file changes.
- `version_update.mode = "cargo"`: update `Cargo.toml` `package.version` (requires workspace policy).
- `version_update.mode = "regex"`: update a file via regex + replacement template.
Decision for v0:
- Support both `cargo` and `regex` modes.
Open questions to resolve:
- If `cargo`, how to handle workspaces (root vs member package)?
- Should a custom command/script hook be allowed in v0, or deferred to v1?

### OS/arch normalization (draft)
Input (uname/Cargo) | Normalized `os` | Normalized `arch`
---|---|---
`darwin` | `darwin` | `amd64` for `x86_64`, `arm64` for `aarch64`
`linux` | `linux` | `amd64` for `x86_64`, `arm64` for `aarch64`
Other | N/A | N/A

Notes:
- Mapping must be explicit and stable; unknown pairs must fail with a clear error.

## Release asset selection rules (draft)
- If `asset_name` is set, require an exact match (case-sensitive) or fail with a list of available assets.
- Else if `asset_template` is set, render it and require a unique match; fail if missing.
- Else, search release assets for a best match using OS/arch mapping.
- Exclude checksum/signature artifacts (e.g., `*.sha256`, `*.sig`, `*.asc`) from matching.
- Prefer assets that include the exact version string; then prefer `.tar.gz`, then `.zip`, then other archives.
- If multiple matches remain, choose the shortest name; if still ambiguous, fail and require `--asset-template` or `--asset-name`.
- If no asset matches, fail with a list; source tarball is used only when explicitly configured as the strategy (or a future explicit fallback flag).
- If multi-target formula support is enabled, resolve assets per target using target-specific overrides when present; each target must resolve uniquely for its OS/arch.
- If multi-target formula output is enabled, compute SHA per target and fail if any target is missing/ambiguous (list target keys in error).

### Checksum acquisition (v0 decision)
- Always download the selected asset and compute SHA256 locally.
- Ignore checksum assets (`.sha256`, `.sha256sum`) in v0.

Spec details still needed:
- Size/time limits and retry/backoff rules for downloads.
- `--dry-run` behavior (skip downloads and print intended URL(s)).

### Version/tag precedence (v0 decision)
- If both `--version` and `--tag` are provided, normalize a leading `v` in the tag and require the values to match.
- Formula `version` should use the normalized version string (no `v` prefix).

Implementation details still needed:
- Exact validation regex/library and error messages for semver mismatches (still accept prerelease/build metadata).

## Release workflow details (v0)
- Validate clean working tree for CLI repo and tap repo (unless `--allow-dirty`).
- Resolve tap repo path; if missing but tap remote is known, auto-clone to a temp dir and warn/show path used; fail if no tap remote is configured.
- Determine new version/tag: prefer explicit `--version`/`--tag`; otherwise default to latest GitHub Release tag (stable by default; allow prereleases with `--include-prerelease`). Default tag format is `X.Y.Z`; tag creation is optional.
- Resolve `--version`/`--tag` precedence; if both are provided, validate the mapping (including `v` prefix rules).
- Update version in repo only if configured (default is no version file update).
- Create and push git tag by default; allow `--skip-tag`.
- Require a GitHub Release when `artifact.strategy = "release-asset"`; fail if missing. Use source tarball only when explicitly configured.
- Resolve asset(s) per target (universal or per-OS/arch) and build URL(s) using host strategy and tag/version.
- Compute SHA256 per resolved asset.
- Update formula url/version/sha (per target when configured).
- Commit and push tap repo by default; allow `--no-push` (commit message from template).
- Provide a summary with next steps and verification commands.
- Idempotency: if tag already exists and `--skip-tag` is not set, fail with guidance; if formula already targets the version, require `--force` to re-apply and continue.

## Git operations and safety
- Validate clean working trees (CLI repo + tap repo) unless `--allow-dirty`.
- Standardize commit messages via templates (simple defaults; configurable).
- Ensure tags are idempotent (fail if exists unless `--force-tag` is added later).
- Support dry-run paths that skip all writes and network requests.
- Detect and validate remotes for both repos; require explicit override when ambiguous.
- Push is default behavior; `--no-push` disables push for tags/commits.
- Multi-repo writes must be atomic: stage changes in temp files/branches and only commit after validation; clean up on failure.
- If tap repo must be cloned on-demand, clone into a temp dir with clear labeling, and delete on success or failure.

## Security and credentials
- Never log tokens or embed in config by default.
- GitHub API token (if used for repo creation) is read from env only.
- Private repos are out-of-scope for v0; `bd doctor` should warn if config implies private access.

## Error handling and safety
- Clear, actionable errors (missing config, auth failures, dirty git, network issues).
- Dry-run should print intended changes without writing or pushing.
- Never overwrite existing formula/config without confirmation.
- Validate required formula fields and naming conventions.
- Consistent exit codes for: missing config, invalid inputs, git state, network failure, audit failure.

### Exit code taxonomy (draft)
- 0: success
- 1: unexpected/internal error
- 2: missing config or required inputs
- 3: invalid user input (naming, license, version)
- 4: git state error (dirty, missing repo, ambiguous remote)
- 5: network error (fetch/sha/github API)
- 6: audit/validation failure (`bd doctor --audit` or formula validation)

## Testing approach
- Unit tests for config parsing and template rendering.
- Integration tests using temp dirs for `bd init` output.
- Release flow tests with mocked tarball URL and SHA function.
- Smoke tests that run `brew audit --new-formula` on generated formula (opt-in in CI).

## Architecture
### Core modules
- cli: command parsing and global flags (clap).
- prompts: interactive question flow (dialoguer).
- config: read/write config (serde + toml).
- repo_detect: infer repo metadata (name, owner, homepage, license, binary name).
- identity: resolve CLI/tap/artifact repo identities from config/flags/remotes.
- git: repository state, tagging, commit, push (libgit2 or shelling out).
- host: resolve tarball URL templates, metadata (trait-based for extensibility).
- github_api: create tap repo and query GitHub metadata (public repos only in v0).
- formula: template rendering, field validation.
- release: orchestrates update-version workflow.
- io: filesystem operations and atomic writes (temp files + rename).
- logging: structured logging for debug, terse console messages for normal output.
- errors: shared error types, exit codes, and user-facing hints.

### Templates
- Formula template with placeholders for name, url, sha256, version, desc, homepage, license, and install block.
- Optional script template if user wants a repo-local release helper (mirrors morningweave).

### Flexibility strategy
- Require only minimal repo info; allow overrides for every field.
- Avoid hard-coding repo layout beyond tap conventions.
- Host support via trait so GitLab/Bitbucket can be added without changing core flows.

## Architectural planning tasks
Status legend: open, in_progress, closed

Definition of done for tasks:
- A short design note (1-2 pages) with decisions, examples, and edge cases.
- Clear acceptance criteria that can become tests or validation checks later.
- Any new flags/config fields are explicitly listed with defaults and precedence.
- If user-facing errors are involved, include the exact error wording + exit code.

A) Identity and config
1) Define repo identity model (CLI/tap/artifact) and override rules; map to config fields and flags.
- Type: architecture
- Priority: high
- Status: in_progress
- DoD: mapping table (identity -> config/flags), precedence rules, and two examples (single-repo vs split-repo).

1b) Define tap repo local path resolution + on-demand clone strategy for release.
- Type: architecture
- Priority: high
- Status: in_progress
- DoD: default path rules, temp dir naming, auto-clone behavior when tap remote is known, cleanup behavior, and required flags/config additions.

1c) Define git remote selection and ambiguity rules for CLI/tap/artifact repos.
- Type: architecture
- Priority: high
- Status: open
- DoD: remote selection order, non-interactive failure behavior, and exact error messages.

2) Lock config schema for `.distill/config.toml` (fields, schema_version, migrations) and publish a minimal example.
- Type: architecture
- Priority: high
- Status: in_progress
- DoD: schema doc + minimal example + migration rules for unknown fields.

3) Define non-interactive contract per command (init/release/doctor): required fields, autodetect sources, and CI-friendly error messages.
- Type: product
- Priority: high
- Status: in_progress
- DoD: final table of required fields, autodetect sources, and exact error text + exit codes.

4) Specify `--import-formula` mapping: source-of-truth rules, merge strategy, and overwrite behavior.
- Type: architecture
- Priority: high
- Status: in_progress
- DoD: field-level merge table (formula wins for formula fields) + explicit overwrite prompts + non-interactive overwrite policy.

5) Define repository metadata detection (Rust-first): `Cargo.toml` parsing, bin targets, license detection, git fallback.
- Type: architecture
- Priority: medium
- Status: open
- DoD: edge-case examples (workspace, multi-bin, missing license/homepage).

5b) Define non-Rust metadata detection presets (Node/Go/Python/etc), bin mapping, and fallback rules.
- Type: architecture
- Priority: medium
- Status: open
- DoD: manifest list, field mapping table (including bin detection), conflict resolution, and required prompts.

B) Formula and templates
6) Specify formula naming rules and class name normalization (including acronyms/digits) with examples and validation.
- Type: architecture
- Priority: high
- Status: open
- DoD: transformation rules + at least 8 examples (acronyms, digits, dashes, underscores).

7) Define formula template(s), validation rules, and override mechanism (install block, template path, optional depends_on/livecheck).
- Type: architecture
- Priority: high
- Status: open
- DoD: default template + override contract + validation checks list + Ruby output examples (single/multi-bin, universal/per-target).

8) Define multi-binary support (decision: allow multiple) and prompt/config capture for `bin.install` entries.
- Type: product
- Priority: medium
- Status: in_progress
- DoD: prompt flow sketch + config schema impact.

9) Define formula `version` handling rules (explicit) and validation.
- Type: architecture
- Priority: medium
- Status: in_progress
- DoD: examples + validation rules.

C) Release and artifacts
10) Specify release asset selection rules (matching, tiebreakers, fallback, error messages) and OS/arch normalization table.
- Type: architecture
- Priority: high
- Status: open
- DoD: matching algorithm with `asset_name`/`asset_template` precedence, OS/arch table, and failure message examples.

10b) Define release discovery policy (GitHub Release required for release-asset; drafts/prereleases; missing release behavior).
- Type: architecture
- Priority: high
- Status: in_progress
- DoD: explicit rules (draft ignore, prerelease opt-in, latest tag default) + examples + exact error messages for missing releases/assets.

10c) Scope version update strategies and config fields (none/cargo/regex) and workspace handling rules.
- Type: architecture
- Priority: medium
- Status: in_progress
- DoD: mode definitions + config schema + examples for Cargo workspaces and regex updates.

10d) Specify cross-platform formula asset support (decision: per-OS/arch sections) and config shape.
- Type: architecture
- Priority: high
- Status: in_progress
- DoD: config schema impact + validation rules + example formula output for each supported mode (universal, per-OS, per-OS+arch).

10e) Define checksum acquisition strategy (decision: download + compute locally), size limits, and retry/backoff behavior.
- Type: architecture
- Priority: high
- Status: in_progress
- DoD: algorithm, limits/defaults, retry/backoff policy, and exact error messages for download/verify failures.

10f) Define version/tag precedence and validation rules (`--version`/`--tag`, decision: normalize `v` prefix, semver strictness).
- Type: architecture
- Priority: medium
- Status: in_progress
- DoD: precedence rules + examples + semver validation (prerelease/build allowed) + exit code mapping for mismatches.

11) Design release orchestration steps, dry-run behavior, idempotency rules, and summary output.
- Type: architecture
- Priority: high
- Status: open
- DoD: step-by-step flow + idempotency matrix + dry-run output sample + exact error messages/exit codes for common failure paths.

12) Decide git integration approach (libgit2 vs shell); define remote detection, push defaults, and error modes.
- Type: architecture
- Priority: high
- Status: open
- DoD: decision rationale + remote selection rules + error code mapping.

D) Host and API
13) Specify GitHub API tap repo creation flow (scopes, naming, public-only constraints, local fallback).
- Type: architecture
- Priority: medium
- Status: open
- DoD: scope list + API flow diagram + fallback behavior when token missing.

14) Define host abstraction (trait/interface) for future providers beyond GitHub.
- Type: architecture
- Priority: medium
- Status: open
- DoD: trait shape + required methods + versioning plan.

E) UX and safety
15) Design `bd init` prompt flow and non-interactive mapping; specify defaults, preview/edit/back, GitHub repo creation, and import/overwrite rules.
- Type: product
- Priority: high
- Status: open
- DoD: prompt sequence + field defaults + preview examples + overwrite decision points + `--yes/--force` semantics + `brew tap-new` availability/fallback behavior (missing brew/offline).

16) Define `bd doctor` checks, `--audit` integration, and failure modes.
- Type: product
- Priority: medium
- Status: open
- DoD: checklist of validations + mapping to exit codes.

17) Define error strategy, exit codes, and user-facing messages for common failures.
- Type: product
- Priority: medium
- Status: open
- DoD: error catalog with exact messages + exit codes.

18) Define preview/confirm UX contract and atomic write behavior for init/release.
- Type: product
- Priority: medium
- Status: open
- DoD: write-atomicity spec + rollback steps + temp file strategy.

18b) Specify preview/diff output for multi-repo changes (config + formula + tap repo).
- Type: product
- Priority: medium
- Status: open
- DoD: diff format examples + ordering rules + dry-run output contract.

F) Testing and docs
19) Plan testing matrix (init, release, doctor) with temp repos and mocked network/sha.
- Type: engineering
- Priority: medium
- Status: open
- DoD: test matrix table + minimal harness outline.

20) Define documentation outputs (tap README, CLI usage, doctor guidance, config examples).
- Type: product
- Priority: low
- Status: open
- DoD: list of docs + owner/source (generated vs hand-written).

Closed items (decisions already made)
21) Document tap layout enforcement (separate tap repo only), creation flow (`brew tap-new` vs manual), and detection rules.
- Type: architecture
- Priority: high
- Status: closed

22) Document release artifact default (release assets) and source tarball strategy (no implicit fallback).
- Type: product
- Priority: high
- Status: closed

23) Document cross-platform asset decision (per-OS/arch sections supported; universal asset allowed).
- Type: product
- Priority: high
- Status: closed

24) Document checksum policy decision (download + compute locally; ignore checksum assets in v0).
- Type: product
- Priority: medium
- Status: closed

25) Document version/tag normalization decision (leading `v` normalized; formula version uses normalized string).
- Type: product
- Priority: medium
- Status: closed

## Open questions (tracked in USER_TODO.md)
None currently; see USER_TODO.md if new decisions are added later.
