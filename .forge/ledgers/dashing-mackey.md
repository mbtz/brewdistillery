---
loop_id: 943ebafc-5849-483e-9c9f-5ff33e927238
loop_name: dashing-mackey
repo_path: /Users/marius/jobb/brewdistill/brewdistillery
created_at: 2026-01-22T09:54:33Z
---

# Loop Ledger: dashing-mackey

## 2026-01-22T09:56:15Z

- run_id: 4ca9de21-d4c3-4b86-8b08-986aea1a8447
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T09:54:33Z
- finished_at: 2026-01-22T09:56:15Z
- exit_code: 0

```
5) Tap layout: always separate `homebrew-<repo>` repo, or allow tap folder inside the main repo?
6) Version source: should `bd release` update `Cargo.toml` by default, or use a configured path/regex?
7) Should we generate a GitHub Actions workflow for releases in v0?
8) Should `bd init` support creating the tap repo on GitHub (API), or only scaffold locally?
9) Do we need support for private repos (auth tokens) in v0?
EOF' in /Users/marius/jobb/brewdistill/brewdistillery succeeded in 52ms:


thinking
**Checking documentation and status files**

I found no USER_FEEDBACK.md and will note its absence. I'll verify whether PRD and USER_TODO files exist and list them if present, then mention that workflow-pattern skill updates aren’t applicable here. Next, I'll summarize these findings and plan follow-up steps accordingly.
exec
/bin/zsh -lc 'ls -la' in /Users/marius/jobb/brewdistill/brewdistillery succeeded in 51ms:
total 40
drwxr-xr-x@ 9 marius  staff   288 Jan 22 10:56 .
drwxr-xr-x@ 4 marius  staff   128 Jan 22 10:14 ..
drwxr-xr-x@ 7 marius  staff   224 Jan 22 10:17 .forge
drwxr-xr-x@ 9 marius  staff   288 Jan 22 10:16 .git
-rw-r--r--@ 1 marius  staff     8 Jan 22 10:17 .gitignore
drwxr-xr-x@ 3 marius  staff    96 Jan 22 10:32 docs
-rw-r--r--@ 1 marius  staff  6204 Jan 22 10:55 PRD.md
-rw-r--r--@ 1 marius  staff  1590 Jan 22 10:54 PROMPT.md
-rw-r--r--@ 1 marius  staff   849 Jan 22 10:56 USER_TODO.md

thinking
**Using workflow-pattern skill and preparing final summary**
codex
Using skill workflow-pattern to follow team workflow expectations.

I drafted a full PRD from the current notes, filling scope, user journeys, CLI surface, architecture, and a concrete set of architectural planning tasks, then captured open design decisions in USER_TODO.md. I didn’t find a `USER_FEEDBACK.md` in the repo.

**Changes**
- New product planning document with critique of prior notes and a v0 architecture/task breakdown in `PRD.md`.
- Open decision list for you to answer in `USER_TODO.md`.

**Files**
- `PRD.md`
- `USER_TODO.md`

If you want, next steps could be:
1) Answer the questions in `USER_TODO.md` so I can tighten the PRD and lock v0 scope.
2) Ask me to refine the CLI command set or config schema once those decisions are made.
tokens used
19 807
Using skill workflow-pattern to follow team workflow expectations.

I drafted a full PRD from the current notes, filling scope, user journeys, CLI surface, architecture, and a concrete set of architectural planning tasks, then captured open design decisions in USER_TODO.md. I didn’t find a `USER_FEEDBACK.md` in the repo.

**Changes**
- New product planning document with critique of prior notes and a v0 architecture/task breakdown in `PRD.md`.
- Open decision list for you to answer in `USER_TODO.md`.

**Files**
- `PRD.md`
- `USER_TODO.md`

If you want, next steps could be:
1) Answer the questions in `USER_TODO.md` so I can tighten the PRD and lock v0 scope.
2) Ask me to refine the CLI command set or config schema once those decisions are made.
```

## 2026-01-22T09:58:52Z

- run_id: 28ef3dde-7312-4f12-9da7-a3972d3087d7
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T09:56:45Z
- finished_at: 2026-01-22T09:58:52Z
- exit_code: 0

```
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -11,3 +11,6 @@
 7) Should we generate a GitHub Actions workflow for releases in v0?
 8) Should `bd init` support creating the tap repo on GitHub (API), or only scaffold locally?
 9) Do we need support for private repos (auth tokens) in v0?
+10) Default formula naming: match repo name, binary name, or allow both with precedence?
+11) Release artifact strategy: use GitHub source tarball or require a specific release asset URL?
+12) Should v0 support multiple binaries in the formula `bin.install` block?



thinking
**Planning final response details**

I'll highlight using the workflow-pattern skill to follow team workflow, note no USER_FEEDBACK, and summarize updates to PRD and USER_TODO with key changes and next steps while deciding tests aren't needed.
thinking
**Outlining final response structure**

I'll include a brief bullet list requesting answers to new USER_TODO items and suggest next steps with a simple numeric list for clarity before crafting the final message.
codex
Using workflow-pattern to follow team workflow expectations.

Updated the PRD to tighten scope, UX/flow, config/architecture clarity, and to turn the architecture into explicit planning tasks with status/type/priority, then added new decision questions to `USER_TODO.md`.

- PRD critique now calls out concrete gaps and improvements; added constraints, UX contract, safety/idempotency, and clearer module boundaries in `PRD.md`
- Expanded scope/flows with `bd doctor`, precedence rules, and release safety details in `PRD.md`
- Planning tasks now include status/type/priority for clearer tracking in `PRD.md`
- Added three new decisions to unblock formula naming and release artifact strategy in `USER_TODO.md`

Next steps (pick any):
1) Answer the new USER_TODO items (10–12) so I can lock defaults.
2) Confirm whether you want a separate section in PRD for CLI flags matrix or keep it inline.
file update:
diff --git a/USER_TODO.md b/USER_TODO.md
index 31d3b9e1ba1a7eddc4676937cf976ef3d892ba85..e7907ec598ce6f75e66c9a9747ab78b431682b2e
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -11,3 +11,6 @@
 7) Should we generate a GitHub Actions workflow for releases in v0?
 8) Should `bd init` support creating the tap repo on GitHub (API), or only scaffold locally?
 9) Do we need support for private repos (auth tokens) in v0?
+10) Default formula naming: match repo name, binary name, or allow both with precedence?
+11) Release artifact strategy: use GitHub source tarball or require a specific release asset URL?
+12) Should v0 support multiple binaries in the formula `bin.install` block?

tokens used
23 014
Using workflow-pattern to follow team workflow expectations.

Updated the PRD to tighten scope, UX/flow, config/architecture clarity, and to turn the architecture into explicit planning tasks with status/type/priority, then added new decision questions to `USER_TODO.md`.

- PRD critique now calls out concrete gaps and improvements; added constraints, UX contract, safety/idempotency, and clearer module boundaries in `PRD.md`
- Expanded scope/flows with `bd doctor`, precedence rules, and release safety details in `PRD.md`
- Planning tasks now include status/type/priority for clearer tracking in `PRD.md`
- Added three new decisions to unblock formula naming and release artifact strategy in `USER_TODO.md`

Next steps (pick any):
1) Answer the new USER_TODO items (10–12) so I can lock defaults.
2) Confirm whether you want a separate section in PRD for CLI flags matrix or keep it inline.
```

## 2026-01-22T10:01:05Z

- run_id: 98589fa2-1267-42c6-a5c8-4cb79798403a
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T09:59:22Z
- finished_at: 2026-01-22T10:01:05Z
- exit_code: 0

```
- Status: open
 
-8) Define error strategy and user-facing messages for common failures.
+9) Define error strategy and user-facing messages for common failures.
 - Type: product
 - Priority: medium
 - Status: open
 
-9) Decide tap layout support (separate tap repo vs in-repo tap folder) and detection rules.
+10) Decide tap layout support (separate tap repo vs in-repo tap folder) and detection rules.
+- Type: architecture
+- Priority: medium
+- Status: open
+
+11) Define credential strategy for private repos and checksum fetching.
+- Type: architecture
+- Priority: medium
+- Status: open
+
+12) Define commit/tag conventions and safe git behaviors (push defaults, force rules).
 - Type: architecture
 - Priority: medium
 - Status: open
 
-10) Plan testing matrix (init, release, doctor) with temp repos and mocked network/sha.
+13) Plan testing matrix (init, release, doctor) with temp repos and mocked network/sha.
 - Type: engineering
 - Priority: medium
 - Status: open
 
+14) Decide whether to generate CI workflow (GitHub Actions) in v0 and its template.
+- Type: product
+- Priority: low
+- Status: open
+
 ## Open questions (tracked in USER_TODO.md)
 See USER_TODO.md for decisions needed before implementation.
diff --git a/USER_TODO.md b/USER_TODO.md
index e7907ec598ce6f75e66c9a9747ab78b431682b2e..3376e72193257fd59421d2e2695713bb68e73934
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -14,3 +14,6 @@
 10) Default formula naming: match repo name, binary name, or allow both with precedence?
 11) Release artifact strategy: use GitHub source tarball or require a specific release asset URL?
 12) Should v0 support multiple binaries in the formula `bin.install` block?
+13) Commit message conventions for formula and release bumps (defaults ok, or want custom templates)?
+14) Tag format: `vX.Y.Z` or `X.Y.Z`, and should tags be enforced or optional?
+15) For private repos, should tokens be allowed in config file or env-only in v0?

tokens used
23 699
Updated `PRD.md` with sharper critiques, added non‑functional requirements, formula rules, release artifact strategy, git safety, security posture, and expanded planning tasks. I also added new design decisions to `USER_TODO.md` to cover commit/tag conventions and private repo token handling.

Files touched:
- `PRD.md`
- `USER_TODO.md`

Next steps (pick any):
1) Answer the new questions in `USER_TODO.md` so we can lock the v0 scope.
2) If you want, I can draft a first-pass config schema and CLI flag mapping once those decisions are in.
```

## 2026-01-22T10:04:12Z

- run_id: 5322e1d8-ca1f-4ded-9e9e-568e65a6b782
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:01:35Z
- finished_at: 2026-01-22T10:04:12Z
- exit_code: 0

```
6) Version source: should `bd release` update `Cargo.toml` by default, or use a configured path/regex?
7) Should we generate a GitHub Actions workflow for releases in v0?
8) Should `bd init` support creating the tap repo on GitHub (API), or only scaffold locally?
9) Do we need support for private repos (auth tokens) in v0?
10) Default formula naming: match repo name, binary name, or allow both with precedence?
11) Release artifact strategy: use GitHub source tarball or require a specific release asset URL?
12) Should v0 support multiple binaries in the formula `bin.install` block?
13) Commit message conventions for formula and release bumps (defaults ok, or want custom templates)?
14) Tag format: `vX.Y.Z` or `X.Y.Z`, and should tags be enforced or optional?
15) For private repos, should tokens be allowed in config file or env-only in v0?
16) Where should the config file live by default: CLI repo root, tap repo root, or a dedicated config directory?
17) Should `bd init` scaffold the tap via `brew tap-new` (and git), or use a minimal manual layout?
18) Should `bd doctor` run `brew audit` by default, or keep it opt-in?
EOF
mv /tmp/user_todo_new.txt USER_TODO.md' in /Users/marius/jobb/brewdistill/brewdistillery succeeded in 52ms:



thinking
**Presenting refined PRD updates**

Used skill workflow-pattern to refine the PRD, tightening gaps and adding new planning tasks. Updated `PRD.md` with a sharper critique, a section on tap layout, repository detection details, and expanded the tasks list while adding the `repo_detect` module. Added new TODO questions 16–18; no tests were run as changes are documentation and planning focused. Next steps: review and answer the new TODO items or request any modifications.
thinking
**Suggesting next steps for TODO and config decisions**

1) Answer the new TODO items added in the PRD.  
2) Confirm command naming conventions and configuration file locations.  

No tests have been run since these are planning and documentation updates.
codex
Using skill: workflow-pattern (team workflow guidance).

Updated `PRD.md` to tighten the critique, clarify assumptions and scope, add a tap layout section, and expand the architecture/tasks list into clearer, actionable items. Added new decision questions to `USER_TODO.md` for config location, tap scaffolding method, and whether `bd doctor` should run `brew audit` by default.

Details
- Refined gaps and scope framing plus added repo metadata detection as a core module in `PRD.md`.
- Added tap layout expectations and Homebrew audit validation as explicit planning items in `PRD.md`.
- Appended new design decisions (items 16–18) to `USER_TODO.md`.

No tests run (docs-only change).

Next steps (pick one)
1) Answer the new items in `USER_TODO.md` (16–18) so I can lock the v0 plan.
2) Tell me if you want me to reshape the CLI surface or release flow now that the tasks are expanded.
tokens used
52 967
Using skill: workflow-pattern (team workflow guidance).

Updated `PRD.md` to tighten the critique, clarify assumptions and scope, add a tap layout section, and expand the architecture/tasks list into clearer, actionable items. Added new decision questions to `USER_TODO.md` for config location, tap scaffolding method, and whether `bd doctor` should run `brew audit` by default.

Details
- Refined gaps and scope framing plus added repo metadata detection as a core module in `PRD.md`.
- Added tap layout expectations and Homebrew audit validation as explicit planning items in `PRD.md`.
- Appended new design decisions (items 16–18) to `USER_TODO.md`.

No tests run (docs-only change).

Next steps (pick one)
1) Answer the new items in `USER_TODO.md` (16–18) so I can lock the v0 plan.
2) Tell me if you want me to reshape the CLI surface or release flow now that the tasks are expanded.
```

## 2026-01-22T10:07:51Z

- run_id: 41740b32-c089-49c9-9c97-50c4e11f9e27
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:04:42Z
- finished_at: 2026-01-22T10:07:51Z
- exit_code: 0

```
- Status: open
 
-12) Define error strategy and user-facing messages for common failures.
+12) Define error strategy, exit codes, and user-facing messages for common failures.
 - Type: product
 - Priority: medium
 - Status: open
 
-13) Define credential strategy for private repos and checksum fetching.
+13) Define credential strategy for private repos and checksum fetching (env vs config storage).
 - Type: architecture
 - Priority: medium
 - Status: open
 
-14) Define commit/tag conventions and safe git behaviors (push defaults, force rules).
+14) Define commit/tag conventions and safe git behaviors (push defaults, force rules, commit templates).
 - Type: architecture
 - Priority: medium
 - Status: open
@@ -285,5 +313,10 @@
 - Priority: low
 - Status: open
 
+18) Define documentation outputs (tap README, CLI usage, doctor guidance).
+- Type: product
+- Priority: low
+- Status: open
+
 ## Open questions (tracked in USER_TODO.md)
 See USER_TODO.md for decisions needed before implementation.
diff --git a/USER_TODO.md b/USER_TODO.md
index e737d825561c68987a2575e3449e0f0798c86cbe..d7ee4c279e04d00aa7413dd64d0528c67cda19ec
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -20,3 +20,5 @@
 16) Where should the config file live by default: CLI repo root, tap repo root, or a dedicated config directory?
 17) Should `bd init` scaffold the tap via `brew tap-new` (and git), or use a minimal manual layout?
 18) Should `bd doctor` run `brew audit` by default, or keep it opt-in?
+19) Should `bd init` support importing an existing formula (read/validate) instead of always generating a new one?
+20) Should `bd init` generate a repo-local helper script (like the morningweave update-version flow), or keep all release logic inside `bd`?

tokens used
31 322
Using skill: workflow-pattern (follow team workflow expectations).

Updated the PRD to tighten gaps, add risk/mitigation notes, clarify UX and config rules, and expand architectural planning tasks, plus added two new decision questions for you.

Details
- Strengthened the critique and added missing concerns (existing formula handling, git/remote safety, docs, exit codes) in `PRD.md`.
- Added a risks/mitigations section, expanded UX rules, and clarified config schema/migration rules in `PRD.md`.
- Expanded and re-scoped the architectural planning task list with clearer boundaries in `PRD.md`.
- Added new explicit decisions about importing existing formulas and generating a helper script in `USER_TODO.md`.

Files touched
- `PRD.md`
- `USER_TODO.md`

Next steps (pick one)
1) Answer the new questions in `USER_TODO.md`.
2) Tell me which of the existing TODO items you want prioritized so I can convert them into concrete implementation tasks.
```

## 2026-01-22T10:13:16Z

- run_id: c98d9bff-2eda-4551-b53d-6ded66bb504b
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:08:21Z
- finished_at: 2026-01-22T10:13:16Z
- exit_code: 0

```
- Type: product
 - Priority: high
 - Status: open
 
-10) Design release orchestration steps, dry-run behavior, and rollback/idempotency rules.
+10) Design release orchestration steps, dry-run behavior, and rollback/idempotency rules (default push, optional tag skip).
 - Type: architecture
 - Priority: high
 - Status: open
@@ -288,7 +298,7 @@
 - Priority: medium
 - Status: open
 
-13) Define credential strategy for private repos and checksum fetching (env vs config storage).
+13) Define credential strategy for GitHub API usage (env-only) and guardrails for private repo configs.
 - Type: architecture
 - Priority: medium
 - Status: open
@@ -306,7 +316,7 @@
 16) Decide whether to generate CI workflow (GitHub Actions) in v0 and its template.
 - Type: product
 - Priority: low
-- Status: open
+- Status: closed
 
 17) Define Homebrew audit/style validation steps and when to run them.
 - Type: engineering
@@ -318,5 +328,20 @@
 - Priority: low
 - Status: open
 
+19) Specify GitHub API tap repo creation flow (scopes, naming, public-only constraints, local fallback).
+- Type: architecture
+- Priority: medium
+- Status: open
+
+20) Decide release artifact default (GitHub release asset vs source tarball) and fallback logic.
+- Type: product
+- Priority: high
+- Status: open
+
+21) Decide multi-binary support and prompt/config capture for multiple `bin.install` entries.
+- Type: product
+- Priority: medium
+- Status: open
+
 ## Open questions (tracked in USER_TODO.md)
 See USER_TODO.md for decisions needed before implementation.

tokens used
62 362
Updated the planning docs to reflect your decisions, tighten the scope, and make the remaining gaps explicit so we can turn them into precise tasks.

- `PRD.md` now records the locked decisions (GitHub-only v0, `distill.toml`, `bd ship` alias, default push, no private repos/CI), clarifies assumptions/risks, adjusts CLI flags, and expands the planning task list to highlight the remaining open choices.
- `USER_TODO.md` is split into resolved decisions and open questions that still need your input.
- `USER_FEEDBACK.md` cleared per your instruction.

Next steps:
1) Review and answer the open items in `USER_TODO.md` (tap layout, release artifact default, multi-binary support, commit/tag conventions, config location, etc.).
2) Tell me if you want me to fold those answers back into `PRD.md` and lock the v0 plan.
```

## 2026-01-22T10:16:26Z

- run_id: e7c9c883-c827-488f-ad5e-22aef8af0cd0
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:13:46Z
- finished_at: 2026-01-22T10:16:26Z
- exit_code: 0

```
@@ -323,7 +329,7 @@
 - Priority: low
 - Status: open
 
-18) Define documentation outputs (tap README, CLI usage, doctor guidance).
+18) Define documentation outputs (tap README, CLI usage, doctor guidance, config examples).
 - Type: product
 - Priority: low
 - Status: open
@@ -343,5 +349,20 @@
 - Priority: medium
 - Status: open
 
+22) Define non-interactive required field list and error messages (CI-friendly).
+- Type: product
+- Priority: medium
+- Status: open
+
+23) Define formula `version` handling rules (explicit vs derived from URL) and validation.
+- Type: architecture
+- Priority: medium
+- Status: open
+
+24) Define preview/confirm UX contract and atomic write behavior for init/release.
+- Type: product
+- Priority: medium
+- Status: open
+
 ## Open questions (tracked in USER_TODO.md)
 See USER_TODO.md for decisions needed before implementation.
diff --git a/USER_TODO.md b/USER_TODO.md
index f223effaa7479c7d9c4ca22eedc4c0e59dcf8b6a..86cc20d908a6954f0506d185f1b64bfec9c782bc
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -23,3 +23,4 @@
 8) `bd doctor`: run `brew audit` by default or opt-in (`--audit`)?
 9) `bd init` import mode: support reading/validating an existing formula instead of always generating?
 10) Generate a repo-local helper script (like morningweave) or keep all release logic inside `bd`?
+11) Formula `version` handling: always include an explicit `version` stanza, or derive from URL when possible?

tokens used
29 881
Updated the PRD to make the planning phase more explicit and actionable, and captured a new user decision item. I also noted the current planning status inline as requested.

Changes made:
- Tightened the critique section with concrete gaps (UX contract, safety, git/tag rules, audit checks, docs) and added an explicit planning status line in `PRD.md`.
- Clarified UX/non-interactive contract expectations, error/exit-code needs, and formula `version` handling in `PRD.md`.
- Expanded the architectural planning tasks with new items for non-interactive requirements, version handling, and preview/confirm behavior in `PRD.md`.
- Added a new decision request about formula `version` handling to `USER_TODO.md`.

Files touched:
- `PRD.md`
- `USER_TODO.md`

Note: I didn’t find an `AGENTS.md` in the repo root.

Next steps (pick any):
1) Answer the open decisions in `USER_TODO.md` (especially #1–#3, #5–#8, and #11).
2) I can draft a concrete non-interactive required-fields table and prompt-to-flag mapping for `bd init`.
3) I can draft a formula naming/class normalization spec (including acronyms/digits) for the PRD.
```

## 2026-01-22T10:19:46Z

- run_id: 06ec006d-7151-4806-a6ea-e5a2bf975933
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:16:56Z
- finished_at: 2026-01-22T10:19:46Z
- exit_code: 0

```
- `bd doctor`: keep `brew audit` opt-in (`--audit`).
- `bd init`: detect existing setup and avoid overwriting without explicit confirmation.
- Helper script: no; keep release logic inside `bd`.

## Decisions needed
1) Tap layout: should v0 only support a separate `homebrew-<repo>` tap repo, or also allow an in-repo tap folder?
2) Release artifact strategy: default to GitHub release asset URL (recommended for binaries) or GitHub source tarball? If both, which is default and what is fallback logic?
3) Multi-binary formula support: allow multiple `bin.install` entries in v0 or defer? If allowed, how should prompts/config capture multiple names?
4) Formula `version` handling: always include an explicit `version` stanza, or derive from URL when possible?
EOF' in /Users/marius/jobb/brewdistill/brewdistillery succeeded in 51ms:



thinking
**Summarizing updates and next steps**

The PRD and USER_TODO were rewritten with decisions, critiques, and config path updates; open questions remain in USER_TODO. No tests were run. Next steps include reviewing PRD and USER_TODO, considering unresolved questions, and deciding on further actions. Changed files are `PRD.md` and `USER_TODO.md`.
thinking
**Providing next steps list**

Next steps: 1) answer USER_TODO decisions, 2) review PRD for accuracy, 3) optionally break tasks into tickets or start config schema; updates include explanations and file references.
codex
Updated the product plan to incorporate your latest decisions, tighten the critique, and make the architecture tasks more actionable. I also cleaned up the outstanding decision list so it only contains open items.

Key updates
- Consolidated v0 decisions into **Decisions locked** and removed conflicting/duplicate notes in `PRD.md`.
- Fixed config location to `.distill/config.toml` and aligned UX/CLI/architecture sections to that path in `PRD.md`.
- Clarified commit/tag defaults, `brew tap-new` optionality, `bd doctor --audit` opt‑in, and “no helper script” in `PRD.md`.
- Reduced USER_TODO to the remaining product decisions in `USER_TODO.md`.

Files changed
- `PRD.md`
- `USER_TODO.md`

No tests run (planning/doc edits only).

Next steps (pick any)
1) Answer the remaining decisions in `USER_TODO.md`.
2) I can turn the architectural planning tasks in `PRD.md` into a first ticket breakdown.
3) I can draft the initial `.distill/config.toml` schema and required fields list.
tokens used
55 716
Updated the product plan to incorporate your latest decisions, tighten the critique, and make the architecture tasks more actionable. I also cleaned up the outstanding decision list so it only contains open items.

Key updates
- Consolidated v0 decisions into **Decisions locked** and removed conflicting/duplicate notes in `PRD.md`.
- Fixed config location to `.distill/config.toml` and aligned UX/CLI/architecture sections to that path in `PRD.md`.
- Clarified commit/tag defaults, `brew tap-new` optionality, `bd doctor --audit` opt‑in, and “no helper script” in `PRD.md`.
- Reduced USER_TODO to the remaining product decisions in `USER_TODO.md`.

Files changed
- `PRD.md`
- `USER_TODO.md`

No tests run (planning/doc edits only).

Next steps (pick any)
1) Answer the remaining decisions in `USER_TODO.md`.
2) I can turn the architectural planning tasks in `PRD.md` into a first ticket breakdown.
3) I can draft the initial `.distill/config.toml` schema and required fields list.
```

## 2026-01-22T10:26:04Z

- run_id: 56b29846-c47a-4292-99e8-2421c204a102
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:20:16Z
- finished_at: 2026-01-22T10:26:04Z
- exit_code: 0

```
- Priority: low
 - Status: open
 
-18) Define documentation outputs (tap README, CLI usage, doctor guidance, config examples).
+17) Define documentation outputs (tap README, CLI usage, doctor guidance, config examples).
 - Type: product
 - Priority: low
 - Status: open
 
-19) Specify GitHub API tap repo creation flow (scopes, naming, public-only constraints, local fallback).
+18) Specify GitHub API tap repo creation flow (scopes, naming, public-only constraints, local fallback).
 - Type: architecture
 - Priority: medium
 - Status: open
 
-20) Decide release artifact default (GitHub release asset vs source tarball) and fallback logic.
+19) Decide release artifact default (GitHub release asset vs source tarball) and fallback logic.
 - Type: product
 - Priority: high
 - Status: open
 
-21) Decide multi-binary support and prompt/config capture for multiple `bin.install` entries.
+20) Decide multi-binary support and prompt/config capture for multiple `bin.install` entries.
 - Type: product
 - Priority: medium
 - Status: open
 
+21) Define formula `version` handling rules (explicit vs derived from URL) and validation.
+- Type: architecture
+- Priority: medium
+- Status: open
+
 22) Define non-interactive required field list and error messages (CI-friendly).
 - Type: product
 - Priority: medium
 - Status: open
 
-23) Define formula `version` handling rules (explicit vs derived from URL) and validation.
-- Type: architecture
-- Priority: medium
-- Status: open
+23) Decide whether to generate CI workflow (GitHub Actions) in v0 and its template.
+- Type: product
+- Priority: low
+- Status: closed
 
 24) Define preview/confirm UX contract and atomic write behavior for init/release.
 - Type: product

tokens used
71 679
Using workflow-pattern skill to follow team workflow expectations.

Updated the PRD to clarify the repo identity model (CLI/tap/artifact), add a draft config shape + example, strengthen UX preview requirements, and reorder planning tasks with in_progress statuses where we started tightening definitions. `PRD.md`

Tests not run (docs-only change).

Next steps:
1) Decide the four open product questions in `USER_TODO.md` (tap layout, artifact strategy, multi-binary, version handling).
2) If you want, I can draft the non-interactive required fields list and a concrete `bd init` prompt flow spec (tasks 3 and 10).
```

## 2026-01-22T10:29:54Z

- run_id: 40853ba7-81ec-46d7-a0da-c2a038a5fefd
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:26:34Z
- finished_at: 2026-01-22T10:29:54Z
- exit_code: 0

```
- Type: architecture
 - Priority: medium
-- Status: open
+- Status: in_progress
 
 15) Plan testing matrix (init, release, doctor) with temp repos and mocked network/sha.
 - Type: engineering
@@ -400,10 +406,10 @@
 - Priority: medium
 - Status: open
 
-19) Decide release artifact default (GitHub release asset vs source tarball) and fallback logic.
+19) Document release artifact default (release assets) and fallback logic (source tarball).
 - Type: product
 - Priority: high
-- Status: open
+- Status: closed
 
 20) Decide multi-binary support and prompt/config capture for multiple `bin.install` entries.
 - Type: product
diff --git a/USER_TODO.md b/USER_TODO.md
index e94d6ef82dae7cd6fe2355f2b31c0b2133c1a543..450f9c136a4bee0c38e0ebee135fb9bb49f6c50e
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -10,15 +10,17 @@
 - CI workflow generation in v0: no.
 - Version update: default does not update main repo version file; allow configurable updater per language/regex.
 - Config location: `.distill/config.toml` in the main repo; `.distill/` directory for related files.
-- Commit message conventions: default conventional commit for release bump.
+- Commit message conventions: configurable templates with a simple default (e.g. `feature: Updated formula for version 'X.Y.Z'`).
 - Tag format: default semver `X.Y.Z`; tags are optional.
 - Tap scaffolding: prefer `brew tap-new`, but keep a flag to choose manual layout.
 - `bd doctor`: keep `brew audit` opt-in (`--audit`).
 - `bd init`: detect existing setup and avoid overwriting without explicit confirmation.
 - Helper script: no; keep release logic inside `bd`.
+- Tap layout: enforce a separate `homebrew-<repo>` tap repo in v0.
+- Release artifact strategy: default to GitHub release assets for binaries, with source tarball as fallback.
 
 ## Decisions needed
-1) Tap layout: should v0 only support a separate `homebrew-<repo>` tap repo, or also allow an in-repo tap folder?
-2) Release artifact strategy: default to GitHub release asset URL (recommended for binaries) or GitHub source tarball? If both, which is default and what is fallback logic?
-3) Multi-binary formula support: allow multiple `bin.install` entries in v0 or defer? If allowed, how should prompts/config capture multiple names?
-4) Formula `version` handling: always include an explicit `version` stanza, or derive from URL when possible?
+1) Multi-binary formula support: allow multiple `bin.install` entries in v0 or defer?
+   - Suggested: allow multiple via config array; interactive flow collects one name, then optionally adds more (loop). Default remains single-bin for simplicity.
+2) Formula `version` handling: always include an explicit `version` stanza, or derive from URL when possible?
+   - Suggested: include explicit `version` by default for determinism; allow `version_mode = "derive"` in config to omit when URL contains the version.

tokens used
64 464
Integrated your feedback into the product plan: `PRD.md` now locks the separate tap‑repo decision, sets release assets as default with tarball fallback, switches commit messages to configurable templates, updates the critique/gaps, adds a release config example, and marks the relevant planning tasks closed/in‑progress. `USER_TODO.md` now reflects the resolved decisions and includes suggested defaults for the two remaining open decisions.  
AGENTS.md was not found in `/Users/marius/jobb/brewdistill/brewdistillery`, so no extra instructions were applied.

Files updated:
- `PRD.md`
- `USER_TODO.md`

Next steps (pick any):
1) Decide on multi‑binary support in v0 (I suggested “allow via config array; optional add‑more loop in prompts”).  
2) Decide formula `version` handling (I suggested “explicit `version` by default; optional derive mode”).
```

## 2026-01-22T10:32:38Z

- run_id: 45050f3a-a583-41d8-a57d-656083371061
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:30:24Z
- finished_at: 2026-01-22T10:32:38Z
- exit_code: 0

```
## Release workflow details (v0)
 - Validate clean working tree for CLI repo and tap repo (unless `--allow-dirty`).
 - Determine new version (prompt or flag). Default tag format is `X.Y.Z`; tag creation is optional.
@@ -366,6 +392,11 @@
 - Priority: high
 - Status: open
 
+10a) Define non-interactive required field list for each command (init/release/doctor) and map flags to config/auto-detect.
+- Type: product
+- Priority: high
+- Status: open
+
 11) Define repository metadata detection rules (name, owner, homepage, license, bin name) and fallbacks.
 - Type: architecture
 - Priority: medium
@@ -406,6 +437,11 @@
 - Priority: medium
 - Status: open
 
+18a) Specify release asset selection rules (matching, fallback, error messages) and OS/arch normalization table.
+- Type: architecture
+- Priority: medium
+- Status: open
+
 19) Document release artifact default (release assets) and fallback logic (source tarball).
 - Type: product
 - Priority: high
diff --git a/USER_TODO.md b/USER_TODO.md
index 450f9c136a4bee0c38e0ebee135fb9bb49f6c50e..5e42e1ad28b81a1f1e8a4168f513052d0ca10ef5
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -21,6 +21,8 @@
 
 ## Decisions needed
 1) Multi-binary formula support: allow multiple `bin.install` entries in v0 or defer?
+   - Please choose: (A) allow multiple in v0 or (B) defer to v1.
    - Suggested: allow multiple via config array; interactive flow collects one name, then optionally adds more (loop). Default remains single-bin for simplicity.
 2) Formula `version` handling: always include an explicit `version` stanza, or derive from URL when possible?
+   - Please choose: (A) always include explicit `version` or (B) derive from URL by default.
    - Suggested: include explicit `version` by default for determinism; allow `version_mode = "derive"` in config to omit when URL contains the version.

tokens used
25 000
AGENTS.md wasn’t found under `/Users/marius/jobb/brewdistill/brewdistillery` or parents. If it lives elsewhere, point me to the exact path.

Updated the plan to be more critical and actionable, and captured the remaining product decisions.
- Added explicit gaps/risks and improvements in `PRD.md`, plus a draft non‑interactive required‑fields section and release asset selection rules in `PRD.md`.
- Inserted two new planning tasks for non‑interactive requirements and asset selection/OS‑arch mapping in `PRD.md`.
- Made the open decisions explicit in `USER_TODO.md` with a clear A/B choice for each item.

Files touched:
- `PRD.md`
- `USER_TODO.md`

If you want, I can refine any of the new sections into tighter tables or checklists.

Next steps (pick any):
1) Decide A/B for multi‑binary support and version handling in `USER_TODO.md`.
2) Review the new non‑interactive required‑fields draft in `PRD.md` and confirm required vs optional inputs.
3) Confirm the release asset selection rules and OS/arch mapping expectations in `PRD.md`.
```

## 2026-01-22T10:37:38Z

- run_id: d0fa8357-33a9-4d20-bafa-15b78f3ec673
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:33:08Z
- finished_at: 2026-01-22T10:37:38Z
- exit_code: 0

```
-- Type: product
-- Priority: high
-- Status: closed
-
-20) Decide multi-binary support and prompt/config capture for multiple `bin.install` entries.
+18) Define preview/confirm UX contract and atomic write behavior for init/release.
 - Type: product
 - Priority: medium
 - Status: open
 
-21) Define formula `version` handling rules (explicit vs derived from URL) and validation.
-- Type: architecture
+F) Testing and docs
+19) Plan testing matrix (init, release, doctor) with temp repos and mocked network/sha.
+- Type: engineering
 - Priority: medium
 - Status: open
 
-22) Define non-interactive required field list and error messages (CI-friendly).
+20) Define documentation outputs (tap README, CLI usage, doctor guidance, config examples).
 - Type: product
-- Priority: medium
+- Priority: low
 - Status: open
 
-23) Decide whether to generate CI workflow (GitHub Actions) in v0 and its template.
-- Type: product
-- Priority: low
+Closed items (decisions already made)
+21) Document tap layout enforcement (separate tap repo only), creation flow (`brew tap-new` vs manual), and detection rules.
+- Type: architecture
+- Priority: high
 - Status: closed
 
-24) Define preview/confirm UX contract and atomic write behavior for init/release.
+22) Document release artifact default (release assets) and fallback logic (source tarball).
 - Type: product
-- Priority: medium
-- Status: open
+- Priority: high
+- Status: closed
 
 ## Open questions (tracked in USER_TODO.md)
 See USER_TODO.md for decisions needed before implementation.

tokens used
42 660
Updated the PRD to make the critique sharper, add missing Rust-first detection and naming/idempotency details, and reorganize the planning tasks into atomic, testable items with clearer ownership and status.

- `PRD.md`: strengthened the critique section with concrete gaps (Rust metadata detection, idempotency, import-formula source of truth, deterministic asset matching).
- `PRD.md`: added a Rust-first repo metadata detection draft plus a formula name normalization draft.
- `PRD.md`: tightened non-interactive requirements to reference Cargo.toml/LICENCE autodetect paths and added idempotency rules to release flow.
- `PRD.md`: expanded asset selection rules with exclusions and tiebreakers; consolidated and cleaned up the planning task list.

Note: no `AGENTS.md` found in `/Users/marius/jobb/brewdistill/brewdistillery`, so I proceeded without repo-specific agent instructions.

Next steps (pick any that you want me to formalize):
1) Decide multi-binary support for v0 (allow multiple `bin.install` vs defer).
2) Decide formula `version` handling policy (explicit stanza vs derived from URL by default).
3) Decide git integration approach (libgit2 vs shell), so the release/orchestration tasks can be locked.
```

## 2026-01-22T10:40:04Z

- run_id: 5203fa4b-332b-4298-9bc1-bd5a3c9837e3
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:38:08Z
- finished_at: 2026-01-22T10:40:04Z
- exit_code: 0

```
-- No CI workflow generation, no private repo support, release artifacts assumed to exist.
+- Out-of-scope items are explicit (no CI workflow generation, no private repos, no bottle pipeline).
 - GitHub API tap creation is in scope; release automation lives inside `bd` (no helper script).
 - Config lives in the main repo under `.distill/` (path now fixed).
+- Tap layout and artifact strategy defaults are now aligned with user feedback.
 
 Needs strengthening / missing detail:
-- Task list has duplicated items and mixed granularity; needs cleanup into atomic, testable tasks.
-- Rust-first repo metadata detection (Cargo.toml fields, bin targets, license) is not specified; weak defaults risk poor UX.
-- Formula naming/class normalization is still underspecified; needs explicit algorithm + examples (acronyms, digits, separators).
-- Release idempotency rules are incomplete (re-release same version, existing tag, formula already updated).
+- Task list still needs more "definition of done" detail to become directly actionable.
+- Rust-first repo metadata detection (Cargo.toml fields, bin targets, license) needs edge-case examples.
+- Formula naming/class normalization still needs a final acronym policy and explicit examples.
+- Release idempotency rules are incomplete (tag already exists, formula already at version, re-run behavior).
 - `bd init --import-formula` source-of-truth mapping and overwrite rules are unclear.
-- Asset selection and OS/arch mapping need deterministic matching + tiebreakers.
+- Asset selection and OS/arch mapping need deterministic matching + explicit tiebreakers.
 - Non-interactive required fields should be a per-command table with autodetect sources and failure modes.
+- Multi-repo write/rollback expectations need to be explicit (atomic writes, temp files, fail-safe cleanup).
 
 Improvements added in this revision:
-- Added Rust metadata detection draft, name normalization draft, and explicit idempotency rules.
-- Consolidated and expanded architectural planning tasks; removed duplicates and clarified ownership.
-- Added an import-formula mapping task and asset selection tiebreaker requirements.
-- Strengthened required-field detection notes (Cargo.toml, LICENSE, GitHub remote).
+- Folded in confirmed user decisions (separate tap repo, release asset default, commit message template).
+- Expanded the "definition of done" expectations for planning tasks to make them more testable.
+- Added explicit notes on multi-repo atomicity and rollback expectations.
+- Tightened non-interactive contract language to require a per-command requirements table.
 
 Remaining execution gaps before implementation (tracked in USER_TODO.md):
 - Multi-binary support decision (v0 vs v1).
@@ -311,6 +313,7 @@
 - Support dry-run paths that skip all writes and network requests.
 - Detect and validate remotes for both repos; require explicit override when ambiguous.
 - Push is default behavior; `--no-push` disables push for tags/commits.
+- Multi-repo writes must be atomic: stage changes in temp files/branches and only commit after validation; clean up on failure.
 
 ## Security and credentials
 - Never log tokens or embed in config by default.
@@ -358,6 +361,10 @@
 ## Architectural planning tasks
 Status legend: open, in_progress, closed
 
+Definition of done for tasks:
+- A short design note (1-2 pages) with decisions, examples, and edge cases.
+- Clear acceptance criteria that can become tests or validation checks later.
+
 A) Identity and config
 1) Define repo identity model (CLI/tap/artifact) and override rules; map to config fields and flags.
 - Type: architecture

tokens used
26 404
Updated `PRD.md` to be more critical/constructive: tightened the critique, folded in the confirmed decisions from `USER_FEEDBACK.md`, added explicit gaps (multi‑repo atomicity, idempotency edge cases), and added a “definition of done” for planning tasks so they become directly actionable. The changes are all in `PRD.md`.

Next steps if you want to unblock planning:
1) Decide multi-binary support (A/B in `USER_TODO.md`).
2) Decide formula `version` handling (A/B in `USER_TODO.md`).
```

## 2026-01-22T10:43:52Z

- run_id: b3ddbe07-0e8e-4bed-8846-d8933434325b
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:40:34Z
- finished_at: 2026-01-22T10:43:52Z
- exit_code: 0

```
- Type: product
 - Priority: medium
 - Status: open
+- DoD: error catalog with exact messages + exit codes.
 
 18) Define preview/confirm UX contract and atomic write behavior for init/release.
 - Type: product
 - Priority: medium
 - Status: open
+- DoD: write-atomicity spec + rollback steps + temp file strategy.
 
 F) Testing and docs
 19) Plan testing matrix (init, release, doctor) with temp repos and mocked network/sha.
 - Type: engineering
 - Priority: medium
 - Status: open
+- DoD: test matrix table + minimal harness outline.
 
 20) Define documentation outputs (tap README, CLI usage, doctor guidance, config examples).
 - Type: product
 - Priority: low
 - Status: open
+- DoD: list of docs + owner/source (generated vs hand-written).
 
 Closed items (decisions already made)
 21) Document tap layout enforcement (separate tap repo only), creation flow (`brew tap-new` vs manual), and detection rules.
diff --git a/USER_TODO.md b/USER_TODO.md
index 5e42e1ad28b81a1f1e8a4168f513052d0ca10ef5..b27e2fc9605a0987184f07b0e4e4f1db166e6f30
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -22,7 +22,9 @@
 ## Decisions needed
 1) Multi-binary formula support: allow multiple `bin.install` entries in v0 or defer?
    - Please choose: (A) allow multiple in v0 or (B) defer to v1.
-   - Suggested: allow multiple via config array; interactive flow collects one name, then optionally adds more (loop). Default remains single-bin for simplicity.
+   - Suggested: (A) allow multiple in v0 via `bin = ["foo","bar"]`; prompt collects one name, then asks "Add another?" in a loop. Default remains single-bin for simplicity.
+   - Why: Rust projects commonly have multiple bins; supporting a list is a small scope increase and keeps formulas correct.
 2) Formula `version` handling: always include an explicit `version` stanza, or derive from URL when possible?
    - Please choose: (A) always include explicit `version` or (B) derive from URL by default.
-   - Suggested: include explicit `version` by default for determinism; allow `version_mode = "derive"` in config to omit when URL contains the version.
+   - Suggested: (A) explicit `version` by default for determinism; allow `version_mode = "derive"` in config to omit when URL contains the version.
+   - Why: avoids surprises when URL templates change and makes diffs clearer.

tokens used
35 837
No AGENTS.md found in `/Users/marius/jobb/brewdistill/brewdistillery`, so I followed the repo files and your prompt only.

Updated planning docs to tighten the PRD into actionable architecture tasks and captured your pending decisions with recommendations.
- Expanded the critique and added concrete missing pieces (non‑interactive contract, idempotency matrix, OS/arch normalization, exit codes). See `PRD.md`.
- Added draft tables for non‑interactive requirements and idempotency/overwrite behavior, plus a draft OS/arch mapping and exit‑code taxonomy. See `PRD.md`.
- Strengthened each planning task with explicit definition‑of‑done guidance so they can turn into testable specs. See `PRD.md`.
- Added recommendations for multi‑binary support and `version` handling to the decisions list. See `USER_TODO.md`.

Files touched:
- `PRD.md`
- `USER_TODO.md`

Next steps (pick one or more):
1) Confirm decision 1 and 2 in `USER_TODO.md` (A/B choices) and I’ll lock those into the PRD.
2) If you want, I can add an explicit decision line for git integration (libgit2 vs shell) and update the PRD accordingly.
```

## 2026-01-22T10:49:51Z

- run_id: 36efb985-225c-4504-a260-a0c95d42a53f
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:44:22Z
- finished_at: 2026-01-22T10:49:51Z
- exit_code: 0

```
2) Lock config schema for `.distill/config.toml` (fields, schema_version, migrations) and publish a minimal example.
 - Type: architecture
 - Priority: high
@@ -465,7 +483,7 @@
 - Type: architecture
 - Priority: high
 - Status: open
-- DoD: matching algorithm + OS/arch table + failure message examples.
+- DoD: matching algorithm with `asset_name`/`asset_template` precedence, OS/arch table, and failure message examples.
 
 11) Design release orchestration steps, dry-run behavior, idempotency rules, and summary output.
 - Type: architecture
@@ -497,7 +515,7 @@
 - Type: product
 - Priority: high
 - Status: open
-- DoD: prompt sequence + field defaults + preview examples + overwrite decision points.
+- DoD: prompt sequence + field defaults + preview examples + overwrite decision points + `--yes/--force` semantics.
 
 16) Define `bd doctor` checks, `--audit` integration, and failure modes.
 - Type: product
diff --git a/USER_TODO.md b/USER_TODO.md
index b27e2fc9605a0987184f07b0e4e4f1db166e6f30..c21deabb750c58ff93b348138ea36794886859ca
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -28,3 +28,11 @@
    - Please choose: (A) always include explicit `version` or (B) derive from URL by default.
    - Suggested: (A) explicit `version` by default for determinism; allow `version_mode = "derive"` in config to omit when URL contains the version.
    - Why: avoids surprises when URL templates change and makes diffs clearer.
+3) Formula class acronym casing policy: how should `bd` treat acronyms when generating the Ruby class name?
+   - Please choose: (A) simple CamelCase of segments (`http-server` -> `HttpServer`), (B) preserve a short acronym list (`HTTP`, `CLI`, `URL`) so `http-server` -> `HTTPServer`.
+   - Suggested: (A) simple CamelCase for determinism; allow an explicit override later if needed.
+   - Why: keeps generation predictable and avoids surprise casing rules in v0.
+4) Tap repo local path + auto-clone behavior for release when tap path is missing.
+   - Please choose: (A) require explicit `--tap-path`/config and fail if missing, (B) auto-clone to a temp dir if tap remote is known.
+   - Suggested: (B) auto-clone to temp for `bd release` when a tap remote is configured; still warn and show path used.
+   - Why: smoother CI/local workflows without forcing permanent clones, but still safe and explicit.

tokens used
38 598
Updated the planning doc to be more concrete and critical, and added the missing decision prompts.

Changes made
- Strengthened the critique section in `PRD.md` and aligned it with actual gaps (asset override, tap path/clone behavior, acronym policy, error mapping).
- Expanded the non‑interactive contract, CLI flags, and config example to include explicit asset selection (`--asset-name`) and tap path expectations in `PRD.md`.
- Added a new planning task for tap repo local path resolution + auto‑clone strategy, and tightened DoD for key tasks in `PRD.md`.
- Added new decision questions for acronym policy and tap repo path/auto‑clone behavior in `USER_TODO.md`.

Files updated
- `PRD.md`
- `USER_TODO.md`

Open decisions for you (added to `USER_TODO.md`)
- Acronym casing policy for formula class name.
- Tap repo local path default + auto‑clone behavior for `bd release`.

If you want me to keep going, the next best steps are:
1) Answer the new questions in `USER_TODO.md`.
2) Tell me if you want the tap repo path + auto‑clone decision to influence defaults (e.g., sibling folder vs temp clone).
3) I can then refine the remaining tasks into testable acceptance criteria.
```

## 2026-01-22T10:55:51Z

- run_id: 481eb95b-2ae0-4c8d-a354-f6876d41e548
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:50:21Z
- finished_at: 2026-01-22T10:55:51Z
- exit_code: 0

```
@@ -485,6 +510,18 @@
 - Status: open
 - DoD: matching algorithm with `asset_name`/`asset_template` precedence, OS/arch table, and failure message examples.
 
+10b) Define release discovery policy (tag-only vs GitHub Release, drafts/prereleases, missing release behavior).
+- Type: architecture
+- Priority: high
+- Status: open
+- DoD: explicit rules + examples + exact error messages for missing releases/assets.
+
+10c) Scope version update strategies and config fields (none/cargo/regex) and workspace handling rules.
+- Type: architecture
+- Priority: medium
+- Status: open
+- DoD: mode definitions + config schema + examples for Cargo workspaces and regex updates.
+
 11) Design release orchestration steps, dry-run behavior, idempotency rules, and summary output.
 - Type: architecture
 - Priority: high
@@ -515,7 +552,7 @@
 - Type: product
 - Priority: high
 - Status: open
-- DoD: prompt sequence + field defaults + preview examples + overwrite decision points + `--yes/--force` semantics.
+- DoD: prompt sequence + field defaults + preview examples + overwrite decision points + `--yes/--force` semantics + `brew tap-new` availability/fallback behavior.
 
 16) Define `bd doctor` checks, `--audit` integration, and failure modes.
 - Type: product
diff --git a/USER_TODO.md b/USER_TODO.md
index c21deabb750c58ff93b348138ea36794886859ca..2694ea21543be92a7b1be14db94ae233bc55519c
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -36,3 +36,11 @@
    - Please choose: (A) require explicit `--tap-path`/config and fail if missing, (B) auto-clone to a temp dir if tap remote is known.
    - Suggested: (B) auto-clone to temp for `bd release` when a tap remote is configured; still warn and show path used.
    - Why: smoother CI/local workflows without forcing permanent clones, but still safe and explicit.
+5) Release discovery policy for assets: require a GitHub Release, or allow tag-only?
+   - Please choose: (A) require a GitHub Release for asset lookup, fail if missing, or (B) allow tag-only and fall back to source tarball when no release exists.
+   - Suggested: (A) require a Release for asset strategy, and only fall back to source tarball when explicitly configured.
+   - Why: release assets are tied to releases; failing fast avoids silently using the wrong artifact.
+6) Version update strategy scope for v0.
+   - Please choose: (A) support `cargo` + `regex` update modes, or (B) `cargo` only in v0.
+   - Suggested: (A) both modes; regex is small surface area and supports non-Rust repos while keeping defaults unchanged.
+   - Why: keeps v0 flexible without forcing custom scripting.

tokens used
63 677
Using skill: workflow-pattern (follow team workflow expectations).

Updated the PRD to surface missing gaps and add concrete planning tasks for release discovery and version update strategies, plus added two new decision prompts to USER_TODO so we can lock those policies before implementation.

- `PRD.md`: added draft sections for release discovery + version update strategies, inserted tasks 10b/10c with DoD, added a release-discovery step in the release workflow, and expanded the `bd init` DoD to cover `brew tap-new` fallback behavior.
- `USER_TODO.md`: added decisions 5–6 (release discovery policy and version update strategy scope).

No tests run (planning/docs only).

If you’re ready, pick the options for USER_TODO items 1–6.  
Possible next steps:
1) You answer USER_TODO items 1–6.  
2) I convert tasks 10b/10c into detailed acceptance criteria + sample error messages once those decisions are locked.
```

## 2026-01-22T11:00:48Z

- run_id: f984512d-75b2-4c78-bcfa-66e26beed900
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T10:56:21Z
- finished_at: 2026-01-22T11:00:48Z
- exit_code: 0

```
- Status: open
 - DoD: mode definitions + config schema + examples for Cargo workspaces and regex updates.
 
+10d) Decide cross-platform formula asset support (single asset vs per-OS/arch sections) and config shape.
+- Type: architecture
+- Priority: high
+- Status: open
+- DoD: explicit v0 decision + config schema impact + example formula output for each supported mode.
+
 11) Design release orchestration steps, dry-run behavior, idempotency rules, and summary output.
 - Type: architecture
 - Priority: high
@@ -572,6 +590,12 @@
 - Status: open
 - DoD: write-atomicity spec + rollback steps + temp file strategy.
 
+18b) Specify preview/diff output for multi-repo changes (config + formula + tap repo).
+- Type: product
+- Priority: medium
+- Status: open
+- DoD: diff format examples + ordering rules + dry-run output contract.
+
 F) Testing and docs
 19) Plan testing matrix (init, release, doctor) with temp repos and mocked network/sha.
 - Type: engineering
diff --git a/USER_TODO.md b/USER_TODO.md
index 2694ea21543be92a7b1be14db94ae233bc55519c..62a5505c7188c0c7d1fba6ac91e34e04af247b7b
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -44,3 +44,7 @@
    - Please choose: (A) support `cargo` + `regex` update modes, or (B) `cargo` only in v0.
    - Suggested: (A) both modes; regex is small surface area and supports non-Rust repos while keeping defaults unchanged.
    - Why: keeps v0 flexible without forcing custom scripting.
+7) Cross-platform formula assets: should v0 support per-OS/arch URLs with `on_macos`/`on_linux` blocks, or require a single universal asset URL?
+   - Please choose: (A) support per-OS/arch sections in v0, or (B) require a single universal asset in v0.
+   - Suggested: (A) support per-OS/arch sections, but keep it minimal: two targets (darwin/linux) and optional arch split only when both assets are provided.
+   - Why: Homebrew runs on macOS and Linux; many CLIs publish per-OS assets, and forcing a universal asset can be a blocker.

tokens used
34 115
Using workflow-pattern to follow team workflow expectations.

Updated the planning docs to tighten critique, add missing architectural tasks, and surface a new decision request.
- `PRD.md`: expanded the “Needs strengthening” critique, added a draft section on cross‑platform assets, added task 10d for multi‑target formula support, and task 18b for preview/diff output; updated improvement and gap lists to reflect those additions.
- `USER_TODO.md`: added decision #7 on cross‑platform asset handling with a recommendation.

Decisions needed (reply with numbers + choice):
1) Multi-binary support (A/B) — `USER_TODO.md`
2) Formula `version` handling (A/B) — `USER_TODO.md`
3) Acronym casing policy (A/B) — `USER_TODO.md`
4) Tap path auto-clone behavior (A/B) — `USER_TODO.md`
5) Release discovery policy (A/B) — `USER_TODO.md`
6) Version update scope (A/B) — `USER_TODO.md`
7) Cross-platform assets (A/B) — `USER_TODO.md`

No tests run (docs-only changes).

Next steps:
1) Answer the items above; I will fold the decisions into `PRD.md` and tighten acceptance criteria.
2) I can draft the full preview/diff output spec and sample outputs if you want.
```

## 2026-01-22T11:04:19Z

- run_id: 109db860-4926-4686-a957-8d9a7dbf6346
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T11:01:18Z
- finished_at: 2026-01-22T11:04:19Z
- exit_code: 0

```
+++ b/USER_TODO.md
@@ -19,32 +19,16 @@
 - Tap layout: enforce a separate `homebrew-<repo>` tap repo in v0.
 - Release artifact strategy: default to GitHub release assets for binaries, with source tarball as fallback.
 
-## Decisions needed
-1) Multi-binary formula support: allow multiple `bin.install` entries in v0 or defer?
-   - Please choose: (A) allow multiple in v0 or (B) defer to v1.
-   - Suggested: (A) allow multiple in v0 via `bin = ["foo","bar"]`; prompt collects one name, then asks "Add another?" in a loop. Default remains single-bin for simplicity.
-   - Why: Rust projects commonly have multiple bins; supporting a list is a small scope increase and keeps formulas correct.
-2) Formula `version` handling: always include an explicit `version` stanza, or derive from URL when possible?
-   - Please choose: (A) always include explicit `version` or (B) derive from URL by default.
-   - Suggested: (A) explicit `version` by default for determinism; allow `version_mode = "derive"` in config to omit when URL contains the version.
-   - Why: avoids surprises when URL templates change and makes diffs clearer.
-3) Formula class acronym casing policy: how should `bd` treat acronyms when generating the Ruby class name?
-   - Please choose: (A) simple CamelCase of segments (`http-server` -> `HttpServer`), (B) preserve a short acronym list (`HTTP`, `CLI`, `URL`) so `http-server` -> `HTTPServer`.
-   - Suggested: (A) simple CamelCase for determinism; allow an explicit override later if needed.
-   - Why: keeps generation predictable and avoids surprise casing rules in v0.
-4) Tap repo local path + auto-clone behavior for release when tap path is missing.
-   - Please choose: (A) require explicit `--tap-path`/config and fail if missing, (B) auto-clone to a temp dir if tap remote is known.
-   - Suggested: (B) auto-clone to temp for `bd release` when a tap remote is configured; still warn and show path used.
-   - Why: smoother CI/local workflows without forcing permanent clones, but still safe and explicit.
-5) Release discovery policy for assets: require a GitHub Release, or allow tag-only?
-   - Please choose: (A) require a GitHub Release for asset lookup, fail if missing, or (B) allow tag-only and fall back to source tarball when no release exists.
-   - Suggested: (A) require a Release for asset strategy, and only fall back to source tarball when explicitly configured.
-   - Why: release assets are tied to releases; failing fast avoids silently using the wrong artifact.
-6) Version update strategy scope for v0.
-   - Please choose: (A) support `cargo` + `regex` update modes, or (B) `cargo` only in v0.
-   - Suggested: (A) both modes; regex is small surface area and supports non-Rust repos while keeping defaults unchanged.
-   - Why: keeps v0 flexible without forcing custom scripting.
+
 7) Cross-platform formula assets: should v0 support per-OS/arch URLs with `on_macos`/`on_linux` blocks, or require a single universal asset URL?
    - Please choose: (A) support per-OS/arch sections in v0, or (B) require a single universal asset in v0.
    - Suggested: (A) support per-OS/arch sections, but keep it minimal: two targets (darwin/linux) and optional arch split only when both assets are provided.
    - Why: Homebrew runs on macOS and Linux; many CLIs publish per-OS assets, and forcing a universal asset can be a blocker.
+8) Checksum acquisition policy: should v0 always download assets and compute SHA256 locally, or allow using a provided `.sha256`/`.sha256sum` asset?
+   - Please choose: (A) always compute locally (ignore checksum assets), or (B) allow checksum assets when explicitly configured.
+   - Suggested: (A) compute locally by default; it is deterministic and avoids ambiguity. Add an explicit opt-in for checksum assets later if needed.
+   - Why: checksum assets vary in format and naming; local hashing avoids parsing errors and mismatched files.
+9) Version/tag precedence and validation: how should `--version` and `--tag` interact, and how strict should semver be?
+   - Please choose: (A) allow `v`-prefixed tags as equivalent (`v1.2.3` => version `1.2.3`) and require matches, or (B) require exact string match with no prefix normalization.
+   - Suggested: (A) normalize a leading `v` for tags, and require `--version` to match after normalization; reject other mismatches.
+   - Why: many projects tag with `v` prefixes, and normalization reduces surprises.

tokens used
62 673
Using workflow-pattern (team workflow expectations).

AGENTS.md not found in the repo (searched for it), so I proceeded with the provided instructions.

I tightened the PRD to call out missing decisions around checksum acquisition, git remote ambiguity, and version/tag precedence; added draft sections for those open questions; and added new architectural planning tasks (1c, 10e, 10f) plus a release-workflow bullet to enforce `--version`/`--tag` mapping in `PRD.md`. I also added two explicit decision requests to `USER_TODO.md` so you can unblock those design choices.

- Updated critique + improvements, added draft sections, and new planning tasks in `PRD.md`.
- Added decisions 8 and 9 in `USER_TODO.md` (checksum policy, version/tag precedence).

If you want, reply with choices for the new items in `USER_TODO.md` so I can lock those into the PRD.

Next steps:
1) Answer items 1–9 in `USER_TODO.md` (new ones are 8 and 9).
2) Tell me which planning task(s) you want expanded into a short design note first.
```

## 2026-01-22T11:12:54Z

- run_id: a64db3cf-e031-4055-acb5-b61ec350af7d
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T11:04:49Z
- finished_at: 2026-01-22T11:12:54Z
- exit_code: 0

```
- Priority: high
-- Status: open
+- Status: in_progress
 - DoD: explicit rules + examples + exact error messages for missing releases/assets.
 
 10c) Scope version update strategies and config fields (none/cargo/regex) and workspace handling rules.
 - Type: architecture
 - Priority: medium
-- Status: open
+- Status: in_progress
 - DoD: mode definitions + config schema + examples for Cargo workspaces and regex updates.
 
 10d) Decide cross-platform formula asset support (single asset vs per-OS/arch sections) and config shape.
@@ -654,7 +657,7 @@
 - Priority: high
 - Status: closed
 
-22) Document release artifact default (release assets) and fallback logic (source tarball).
+22) Document release artifact default (release assets) and source tarball strategy (no implicit fallback).
 - Type: product
 - Priority: high
 - Status: closed
diff --git a/USER_TODO.md b/USER_TODO.md
index d3d153faa2fe2d4cdfe3e45e19beb5c41c59adea..5963b8cf82daaefafe38a5b8535dbe5f079fa4dc
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -8,7 +8,7 @@
 - Private repos: not in v0 (v1 backlog).
 - Default formula naming: repo name by default; allow override.
 - CI workflow generation in v0: no.
-- Version update: default does not update main repo version file; allow configurable updater per language/regex.
+- Version update: default does not update main repo version file; support `cargo` and `regex` update modes.
 - Config location: `.distill/config.toml` in the main repo; `.distill/` directory for related files.
 - Commit message conventions: configurable templates with a simple default (e.g. `feature: Updated formula for version 'X.Y.Z'`).
 - Tag format: default semver `X.Y.Z`; tags are optional.
@@ -17,4 +17,9 @@
 - `bd init`: detect existing setup and avoid overwriting without explicit confirmation.
 - Helper script: no; keep release logic inside `bd`.
 - Tap layout: enforce a separate `homebrew-<repo>` tap repo in v0.
-- Release artifact strategy: default to GitHub release assets for binaries, with source tarball as fallback.
+- Release artifact strategy: default to GitHub release assets; source tarball is an explicit strategy (no implicit fallback).
+- Multi-binary formula support: allow multiple `bin.install` entries in v0.
+- Formula `version` handling: always include an explicit `version` stanza.
+- Formula class acronym casing: simple CamelCase of segments (no acronym preservation).
+- Tap repo auto-clone: `bd release` auto-clones tap repo to a temp dir when remote is known.
+- Release discovery: GitHub Release required for release-asset strategy; tag-only is not supported.

tokens used
82 916
Updated the planning docs to fold in your latest decisions, tighten a few gaps, and sync the decision backlog.

- `PRD.md` now reflects the resolved decisions (multi-bin support, explicit `version`, simple CamelCase class naming, tap auto-clone, GitHub Release requirement, cargo+regex updates) and updates the relevant sections: formula rules, release discovery/workflow, idempotency, and non-interactive requirements.
- `PRD.md` adds a dedicated non‑Rust metadata detection task (5b), updates task statuses where decisions were made, and adjusts the critique section to focus on what still needs detail.
- `USER_TODO.md` now records the newly resolved decisions and clarifies the release-asset vs source‑tarball strategy (no implicit fallback).

Tests not run (planning-only edits).

Next steps (pick any):
1) Decide open items in `USER_TODO.md` (cross‑platform assets, checksum policy, version/tag precedence).
2) Specify non‑Rust metadata presets + Cargo workspace handling (tasks 5b and 10c) so we can lock the config schema and prompt flow.
```

## 2026-01-22T11:19:57Z

- run_id: 4f6f29b3-b780-433b-b7c5-633d68078d4a
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T11:13:24Z
- finished_at: 2026-01-22T11:19:57Z
- exit_code: 0

```
- DoD: mode definitions + config schema + examples for Cargo workspaces and regex updates.
 
-10d) Decide cross-platform formula asset support (single asset vs per-OS/arch sections) and config shape.
+10d) Specify cross-platform formula asset support (decision: per-OS/arch sections) and config shape.
 - Type: architecture
 - Priority: high
-- Status: open
-- DoD: explicit v0 decision + config schema impact + example formula output for each supported mode.
+- Status: in_progress
+- DoD: config schema impact + validation rules + example formula output for each supported mode.
 
-10e) Define checksum acquisition strategy (download vs checksum asset), size limits, and retry/backoff behavior.
+10e) Define checksum acquisition strategy (decision: download + compute locally), size limits, and retry/backoff behavior.
 - Type: architecture
 - Priority: high
-- Status: open
+- Status: in_progress
 - DoD: algorithm, limits, and exact error messages for download/verify failures.
 
-10f) Define version/tag precedence and validation rules (`--version`/`--tag`, `v` prefix, semver strictness).
+10f) Define version/tag precedence and validation rules (`--version`/`--tag`, decision: normalize `v` prefix, semver strictness).
 - Type: architecture
 - Priority: medium
-- Status: open
+- Status: in_progress
 - DoD: precedence rules + examples + exit code mapping for mismatches.
 
 11) Design release orchestration steps, dry-run behavior, idempotency rules, and summary output.
@@ -662,5 +680,20 @@
 - Priority: high
 - Status: closed
 
+23) Document cross-platform asset decision (per-OS/arch sections supported; universal asset allowed).
+- Type: product
+- Priority: high
+- Status: closed
+
+24) Document checksum policy decision (download + compute locally; ignore checksum assets in v0).
+- Type: product
+- Priority: medium
+- Status: closed
+
+25) Document version/tag normalization decision (leading `v` normalized; formula version uses normalized string).
+- Type: product
+- Priority: medium
+- Status: closed
+
 ## Open questions (tracked in USER_TODO.md)
 See USER_TODO.md for decisions needed before implementation.

tokens used
59 088
Using workflow-pattern to follow team workflow expectations.

Updated `PRD.md` to bake in the latest decisions (per-OS/arch assets, checksum policy, v-prefix normalization), tighten the critique/gaps list, and refine architectural tasks so they’re ready to split into concrete specs. I also added a tentative per-target config sketch and moved the relevant tasks to in_progress to reflect active design work. The release/asset sections now explicitly call out what’s decided vs. still needing spec detail, and the non-interactive contract now accounts for per-target asset selection. All changes are contained in `PRD.md`.

Next steps (pick any):
1) Decide semver strictness + prerelease/build handling, and whether drafts/prereleases are allowed in release discovery.
2) Confirm the exact Homebrew DSL for per-OS/arch blocks (then I can lock the formula output examples).
3) Specify checksum download limits/retry/backoff and dry-run behavior so we can finalize 10e.
```

## 2026-01-22T11:23:40Z

- run_id: c2d861ad-89cd-4df8-b2e3-50b46e98d9d5
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T11:20:27Z
- finished_at: 2026-01-22T11:23:40Z
- exit_code: 0

```
11) Design release orchestration steps, dry-run behavior, idempotency rules, and summary output.
 - Type: architecture
 - Priority: high
 - Status: open
-- DoD: step-by-step flow + idempotency matrix + summary output sample.
+- DoD: step-by-step flow + idempotency matrix + dry-run output sample + exact error messages/exit codes for common failure paths.
 
 12) Decide git integration approach (libgit2 vs shell); define remote detection, push defaults, and error modes.
 - Type: architecture
@@ -630,7 +635,7 @@
 - Type: product
 - Priority: high
 - Status: open
-- DoD: prompt sequence + field defaults + preview examples + overwrite decision points + `--yes/--force` semantics + `brew tap-new` availability/fallback behavior.
+- DoD: prompt sequence + field defaults + preview examples + overwrite decision points + `--yes/--force` semantics + `brew tap-new` availability/fallback behavior (missing brew/offline).
 
 16) Define `bd doctor` checks, `--audit` integration, and failure modes.
 - Type: product
diff --git a/USER_TODO.md b/USER_TODO.md
index 5963b8cf82daaefafe38a5b8535dbe5f079fa4dc..effeb1c6e9e90dc22b1d2141ca3b12ebdc706f13
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -23,3 +23,15 @@
 - Formula class acronym casing: simple CamelCase of segments (no acronym preservation).
 - Tap repo auto-clone: `bd release` auto-clones tap repo to a temp dir when remote is known.
 - Release discovery: GitHub Release required for release-asset strategy; tag-only is not supported.
+
+## Decisions needed
+1) Release discovery: how should drafts/prereleases be handled when selecting assets?
+   - Please choose: (A) ignore drafts/prereleases by default; allow `--include-prerelease` to opt-in, (B) allow prereleases by default when no stable exists, (C) always allow prereleases (no opt-in).
+2) Tag/version source when `--tag` and `--version` are omitted.
+   - Please choose: (A) require explicit input in v0 (no auto-latest), or (B) default to latest GitHub Release tag when available.
+3) Semver strictness for `--version` (after `v` normalization for tags).
+   - Please choose: (A) strict `X.Y.Z` only, (B) allow prerelease/build metadata (`X.Y.Z-rc.1`, `+build.1`), (C) accept any string and only validate non-empty.
+4) `bd release` idempotency when formula already targets the requested version.
+   - Please choose: (A) treat as no-op success with a warning, (B) fail with a dedicated exit code, (C) require `--force` to re-apply and continue.
+5) `bd init --import-formula` merge policy for conflicting fields.
+   - Please choose: (A) formula is source-of-truth for formula fields; config only fills gaps, (B) config is source-of-truth unless `--prefer-formula` is set, (C) always prompt per field in interactive mode and fail in non-interactive unless `--prefer-formula`.

tokens used
48 118
Using skill: workflow-pattern (team workflow guidance). No `AGENTS.md` found under `/Users/marius/jobb/brewdistill/brewdistillery`.

Updated the PRD to tighten gaps and make the architecture tasks more testable, and captured new decision requests.
- Strengthened critique + remaining execution gaps to call out release idempotency, semver strictness, multi-target formula output, and per-target asset resolution. `PRD.md`
- Tightened task DoD language to require concrete Ruby output examples, default limits, and explicit error/exit code coverage. `PRD.md`
- Clarified release workflow to resolve per-target assets and compute per-target SHA. `PRD.md`
- Added explicit decision questions for release discovery, version source, semver strictness, idempotency, and import-formula merge policy. `USER_TODO.md`

Decisions needed from you (added to `USER_TODO.md`):
- Release discovery: drafts/prereleases handling
- Version/tag source when not provided
- Semver strictness
- Release idempotency when formula already matches
- Import-formula merge policy

Next steps (pick any):
1) Answer the decisions in `USER_TODO.md`
2) Ask me to expand any specific task into a 1–2 page design note draft
```

## 2026-01-22T12:20:31Z

- run_id: 145ba430-4787-4a68-a089-bb1c6620d4c0
- loop_name: dashing-mackey
- status: success
- profile: codex
- harness: codex
- prompt_source: base
- prompt_path: /Users/marius/jobb/brewdistill/brewdistillery/PROMPT.md
- prompt_override: false
- started_at: 2026-01-22T12:12:43Z
- finished_at: 2026-01-22T12:20:31Z
- exit_code: 0

```
-- DoD: field-level merge table + explicit overwrite prompts.
+- Status: in_progress
+- DoD: field-level merge table (formula wins for formula fields) + explicit overwrite prompts + non-interactive overwrite policy.
 
 5) Define repository metadata detection (Rust-first): `Cargo.toml` parsing, bin targets, license detection, git fallback.
 - Type: architecture
@@ -579,7 +581,7 @@
 - Type: architecture
 - Priority: high
 - Status: in_progress
-- DoD: explicit rules + examples + exact error messages for missing releases/assets.
+- DoD: explicit rules (draft ignore, prerelease opt-in, latest tag default) + examples + exact error messages for missing releases/assets.
 
 10c) Scope version update strategies and config fields (none/cargo/regex) and workspace handling rules.
 - Type: architecture
@@ -603,7 +605,7 @@
 - Type: architecture
 - Priority: medium
 - Status: in_progress
-- DoD: precedence rules + examples + semver strictness decision + exit code mapping for mismatches.
+- DoD: precedence rules + examples + semver validation (prerelease/build allowed) + exit code mapping for mismatches.
 
 11) Design release orchestration steps, dry-run behavior, idempotency rules, and summary output.
 - Type: architecture
@@ -701,4 +703,4 @@
 - Status: closed
 
 ## Open questions (tracked in USER_TODO.md)
-See USER_TODO.md for decisions needed before implementation.
+None currently; see USER_TODO.md if new decisions are added later.
diff --git a/USER_TODO.md b/USER_TODO.md
index 5963b8cf82daaefafe38a5b8535dbe5f079fa4dc..66e42b1088b6b42dd43daaa7265b02a948d69364
--- a/USER_TODO.md
+++ b/USER_TODO.md
@@ -22,4 +22,10 @@
 - Formula `version` handling: always include an explicit `version` stanza.
 - Formula class acronym casing: simple CamelCase of segments (no acronym preservation).
 - Tap repo auto-clone: `bd release` auto-clones tap repo to a temp dir when remote is known.
-- Release discovery: GitHub Release required for release-asset strategy; tag-only is not supported.
+- Release discovery: GitHub Release required for release-asset strategy; tag-only is not supported; drafts ignored; prereleases ignored unless `--include-prerelease`; default to latest Release tag when `--tag`/`--version` are omitted.
+- Semver strictness: accept prerelease/build metadata (e.g. `1.2.3-rc.1`, `1.2.3+build.1`).
+- Release idempotency: when formula already targets the requested version, require `--force` to re-apply and continue.
+- `bd init --import-formula`: formula is source-of-truth for formula fields; config fills gaps.
+
+## Decisions needed
+None right now.

tokens used
125 564
Integrated your latest USER_FEEDBACK decisions into the PRD, tightened release discovery/idempotency/non‑interactive rules, and updated planning task statuses; USER_TODO now reflects that there are no pending decisions.

- `PRD.md`: folded in prerelease handling + latest tag default + semver acceptance + `--force` idempotency + import‑formula source‑of‑truth, updated release discovery section, non‑interactive contract, idempotency matrix, release workflow, CLI flags, and task statuses/DoD to match workflow expectations.
- `USER_TODO.md`: added the resolved decisions and marked “Decisions needed” as none.
- Note: no `AGENTS.md` found in `/Users/marius/jobb/brewdistill/brewdistillery`, so I proceeded without it.
- Tests not run (docs‑only updates).

Next steps (pick any):
1) Decide prerelease selection order when `--include-prerelease` is set and stable releases also exist, plus exact error wording/exit codes.  
2) Define the field‑level `--import-formula` merge table and non‑interactive overwrite policy.  
3) Finalize idempotency behavior for existing tags and map exit codes/messages.
```

