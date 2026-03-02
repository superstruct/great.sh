# 0010: Complete All Stubs, TODOs, and Gaps in the Rust CLI

**Priority:** P0 (umbrella -- contains P0, P1, and P2 sub-groups)
**Type:** feature / bugfix / refactor
**Module:** Multiple -- see groups below
**Status:** complete — all 11 groups verified done (see Group Completion Status below)
**Estimated Complexity:** XL (11 groups, ~40 individual work items) — all completed

## Context

A full audit of the `great` CLI codebase reveals 11 groups of incomplete work:
TODO comments, stub implementations, dead code, missing tests, and no
cross-platform test infrastructure. This task tracks all of them in one place so
nothing falls through the cracks. Each group is independently implementable and
has its own priority, acceptance criteria, and complexity estimate.

The codebase has progressed well beyond stubs -- `apply`, `status`, `doctor`,
`diff`, `init`, `mcp`, `vault`, `sync`, and `template` all have substantial
implementations. What remains is finishing the last-mile gaps in each module.

## Group Completion Status

| Group | Topic | Status | Evidence (codebase verification 2026-02-27) |
|---|---|---|---|
| A | Tool Install Mapping | DONE | `apply.rs:272-316` — `tool_install_spec()` covers cdk, az, gcloud, aws, pnpm, uv, starship, bw |
| B | Starship Configuration | DONE | `apply.rs:851-947` — `configure_starship()` generates config + shell init |
| C | MCP Add Command | DONE | `mcp.rs:109-164` — `run_add()` uses `toml_edit` to modify great.toml |
| D | Doctor --fix | DONE | `doctor.rs:52-229` — 8 fix action types, pre-caches sudo |
| E | Update Command | DONE | `update.rs:1-206` — queries GitHub API, self-replaces binary |
| F | Vault Completion | DONE | `vault.rs` — login, unlock, set, import all implemented |
| G | Sync Pull --apply | DONE | `sync.rs:14-131` — `--apply` flag, backup + write |
| H | Template Update from Registry | DONE | `template.rs:183-277` — fetches from GitHub, downloads to local |
| I | Dead Code and Safety Cleanup | DONE | iteration-016 commit `9a04955`; `cargo clippy` = 0 warnings |
| J | Integration Test Coverage | DONE | `tests/cli_smoke.rs` — 90 tests |
| K | Docker Test Rigs | DONE | `docker-compose.yml` + 9 files in `docker/` |

All 11 groups verified complete via codebase inspection (2026-02-27). GROUP I
explicitly tracked in iteration 016. Other groups landed across iterations
003–010. This umbrella task is fully resolved.

---

## GROUP A: Tool Install Mapping Table (P0, Size: L)

**Problem:** `src/cli/apply.rs:198-210` documents 8 CLI tools that need
special install paths differing from a simple `brew install <name>`. Currently
the apply command passes the tool name directly to package managers, which fails
for tools like `cdk` (npm global), `az` (brew name differs), and `gcloud`
(snap/curl on Linux).

**Files:**
- `src/cli/apply.rs` -- lines 196-260 (CLI tool install loop)
- `src/cli/apply.rs` -- line 350 (Bitwarden CLI npm-first logic)
- `src/config/schema.rs` -- lines 58-60 (TODOs for special install paths)

**Requirements:**
1. Create a tool-name-to-install-command mapping (struct or HashMap) covering:
   `cdk` (npm -g aws-cdk), `az` (brew azure-cli / curl on Linux), `gcloud`
   (brew google-cloud-sdk / snap on Linux), `aws` (brew awscli / curl on
   Linux), `pnpm` (npm -g or brew), `uv` (brew or pip or curl), `starship`
   (brew), `bitwarden-cli` (npm -g @bitwarden/cli).
2. The mapping must be platform-aware: prefer Homebrew on macOS/Ubuntu/WSL,
   fall back to npm/pip/curl/snap on other Linux distros.
3. Integrate the mapping into the CLI tool install loop in `apply.rs` so
   `great apply` installs these tools correctly.
4. Fix Bitwarden CLI install at line 350 to try `npm install -g @bitwarden/cli`
   first, then fall back to brew/apt.

**Acceptance Criteria:**
- [ ] `great apply --dry-run` with `cdk = "latest"` in config reports "would install via npm".
- [ ] `great apply --dry-run` with `az = "latest"` reports the correct platform-specific install method.
- [ ] Bitwarden CLI install attempts npm before brew/apt.
- [ ] `cargo clippy` produces zero warnings for modified files.
- [ ] Unit tests cover the mapping table for at least macOS and Ubuntu platforms.

**Dependencies:** Task 0007 (package manager) -- landed.

---

## GROUP B: Starship Configuration (P1, Size: M)

**Problem:** `src/cli/init.rs:119-121` and `src/cli/apply.rs:369-375` have TODO
comments noting that after installing Starship, the CLI should generate a preset
config and add shell init lines. Currently Starship is installed but never
configured.

**Files:**
- `src/cli/init.rs` -- lines 117-122
- `src/cli/apply.rs` -- lines 368-375

**Requirements:**
1. After installing Starship, generate `~/.config/starship.toml` with a
   great.sh preset (or leave existing if present).
2. Detect the user's shell (bash/zsh/fish) and add the appropriate init line
   to their shell profile (`~/.bashrc`, `~/.zshrc`, `~/.config/fish/config.fish`).
3. Guard against duplicate init lines if already present.

**Acceptance Criteria:**
- [ ] `great apply` with `starship = "latest"` creates `~/.config/starship.toml` if absent.
- [ ] Shell init line is appended only if not already present in the profile.
- [ ] Supports bash, zsh, and fish shells.
- [ ] `--dry-run` reports what would be written without modifying files.

**Dependencies:** GROUP A (install mapping for starship).

---

## GROUP C: MCP Add Command (P1, Size: S)

**Problem:** `src/cli/mcp.rs:131-143` -- `great mcp add <name>` currently prints
a TOML snippet for the user to manually paste into `great.toml`. It should
actually modify the file.

**Files:**
- `src/cli/mcp.rs` -- `run_add()` function (lines 110-145)

**Requirements:**
1. Read the existing `great.toml`, insert a new `[mcp.<name>]` section with
   sensible defaults (command = "npx", args = ["-y", "@modelcontextprotocol/server-<name>"]).
2. Write the modified config back, preserving existing content and formatting
   as much as possible (use `toml_edit` crate if needed for format preservation).
3. Print the added entry and remind the user to run `great apply`.

**Acceptance Criteria:**
- [ ] `great mcp add postgres` appends `[mcp.postgres]` to `great.toml`.
- [ ] Running `great mcp add postgres` twice does not create duplicates (warns instead).
- [ ] The modified `great.toml` parses correctly after modification.
- [ ] Integration test verifies the file is modified correctly.

**Dependencies:** None (great.toml loading is already implemented).

---

## GROUP D: Doctor --fix (P1, Size: L)

**Problem:** `src/cli/doctor.rs:28-30` -- the `--fix` flag is accepted but
prints "not yet implemented". Users who see "Run `great doctor --fix`" at the
summary (line 67) hit a dead end.

**Files:**
- `src/cli/doctor.rs` -- lines 27-30, plus all `check_*` functions

**Requirements:**
1. When `--fix` is passed, the doctor command should attempt to fix each failed
   check automatically: install missing essential tools, install Homebrew if
   missing, set up missing directories.
2. Each fix attempt should report success or failure individually.
3. Fixes that require user input (e.g., API keys) should print instructions
   instead of attempting to fix.
4. After all fixes, re-run the diagnostic to show updated results.

**Acceptance Criteria:**
- [ ] `great doctor --fix` attempts to install a missing essential tool (e.g., `bat`) via available package manager.
- [ ] Fixes that cannot be automated (API keys, manual installs) print clear instructions.
- [ ] A summary shows what was fixed vs. what still needs manual attention.
- [ ] `--fix` without any failures reports "nothing to fix".

**Dependencies:** GROUP A (install mapping needed for correct tool installation).

---

## GROUP E: Update Command (P1, Size: M)

**Problem:** `src/cli/update.rs:23-27` and `30-36` -- both `--check` and the
default self-update path print "not yet available" stubs. The `tokio` and
`reqwest` dependencies are already in `Cargo.toml` for this purpose.

**Files:**
- `src/cli/update.rs` -- entire file (40 lines)

**Requirements:**
1. `great update --check` queries the GitHub Releases API
   (`https://api.github.com/repos/great-sh/great/releases/latest`) and
   compares the tag against `CURRENT_VERSION` using semver.
2. `great update` (without `--check`) downloads the appropriate release binary
   for the current platform/arch and replaces the running binary (self-update).
3. Handle gracefully: no internet, rate-limited, already up-to-date, permission
   errors on binary replacement.

**Acceptance Criteria:**
- [ ] `great update --check` prints "up to date" or "new version available: vX.Y.Z".
- [ ] `great update` downloads and replaces the binary (tested manually, mocked in CI).
- [ ] Network errors produce a clear message, not a panic.
- [ ] `cargo clippy` produces zero warnings.

**Dependencies:** Requires a GitHub repository with releases (can be mocked for tests).

---

## GROUP F: Vault Completion (P1, Size: L)

**Problem:** Three vault subcommands are stubs:
- `vault login` (line 45-49) -- prints "not yet available"
- `vault unlock` (line 51-55) -- prints "not yet available"
- `vault import` (line 103-141) -- lists keys but cannot actually import them

**Files:**
- `src/cli/vault.rs` -- `run_login()`, `run_unlock()`, `run_import()`
- `src/vault/mod.rs` -- provider implementations

**Requirements:**
1. `vault login` authenticates with the local system keychain (macOS Keychain,
   Linux Secret Service/libsecret) using the existing keychain provider.
2. `vault unlock` opens/creates a local encrypted vault file
   (`~/.config/great/vault.enc`) with a user-provided passphrase.
3. `vault import` reads secrets from an env file or a named provider and
   stores them via `vault set` for each key-value pair.

**Acceptance Criteria:**
- [ ] `vault login` stores and retrieves a test credential via system keychain.
- [ ] `vault import .env` reads key=value pairs from a dotenv file and stores each.
- [ ] `vault import` from a provider that lists zero secrets prints "nothing to import".
- [ ] Error messages clearly state what went wrong (keychain locked, file not found, etc.).

**Dependencies:** Existing vault provider infrastructure in `src/vault/mod.rs`.

---

## GROUP G: Sync Pull --apply (P1, Size: S)

**Problem:** `src/cli/sync.rs:63-72` -- `great sync pull` loads the sync blob
but prints "Would restore great.toml" instead of actually restoring it.

**Files:**
- `src/cli/sync.rs` -- `run_pull()` function

**Requirements:**
1. Add `--apply` flag to the `Pull` variant.
2. When `--apply` is passed, write the pulled config blob to `great.toml`
   (with a backup of the existing file to `great.toml.bak`).
3. Without `--apply`, show a preview of what would change (diff against current).

**Acceptance Criteria:**
- [ ] `great sync pull --apply` overwrites `great.toml` and creates `great.toml.bak`.
- [ ] `great sync pull` (without --apply) shows the pulled config without modifying files.
- [ ] If no sync data exists, prints a clear message directing to `great sync push`.

**Dependencies:** Existing sync infrastructure in `src/sync/mod.rs`.

---

## GROUP H: Template Update from Registry (P2, Size: M)

**Problem:** `src/cli/template.rs:130-134` -- `great template update` prints
"Template registry is not yet available."

**Files:**
- `src/cli/template.rs` -- `run_update()` function

**Requirements:**
1. Check the great-sh GitHub releases (or a templates repo) for updated
   template files.
2. Download new/updated templates and store them in the local data directory
   alongside the built-in templates.
3. `great template list` should show both built-in and downloaded templates.

**Acceptance Criteria:**
- [ ] `great template update` checks GitHub for new templates and reports what changed.
- [ ] Downloaded templates appear in `great template list`.
- [ ] Network errors produce a clear message, not a panic.
- [ ] Built-in templates are never overwritten (downloaded templates shadow them).

**Dependencies:** GROUP E (shares GitHub API client code).

---

## GROUP I: Dead Code and Safety Cleanup (P1, Size: S)

**Problem:** 11 dead code items identified across modules, and 5 `.unwrap()`
calls on `Option`/`Result` in non-test production code paths that should use
`?`, `.unwrap_or()`, or `.unwrap_or_default()`.

**Files:**
- `src/config/mod.rs` -- dead code items
- `src/error.rs` -- unused `GreatError` variants (Network, possibly others)
- `src/mcp/mod.rs` -- dead code items
- `src/platform/package_manager.rs` -- dead code items
- `src/platform/runtime.rs` -- dead code items
- `src/sync/mod.rs` -- dead code items
- `src/vault/mod.rs` -- dead code items
- `src/cli/status.rs` -- `.unwrap_or("")` on line 191 (safe but inconsistent pattern)
- `src/cli/doctor.rs` -- `.unwrap_or("")` on line 377 (same pattern)

**Requirements:**
1. Run `cargo clippy -- -W dead_code` and either use, remove, or explicitly
   `#[allow(dead_code)]` (with a comment explaining why) each item.
2. Replace any `.unwrap()` in non-test code with `?`, `.unwrap_or()`,
   `.unwrap_or_default()`, or `.ok()` as appropriate.
3. Confirm `tokio` and `reqwest` are used (they will be once GROUP E lands);
   if not yet, add `#[allow(unused)]` with a note.

**Acceptance Criteria:**
- [ ] `cargo clippy` produces zero warnings with default lint level.
- [ ] Zero `.unwrap()` calls exist in `src/cli/*.rs` outside of `#[cfg(test)]` blocks.
- [ ] All dead code items are resolved (used, removed, or annotated).

**Dependencies:** None (can be done in parallel with everything else).

---

## GROUP J: Integration Test Coverage (P0, Size: L)

**Problem:** Only 4 smoke tests exist in `tests/cli_smoke.rs` (help, version,
init help, no-args). No integration tests exercise the actual subcommands
against real or fixture configs.

**Files:**
- `tests/cli_smoke.rs` -- existing 4 tests
- New: `tests/` directory -- additional test files

**Requirements:**
1. Add integration tests for each subcommand that can run without side effects:
   - `great status` (no config) -- should succeed with warning
   - `great status --json` -- should output valid JSON
   - `great doctor` -- should succeed and print summary
   - `great diff` (no config) -- should print error and exit cleanly
   - `great template list` -- should list built-in templates
   - `great mcp list` (no config) -- should show "no servers" message
   - `great vault list` -- if it exists, or `vault` help
   - `great sync push` (no config) -- should error gracefully
   - `great apply --dry-run` (no config) -- should error with exit code 1
2. Add fixture-based tests using a temp directory with a sample `great.toml`:
   - `great apply --dry-run` -- should display plan without modifying system
   - `great diff --config <fixture>` -- should show diffs against system state
   - `great init --template ai-minimal` -- should create great.toml in temp dir

**Acceptance Criteria:**
- [ ] At least 12 integration tests exist and pass with `cargo test`.
- [ ] Tests for `status`, `doctor`, `diff`, `apply --dry-run`, `template list`, and `init --template` are present.
- [ ] Fixture-based tests use `tempfile` for isolation (no filesystem pollution).
- [ ] All tests pass in CI (GitHub Actions on ubuntu-latest).
- [ ] `cargo test` completes in under 60 seconds.

**Dependencies:** None (tests exercise existing code).

---

## GROUP K: Docker Test Rigs for Cross-Platform Testing (P1, Size: L)

**Problem:** No cross-platform test infrastructure exists. The legacy repo at
`/home/isaac/src/great-sh/docker-compose.yml` has a pattern using `dockurr`
images for Windows and macOS, plus `qemux/qemu` for Ubuntu, but those are
heavyweight VM-based containers. The new repo needs lighter containers for CI
and heavier ones for manual cross-platform validation.

**Files:**
- New: `docker-compose.yml` at repo root
- New: `docker/` directory for Dockerfiles and test scripts
- Reference: `/home/isaac/src/great-sh/docker-compose.yml`

**Requirements:**
1. Create a `docker-compose.yml` with lightweight Linux test containers:
   - `ubuntu-test`: Ubuntu 22.04 with curl, git, build-essential
   - `fedora-test`: Fedora 39 with dnf, git, gcc
   Both containers mount the repo source and run `cargo test` inside.
2. Create test scripts (`docker/test-ubuntu.sh`, `docker/test-fedora.sh`) that
   install Rust, build the CLI, and run the integration test suite.
3. Add heavyweight containers (commented out by default) for full-stack testing:
   - Windows 11 via `dockurr/windows` (from legacy pattern)
   - macOS 15 via `dockurr/macos` (from legacy pattern)
4. Document usage in a comment block at the top of `docker-compose.yml`.

**Acceptance Criteria:**
- [ ] `docker compose up ubuntu-test` builds and runs `cargo test` successfully.
- [ ] `docker compose up fedora-test` builds and runs `cargo test` successfully.
- [ ] Windows and macOS containers are present but commented out with instructions.
- [ ] Test scripts are executable and idempotent.
- [ ] Container definitions do not require KVM (so they work in CI and on developer machines without hardware virtualization).

**Dependencies:** GROUP J (integration tests must exist to run in containers).

---

## Implementation Order

The recommended implementation sequence, respecting dependencies:

```
Phase 1 (P0 -- unblocks everything):
  GROUP J: Integration Tests     -- no deps, enables CI confidence
  GROUP A: Tool Install Mapping  -- no deps, unblocks B and D

Phase 2 (P1 -- this iteration):
  GROUP I: Dead Code Cleanup     -- no deps, quick win
  GROUP C: MCP Add Command       -- no deps, small
  GROUP G: Sync Pull --apply     -- no deps, small
  GROUP B: Starship Config       -- depends on A
  GROUP D: Doctor --fix           -- depends on A
  GROUP E: Update Command        -- no deps (but shares code with H)
  GROUP F: Vault Completion      -- no deps
  GROUP K: Docker Test Rigs      -- depends on J

Phase 3 (P2 -- next iteration):
  GROUP H: Template Update       -- depends on E
```

## Notes

- **Split guidance:** If this umbrella task is too large for a single iteration,
  each GROUP can be extracted into its own task file (0010a, 0010b, ... or
  0011-0021). The acceptance criteria are already self-contained per group.
- **No new crate dependencies anticipated** except possibly `toml_edit` (for
  GROUP C format-preserving TOML writes) and `self_replace` or `self-update`
  (for GROUP E binary replacement).
- **The `tokio` and `reqwest` crates** are already in `Cargo.toml` and will
  finally be used by GROUPs E and H (GitHub API calls).
- **Legacy reference:** The Docker patterns in `/home/isaac/src/great-sh/docker-compose.yml`
  use VM-based container images (`dockurr/*`, `qemux/*`) which require KVM.
  GROUP K's lightweight containers should work without KVM for CI compatibility.
