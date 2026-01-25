# USER_TODO

## Resolved TODOs (from USER_FEEDBACK on 2026-01-22T15:40:00)
- Included `.forge/ledgers/zesty-quimby.md` in a separate chore commit (2026-01-23).
- Committed the updated `.forge/ledgers/zesty-quimby.md` entry as a chore (2026-01-23).
- Confirmed uncommitted ledger changes should be committed as a separate chore (resolved 2026-01-23).
- Committed the latest `.forge/ledgers/zesty-quimby.md` update as `chore: update forge ledger` (7cba0c9) on 2026-01-25.

## Decisions needed
- Confirm Cargo workspace version update policy: current implementation updates root `[package]` by default, uses `[workspace.package]` if present and no `version_update.cargo_package`, otherwise requires `version_update.cargo_package` to target a member package.
- Confirm `--include-prerelease` latest-release behavior: current implementation selects the newest non-draft release even if it is a prerelease (i.e., latest overall, not "prefer stable").
