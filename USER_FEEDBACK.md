2026-01-27T14:23:45
Do not delete the timestamp above; it records the last read time for this file.

Feedback: bd release should support tag-first Homebrew releases (no GitHub Release required)

Context
- Current default strategy ("release-asset") fails when a repo only has tags, or if no tag is set.
- Desired flow: prompt for version, create tag if needed, update tap formula using tag tarball URL and optionally create a GitHub Release only when requested.

Requested tasks
- Change bd init defaults to set [artifact] strategy = "source-tarball" (tag tarball), or prompt and default to source-tarball in interactive mode.
- In interactive release, prompt for version when omitted; keep non-interactive strict.
- `bd release` should be interactive if no other input is given, assuming we want to update the version tags and asking for the necesaary inputs for deploying a release.
- Add an interactive choice for artifact strategy during release (tag tarball vs GitHub Release assets) and persist to config.
- Implement tag-first release flow for source-tarball:
  - Do not require a GitHub Release.
  - If --version is omitted in interactive mode, prompt; do not auto-query releases.
  - Create the tag (unless --skip-tag) before writing tap updates.
- Add optional GitHub Release creation for release-asset strategy:
  - Look for .gitconfig to see if we are able to commit and push to a remote github repo automatically. (Should be no need for specific GH_TOKEN)
  - New flags: --create-release / --no-create-release.
  - If release missing and create-release enabled: create tag (if needed), then create GitHub Release. Should be available in the interactive release mode as well.
  - Fail with a clear message when GITHUB_TOKEN/GH_TOKEN is missing.
- Update release preview output to explicitly state: will create tag, will update tap formula, and (if enabled) will create GitHub Release.
- Improve error messaging:
  - If release-asset and no release exists: suggest creating a release or switching to source-tarball.
  - If source-tarball and version missing in non-interactive mode: require --version.
- Update docs and examples to reflect tag-first defaults:
  - README, docs/config.example.toml, docs/release-discovery.md, docs/release-orchestration.md.
- Add tests for:
  - Interactive prompt for version when omitted.
  - Source-tarball flow does not require a GitHub Release.
  - Release-asset flow can create a GitHub Release when missing (mocked API).
