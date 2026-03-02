# 0016: `great loop install` — Overwrite Safety and `--force` Flag

**Priority:** P0
**Type:** enhancement
**Module:** `src/cli/loop_cmd.rs`
**Status:** pending
**Estimated Complexity:** M

## Context

`great loop install` unconditionally calls `std::fs::write` for every managed
file, silently destroying any user customizations to those files.  A user who
has edited an agent persona (`~/.claude/agents/davinci.md`) or a slash command
(`~/.claude/commands/loop.md`) will lose those edits the next time they upgrade
great.sh and re-run `great loop install`.

### Files written by `great loop install` (current behavior)

| Path | Description |
|---|---|
| `~/.claude/agents/nightingale.md` | Agent persona |
| `~/.claude/agents/lovelace.md` | Agent persona |
| `~/.claude/agents/socrates.md` | Agent persona |
| `~/.claude/agents/humboldt.md` | Agent persona |
| `~/.claude/agents/davinci.md` | Agent persona |
| `~/.claude/agents/vonbraun.md` | Agent persona |
| `~/.claude/agents/turing.md` | Agent persona |
| `~/.claude/agents/kerckhoffs.md` | Agent persona |
| `~/.claude/agents/rams.md` | Agent persona |
| `~/.claude/agents/nielsen.md` | Agent persona |
| `~/.claude/agents/knuth.md` | Agent persona |
| `~/.claude/agents/gutenberg.md` | Agent persona |
| `~/.claude/agents/hopper.md` | Agent persona |
| `~/.claude/agents/dijkstra.md` | Agent persona |
| `~/.claude/agents/wirth.md` | Agent persona |
| `~/.claude/commands/loop.md` | Slash command |
| `~/.claude/commands/bugfix.md` | Slash command |
| `~/.claude/commands/deploy.md` | Slash command |
| `~/.claude/commands/discover.md` | Slash command |
| `~/.claude/commands/backlog.md` | Slash command |
| `~/.claude/teams/loop/config.json` | Agent Teams config |

Notes on files that are already safe (no change needed):

- `~/.claude/settings.json` — already non-destructive: creates only when absent,
  and only mutates the `statusLine` key in an existing file.
- `.tasks/reports/.template.md` — written only when `--project` is passed; this
  file should also receive the same overwrite check (see "Partial install"
  below).
- `.gitignore` — already guarded by a content check before appending.

### Desired new behavior

**Default (interactive, no `--force`):**

1. Before writing, collect the subset of the 21 managed files that already
   exist on disk.
2. If that subset is empty, proceed silently (first install — no prompt needed).
3. If one or more files already exist, print a concise list of the would-be
   overwritten paths and prompt:

   ```
   The following files already exist and will be overwritten:
     ~/.claude/agents/davinci.md
     ~/.claude/commands/loop.md
   Overwrite? [y/N]
   ```

   Default answer is **N** (abort). The user must type `y` or `yes`
   (case-insensitive) to proceed.

4. If stdin is not a TTY (i.e., the command is run in a script or with piped
   input), treat it as a non-interactive context: abort without prompting and
   print an explanatory message directing the user to use `--force`.

**`--force` flag:**

- Skip the existence check and prompt entirely.
- Overwrite all files as the current code does.
- Print a single informational line: `(--force: overwriting existing files)`
  before the install proceeds.

### Partial install consideration

Only files that already exist need to be listed in the prompt. Files that are
new (do not yet exist) are written silently regardless. This means a partial
install (some files present, some absent) correctly prompts only for the
colliding paths.

## Acceptance Criteria

- [ ] When one or more managed files already exist and stdin is a TTY, `great loop install` prints the list of conflicting paths and prompts `Overwrite? [y/N]`; answering `N` or pressing Enter aborts with exit code 1 and no files are written.
- [ ] Answering `y` at the prompt proceeds with the full install (all 21 managed files written, including those that did not previously exist).
- [ ] `great loop install --force` skips the prompt and overwrites all files silently (reproduces current behavior); exit code is 0 on success.
- [ ] When stdin is not a TTY and at least one managed file already exists, the command aborts with a non-zero exit code and prints a message instructing the user to use `--force`; no files are written.
- [ ] A fresh install (no managed files exist yet) completes without any prompt regardless of whether `--force` is passed.

## Files That Need to Change

- `src/cli/loop_cmd.rs`
  - Add `force: bool` field to the `Install` variant in `LoopCommand`.
  - Extract a helper that checks which of the 21 managed paths already exist.
  - Add TTY detection (e.g., `std::io::IsTerminal` on stdin, stabilized in
    Rust 1.70) and stdin-reading for the `y/N` prompt.
  - Thread `force` through `run_install`.

## Dependencies

- None. Standalone enhancement; no dependency on other backlog tasks.
- `std::io::IsTerminal` requires Rust 1.70+. Verify the MSRV in `Cargo.toml`
  before implementing; bump if needed.

## Out of Scope

- Making `great loop install --project` prompt before overwriting
  `.tasks/reports/.template.md` — that is a follow-on task if users request it.
- Per-file granularity (e.g., `--force-agents`, `--force-commands`) — the
  single `--force` flag is sufficient for v1.
- Diff output showing what changed inside each file — a future `--diff` mode
  could be added separately.
