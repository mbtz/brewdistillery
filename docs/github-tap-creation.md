# GitHub API tap repo creation flow

`bd init` can create the tap repository on GitHub via the public GitHub REST
API, using a token provided via environment variables.

This flow is opt-in via:
- `bd init --create-tap`

This document specifies the required scopes, flow ordering, and fallback
behavior.

## Token source and required scopes

Token sources (checked in this order):
- `GITHUB_TOKEN`
- `GH_TOKEN`

Required scopes:
- Public repos only: `public_repo`
- Organization repos: the token must be authorized for the target org

Tokens are read from the environment and are never persisted to config.

## Flow overview

The creation flow is opt-in and should be explicitly requested by the user.

High-level sequence:

```text
resolve tap owner/repo
  -> require token (env)
  -> create repo via GitHub API
  -> capture remote URL from response
  -> continue with normal tap resolution (clone or use --tap-path)
```

## Dry-run behavior

Dry-run mode avoids network calls:
- Prints: `dry-run: would create tap repo <owner>/<repo> on GitHub`
- Sets `tap.remote` to `https://github.com/<owner>/<repo>.git`
- Continues with normal preview behavior without cloning

## API steps and ordering

1. Validate inputs:
   - `tap.owner` must be non-empty
   - `tap.repo` must be non-empty
2. Require a token from env.
3. Determine API path:
   - If `tap.owner` matches the authenticated user: `POST /user/repos`
   - Otherwise: `POST /orgs/{owner}/repos`
4. Send create request with:
   - `name = <tap.repo>`
   - `private = false`
5. On success:
   - Use the returned clone URL to set `tap.remote`
   - Continue with tap path resolution and cloning behavior

## Failure and fallback behavior

Missing token:
- Fail with exit code 5:
  - `GitHub token missing; set GITHUB_TOKEN or GH_TOKEN to create the tap repo`

Invalid inputs:
- Exit code 2:
  - `missing required fields for --create-tap: tap-owner, tap-repo`
- Exit code 3:
  - `tap owner cannot be empty`
  - `tap repo cannot be empty`
- Exit code 3 (conflict):
  - `--create-tap cannot be used with --tap-new`
  - `--create-tap cannot be used with --tap-remote`

API errors:
- Exit code 5 with an actionable message derived from the API response.

Fallback guidance when creation is not possible:
- Create the tap manually, then re-run `bd init` with one of:
  - `--tap-remote <url>`
  - `--tap-path <path>`
