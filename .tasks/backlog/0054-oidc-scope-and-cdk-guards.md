# 0054 — OIDC deploy roles trust every branch; scope to release/tags

| Field | Value |
|---|---|
| Priority | P2 |
| Type | security |
| Module | `infra/cdk/lib/great-sh-stack.ts` |
| Status | backlog |
| Estimated Complexity | S (code) + infra deploy |

## Problem

The GitHub OIDC trust conditions for the site-deploy and CDK roles use
`repo:superstruct/great.sh:*`, so a workflow on ANY branch or PR of the repo
can assume the production deploy roles. Deploys are only intended from the
`release` branch (site, CDK) and `v*` tags (release).

## Proposed Fix

Restrict the `token.actions.githubusercontent.com:sub` condition to
`repo:superstruct/great.sh:ref:refs/heads/release` and
`repo:superstruct/great.sh:ref:refs/tags/v*` (StringLike). Verify with
`cdk diff` before deploying; requires a CDK deploy to take effect.

Also note: `CLOUDFRONT_DISTRIBUTION_ID` is a hand-set repo variable — consider
wiring it from the stack output (SSM parameter or workflow lookup) so it can't
go stale. Workflows now fail loudly when it is unset.

## Acceptance Criteria

- A workflow run on a non-release branch cannot assume the deploy roles
- Release-branch and tag deploys still work end-to-end
