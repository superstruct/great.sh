# 0025 — Pre-cache sudo credentials before Homebrew install in `great apply`

**Priority:** P2
**Type:** enhancement
**Module:** src/cli/apply.rs
**Status:** TODO

## Context

`great apply` on macOS runs the Homebrew install script with `NONINTERACTIVE=1`
(apply.rs:436, doctor.rs:123). This flag tells the Homebrew installer **not to
prompt for a password**. When sudo credentials aren't cached, the installer
fails immediately rather than asking the user — even though the user *can* sudo.

```
Need sudo access on macOS (e.g. the user great-sh needs to be an Administrator)
Failed to install Homebrew!
```

Meanwhile, `bootstrap.rs:377` runs `sudo apt-get install` **without** `-n`,
letting the password prompt appear normally. The codebase has two inconsistent
sudo strategies: interactive (bootstrap) and non-interactive (Homebrew).

### Root cause

`NONINTERACTIVE=1` suppresses the sudo password prompt. There is no pre-flight
`sudo -v` to cache credentials before the install script runs. The user has
admin rights and could authenticate, but never gets the chance.

### Affected files

| File | Lines | Issue |
|------|-------|-------|
| `src/cli/apply.rs` | 433–438 | `NONINTERACTIVE=1` prevents sudo prompt |
| `src/cli/doctor.rs` | 122–124 | Same `NONINTERACTIVE=1` pattern |
| `src/cli/bootstrap.rs` | 374–386 | Uses interactive sudo (correct — inconsistency) |

## Approach: Pre-cache sudo with `sudo -v`

Run `sudo -v` early in the apply flow — before any installs — to prompt once
and cache credentials. Then all subsequent sudo calls (Homebrew, apt, bootstrap)
use the cached token. This is the pattern used by `mas` (Mac App Store CLI),
Ansible's `become`, and macOS installer scripts.

### Implementation

**Step 1 — Determine if sudo will be needed**

Before the install phase, check whether any upcoming operations require sudo:
- Homebrew install (macOS, Linux when not already installed)
- `sudo apt-get` calls in bootstrap (Linux/WSL)
- Docker install (Linux)

```rust
let needs_sudo = (needs_homebrew && !info.capabilities.has_homebrew)
    || matches!(info.platform, Platform::Linux { .. } | Platform::Wsl { .. });
```

**Step 2 — Pre-cache credentials (interactive only)**

```rust
if std::io::stdin().is_terminal() && needs_sudo && !info.is_root {
    output::info("Some tools require administrator access. You may be prompted for your password.");
    let _ = std::process::Command::new("sudo")
        .args(["-v"])
        .status();
}
```

`sudo -v` validates the user's credentials and starts the cache timer
(default 5 minutes on macOS, 15 on most Linux). If already cached (e.g.
user ran `sudo` recently), it's a no-op.

**Step 3 — Keep credentials alive during long applies (optional)**

Spawn a background thread that refreshes the cache every 60 seconds:

```rust
let sudo_keepalive = if needs_sudo && !info.is_root {
    Some(std::thread::spawn(|| {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
            if std::process::Command::new("sudo")
                .args(["-vn"])  // non-interactive refresh
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .is_err()
            {
                break; // sudo gone, stop trying
            }
        }
    }))
} else {
    None
};
```

The `-vn` flag refreshes non-interactively — if the cache already expired
(e.g. user walked away for 20 min), it fails silently rather than prompting
mid-install. The thread is a daemon thread and dies when the process exits.

**Step 4 — `NONINTERACTIVE=1` now works**

With credentials cached, the existing Homebrew install command at apply.rs:436
succeeds as-is. No changes to the Homebrew invocation itself.

**Step 5 — Apply the same pattern to `doctor.rs`**

doctor.rs:122 has the same `NONINTERACTIVE=1` Homebrew install. Either:
- Extract a shared `ensure_sudo_cached()` helper used by both apply and doctor
- Or inline `sudo -v` before the Homebrew fix in doctor's `--fix` path

### Non-interactive fallback (CI)

When `!stdin().is_terminal()`, skip the `sudo -v` prompt entirely. The existing
`NONINTERACTIVE=1` behavior is correct for CI — fail fast rather than hang.
No change needed for this path.

## Acceptance Criteria

- [ ] When `great apply` runs interactively on macOS and Homebrew is absent,
      the user sees a single password prompt (`sudo -v`) before installation
      begins — not a failure.
- [ ] After the `sudo -v` prompt, the Homebrew install with `NONINTERACTIVE=1`
      succeeds using the cached credentials.
- [ ] `great doctor --fix` also pre-caches sudo before attempting Homebrew
      install.
- [ ] In non-interactive contexts (piped stdin, CI), no sudo prompt is
      attempted — existing fail-fast behavior is preserved.
- [ ] The sudo keepalive thread does not outlive the process or produce
      visible output.
- [ ] `bootstrap.rs` sudo calls (apt-get) also benefit from the cached
      credentials — no double-prompting.
- [ ] No new crate dependencies added.

## Dependencies

None — uses only `std::process::Command` and `std::io::stdin().is_terminal()`
(stabilised in Rust 1.70).

## Notes

- `sudo -v` cache timeout is configurable via `/etc/sudoers` (`timestamp_timeout`).
  Default is 5 min on macOS, 15 min on most Linux. The keepalive thread handles
  applies longer than the timeout.
- The `sudo2` crate was evaluated and rejected — it re-executes the entire
  binary as root, which is overkill and causes permission issues for user-owned
  files (mise installs, config). The problem is credential caching, not
  whole-process elevation.
- The consolidated error messaging from the original 0025 scope (listing
  skipped tools on Homebrew failure) is still a nice-to-have but is a separate
  concern — the pre-cache approach prevents the failure in the first place.
