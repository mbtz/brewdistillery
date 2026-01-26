# USER_TODO

## Resolved TODOs (from USER_FEEDBACK on 2026-01-22T15:40:00)
- Included `.forge/ledgers/zesty-quimby.md` in a separate chore commit (2026-01-23).
- Committed the updated `.forge/ledgers/zesty-quimby.md` entry as a chore (2026-01-23).
- Confirmed uncommitted ledger changes should be committed as a separate chore (resolved 2026-01-23).
- Committed the latest `.forge/ledgers/zesty-quimby.md` update as `chore: update forge ledger` (7cba0c9) on 2026-01-25.

## Decisions needed
- [sv-b7j] Confirm Cargo workspace version update policy (as of 2026-01-25):
  current implementation updates root `[package]` by default, uses `[workspace.package]` if root is missing and no `version_update.cargo_package` is set, and otherwise requires `version_update.cargo_package` to target a member package.
- [sv-25d] Confirm `--include-prerelease` selection policy (as of 2026-01-25):
  choose one default: (A) latest overall non-draft release (current behavior), or (B) prefer the latest stable release when one exists, even when `--include-prerelease` is set.
