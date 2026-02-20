# Review: Spec 0010 -- Complete All Stubs, TODOs, and Gaps

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-20
**Spec:** `/home/isaac/src/sh.great/.tasks/specs/0010-complete-all-stubs-spec.md`
**Backlog:** `/home/isaac/src/sh.great/.tasks/backlog/0010-complete-all-stubs.md`
**Verdict:** CONDITIONAL PASS -- 5 BLOCKs must be resolved, 11 WARNs noted

---

## BLOCK Items (must fix before building)

### BLOCK-1: GROUP A -- `install_with_spec` signature is incompatible with `PackageManager` trait

The spec defines `install_with_spec` with parameter `managers: &[Box<dyn PackageManager>]`, but this function needs to call individual install methods (`npm install -g`, `brew install`, `curl | bash`, `snap install`, `pip install`) that are **not** delegated through the `PackageManager` trait. The spec's algorithm (steps 1-5) spawns raw shell commands (`npm install -g`, `curl -fsSL`, `sudo snap install`, `pip install`) rather than calling `mgr.install()`.

**Question:** Why does the function accept `managers` at all if it never calls `mgr.install()` in steps 1-5? The only place it would use a `PackageManager` is in step 2 (brew), but the spec manually constructs the brew formula name from `BrewInstall::Named`. Is `managers` needed to check whether Homebrew is *available*, or does the function just call `command_exists("brew")` directly? This signature will confuse the builder.

**Fix required:** Clarify whether `managers` is used for availability detection only, or if the function actually delegates to the `PackageManager::install()` method (in which case the brew formula name remapping in `BrewInstall::Named` needs to be passed through `mgr.install(formula_name, version)`).

### BLOCK-2: GROUP C -- `discover_config()` searches parent directories, not `current_dir`

The spec's `run_add()` calls `config::discover_config()` to find `great.toml`, which walks up parent directories. But `mcp_table.insert(name, ...)` writes back to *that discovered path*. If `great.toml` is found in a parent directory (e.g., `/home/user/projects/great.toml`) while the user runs `great mcp add` from a subdirectory, the write goes to the parent.

This is actually consistent with how `run_list()` works today. **However**, the existing `run_add()` (lines 114-120 of `/home/isaac/src/sh.great/src/cli/mcp.rs`) already loads the config and checks for duplicates via `config::load()`. The spec replaces this with `toml_edit` parsing, but the duplicate-check logic reads `doc.get("mcp")` as a top-level key.

**Question:** In the TOML schema, MCP servers are declared as `[mcp.postgres]` which flattens to a top-level `mcp` table with a nested `postgres` key. But `GreatConfig` deserializes `mcp` as `Option<HashMap<String, McpConfig>>`. If someone has `[mcp]` as an empty table, `doc.get("mcp")` returns `Some(Table)`, but `table.contains_key(name)` will correctly return false. This seems fine. But what happens if the user has inline tables like `mcp = {}`? The `as_table()` call in step 4 would return `None` for an inline table, skipping the duplicate check entirely and potentially creating a duplicate. Is this a realistic concern?

**More critically:** The spec uses `toml_edit::DocumentMut` but the crate is called `toml_edit` version `0.22`. In `toml_edit` 0.22, the parsed document type is `toml_edit::DocumentMut` (introduced in 0.22 to replace the old `Document`). The line `raw.parse::<toml_edit::DocumentMut>()` is correct for 0.22. This is fine.

**Actual block:** The spec writes `doc["mcp"].as_table_mut()` on line 441, but this will panic if `"mcp"` was just inserted as an empty table in line 439. The `doc["mcp"]` indexing operator on `DocumentMut` returns `&Item`, but after `doc.insert("mcp", Item::Table(...))`, re-indexing with `doc["mcp"]` should work. Let me reconsider -- `DocumentMut` implements `Index<&str>` which returns `&Item`. But `.as_table_mut()` requires `&mut Item`. You need `doc["mcp"].as_table_mut()` but `doc["mcp"]` via `Index` returns a shared reference. The spec should use `doc.get_mut("mcp")` instead.

**Fix required:** Replace `doc["mcp"].as_table_mut()` with `doc.get_mut("mcp").and_then(|item| item.as_table_mut())` to avoid the shared/mutable reference conflict. Alternatively, use `doc["mcp"]` with `IndexMut` which requires `&mut doc["mcp"]`.

### BLOCK-3: GROUP D -- `check_*` function signatures diverge from existing code

The spec changes every `check_*` function to accept `fixable: &mut Vec<FixableIssue>` as a second parameter. The existing signatures are:

```
fn check_platform(result: &mut DiagnosticResult)
fn check_essential_tools(result: &mut DiagnosticResult)
fn check_ai_agents(result: &mut DiagnosticResult)
fn check_config(result: &mut DiagnosticResult)
fn check_shell(result: &mut DiagnosticResult)
```

But the spec also says doctor.rs should call `crate::cli::apply::find_install_spec` and `crate::cli::apply::install_with_spec`. These functions are currently private in `apply.rs`. The spec says to make them `pub(crate)`, plus `ToolInstallSpec` and `BrewInstall` types.

**Question:** The spec states that `check_essential_tools` should push `FixableIssue` entries. But `check_essential_tools` iterates over a hardcoded list of `(cmd, name, install_hint)` tuples. The `FixAction::InstallTool` carries `name: String` -- but which name? The command name (`cmd`) like `"gh"`, or the human-readable `name` like `"GitHub CLI"`? Looking at the fix logic, `find_install_spec(name)` is called, so it must be the command name. But the spec example shows `name: cmd.to_string()` -- is `cmd` the command name or the description? In the existing code, `cmd` is the first tuple element (e.g., `"git"`, `"curl"`, `"gh"`), which is correct. This is fine, but the spec should be explicit.

**Actual block:** The re-check after fixes calls the same `check_*` functions but with `&mut Vec::new()` as the fixable parameter. This silently discards any new fixable issues found on re-check. More importantly, the `DiagnosticResult` struct is extended with `fixes_attempted` and `fixes_succeeded` fields, but the spec says `DiagnosticResult::default()` for the recheck. The existing struct does NOT derive `Default`. The spec must either add `#[derive(Default)]` to `DiagnosticResult` or manually construct it.

**Fix required:** Add `#[derive(Default)]` to `DiagnosticResult` or show the manual construction. Also clarify that `cmd` in the `FixAction::InstallTool` means the CLI command name (first tuple element), not the human-readable description.

### BLOCK-4: GROUP E -- `async` in a sync `main()` creates nested runtime risk

The spec proposes creating a new `tokio::runtime::Runtime` inside `update.rs`. But `main()` is synchronous. The spec acknowledges this and recommends "Option A" with `tokio::runtime::Runtime::new()`.

**Question:** GROUP H (template update) uses the same pattern. If both are called in the same process (they won't be -- they're separate subcommands), there's no conflict. But what happens if someone later makes `main()` async with `#[tokio::main]`? Then `Runtime::new()` inside a tokio context will panic with "Cannot start a runtime from within a runtime."

**Actual block:** The spec shows `check_for_update().await` on line 768 inside the synchronous `run()` function. This will not compile -- you cannot use `.await` in a non-async function. The spec then shows the correct pattern below (wrapping in `rt.block_on()`), but the initial `run()` function listing is misleading and contradictory.

**Fix required:** Remove the `.await` calls from the synchronous `run()` function body (lines 768, 771). Show only the `rt.block_on()` wrapper pattern. Also add a comment warning about the nested-runtime risk if `main()` ever becomes async.

### BLOCK-5: GROUP E -- `platform_asset_name()` uses non-existent `Platform` method/pattern

The spec's `platform_asset_name()` function pattern-matches on `(&info.platform, info.platform.arch())`. But the match arms use `platform::Platform::MacOS { .. }` which is correct, and `platform::Architecture::Aarch64` which is also correct. However, the function calls `platform::detect_platform_info()` directly rather than accepting it as a parameter.

**Actual block:** The match arm `platform::Platform::Linux { .. } | platform::Platform::Wsl { .. }` uses an or-pattern. This is valid Rust syntax (since 1.53). But the match is on `(&info.platform, info.platform.arch())` which is a tuple of `(&Platform, Architecture)`. The first element is a reference. The patterns `(platform::Platform::MacOS { .. }, ...)` are matching against a *reference* to an enum. In Rust, pattern matching on `&Platform::MacOS { .. }` works due to auto-deref in match ergonomics. This will compile, but the or-pattern `platform::Platform::Linux { .. } | platform::Platform::Wsl { .. }` inside a tuple pattern should also work. Let me verify -- yes, this compiles.

However, `info.platform.arch()` is called on `info.platform` which is a `Platform` (not a reference). And `info` is a local binding from `detect_platform_info()`. The `arch()` method returns `Architecture` which is `Copy`. So `info.platform.arch()` returns `Architecture` by value. Matching `(ref, value)` against `(Pattern, Pattern)` is fine.

**Revised concern:** The match arm catches MacOS, Linux, and WSL for x86_64 and aarch64. But `Platform::Windows` is not handled. The `_ => anyhow::bail!(...)` covers it. The spec says "unsupported platform for self-update" which is correct for Windows since this is a Unix binary replacement pattern. But the spec's self-update uses `std::fs::rename` which works differently on Windows. Since the `_` arm catches Windows, this is fine.

**Downgrading to WARN -- see WARN-1.**

**Replacement BLOCK-5:** GROUP F -- `vault import` spec calls `vault::get_provider()` which returns `Option<Box<dyn SecretProvider>>`, but then passes the result to `import_from_provider(provider.as_ref())`. The `provider` is a `Box<dyn SecretProvider>`, so `provider.as_ref()` returns `&dyn SecretProvider`. This is correct. However, the spec's `import_from_provider` calls `source.list(None)` -- and both `OnePasswordProvider::list()` and `BitwardenProvider::list()` return `Err(bail!(...))`. This means importing from `1password` or `bitwarden` as a *source* will always fail at the `list()` call. The backlog item says "vault import from a provider that lists zero secrets prints 'nothing to import'" -- but importing from 1password/bitwarden will hit an error, not an empty list.

**Fix required:** Either (a) document that only `env` and `keychain` are supported as import sources (since those are the only providers that implement `list()` without erroring), or (b) add `list()` implementations to 1Password and Bitwarden providers, or (c) handle the `Err` from `list()` as "provider does not support listing" with a distinct message from a generic failure.

---

## WARN Items (proceed with caution)

### WARN-1: GROUP E -- No Windows self-update path

The self-update binary replacement uses Unix rename semantics. On Windows, replacing a running executable requires different techniques (e.g., `self_replace` crate). The backlog mentioned `self_replace` or `self-update` crates. The spec chose manual rename. Since `platform_asset_name()` bails on Windows, this is technically guarded, but there is no Windows-specific download/update path at all.

### WARN-2: GROUP A -- Security concern with `curl | bash` and `curl | sudo bash`

The spec prescribes `curl -fsSL <url> | bash` for starship/uv and `curl -fsSL <url> | sudo bash` for az. Piping curl to bash (especially with sudo) is a well-known security risk. The spec does not mention:
- TLS certificate verification (curl uses it by default, but no `--proto '=https'` flag)
- Checksum verification of downloaded scripts
- Any user confirmation before piping to sudo bash

The backlog does not require user confirmation, but `apply.rs` already has a `--yes` flag. Should `curl | sudo bash` require explicit `--yes` consent?

### WARN-3: GROUP B -- Shell profile modification without backup

The Starship configuration appends to `~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish` without creating a backup. If the append corrupts the file (unlikely but possible with concurrent writes), there is no recovery. The sync pull spec (GROUP G) creates `.bak` files -- should shell profiles get the same treatment?

### WARN-4: GROUP C -- `toml_edit` dependency adds compile time

The spec adds `toml_edit = "0.22"` for a single function (`mcp add`). The `toml` crate is already in `Cargo.toml`. Have you considered that `toml_edit` pulls in `winnow` (parser) as a dependency? This is a non-trivial addition to compile time for one feature. The alternative -- read as string, append `[mcp.name]` section to the end -- is simpler but less robust. The tradeoff seems acceptable given the format-preservation requirement, but should be noted.

### WARN-5: GROUP E -- GitHub API rate limiting not handled beyond error message

The spec mentions "GitHub API returned an error (possible rate limit)" as an error message, but does not implement:
- Checking `X-RateLimit-Remaining` headers
- Using `GITHUB_TOKEN` if available for authenticated requests (5000/hour vs 60/hour)
- Retry-after logic

For a CLI that users run frequently, hitting the 60/hour unauthenticated limit is realistic in CI environments.

### WARN-6: GROUP F -- `.env` file parsing is naive

The dotenv parser handles `KEY=VALUE`, `KEY="VALUE"`, `KEY='VALUE'`, and `export KEY=VALUE`. But it does not handle:
- Multiline values (`KEY="line1\nline2"` or heredoc-style)
- Escaped quotes inside quoted values (`KEY="value with \"quotes\""`)
- Inline comments (`KEY=VALUE # this is a comment`)
- Values with `=` signs (`KEY=base64string==`)

The `split_once('=')` approach handles the last case correctly (splits on first `=` only). The others are edge cases but real. The backlog does not require full dotenv compliance, so this is a WARN, not a BLOCK.

### WARN-7: GROUP G -- `config::load(Some(config_path.to_str().unwrap_or("great.toml")))` is fragile

In sync pull's apply path, the spec calls `config_path.to_str().unwrap_or("great.toml")`. If `config_path` contains non-UTF-8 characters, this silently falls back to `"great.toml"` which may be a different file. The `unwrap_or` hides a real error. This should use `.context("path is not valid UTF-8")?` instead.

### WARN-8: GROUP H -- Template name collision between built-in and downloaded

The spec says `run_apply()` checks built-in templates first, then falls back to downloaded. If a downloaded template has the same name as a built-in (e.g., `ai-minimal.toml`), the built-in always wins. The backlog says "Built-in templates are never overwritten (downloaded templates shadow them)" -- but the spec does the opposite: built-in templates shadow downloaded ones. This contradicts the backlog requirement.

### WARN-9: GROUP J -- Test assertions on stderr output depend on color/unicode support

The integration tests use `predicate::str::contains("Dry run mode")` on stderr. The `output::warning()` function prepends a Unicode warning symbol. In some CI environments, the terminal may not support Unicode, or `colored` may strip ANSI codes. The `predicate::str::contains` checks the raw bytes, so the Unicode character should still be present. But `colored` respects `NO_COLOR` and `TERM=dumb`. If CI sets these, the test might still pass since `contains` is checking the text portion, not the color codes. This should be fine, but worth noting.

### WARN-10: GROUP J -- `mcp_add_creates_entry` test checks stderr for "Added" but output goes through `output::success`

The test at line 479 of the spec expects `.stderr(predicate::str::contains("Added [mcp.postgres]"))`. The spec's `run_add()` calls `output::success(&format!("Added [mcp.{}] to {}", name, config_path.display()))`. Since `output::success()` writes to stderr, this is correct. However, the success message includes the full path (`config_path.display()`), so the assertion should work since `contains` does substring matching.

### WARN-11: GROUP K -- Docker test scripts copy entire repo including `target/` directory

The test scripts run `cp -r /workspace/. /build/`. The `/workspace` mount includes the `target/` directory which can be gigabytes in size. This will dramatically slow down the Docker tests. The scripts should either exclude `target/` or use `.dockerignore`. Since the volume is mounted read-only (`:ro`), the copy is necessary, but `cp -r /workspace/. /build/` should skip `target/`:

```bash
rsync -a --exclude target /workspace/ /build/
```

Or add a `.dockerignore` and use `COPY` in the Dockerfile instead of volume mounts.

---

## PASS Items

### PASS-1: GROUP A -- Tool mapping table is complete and covers all TODOs

The 8 tools listed in the TODO comment at `apply.rs:198-210` are all covered by `special_install_specs()`. The struct design with `BrewInstall::Default` vs `BrewInstall::Named` is clean.

### PASS-2: GROUP A -- Tests cover both known and unknown tools, plus the "at least one method" invariant

The test `test_all_specs_have_at_least_one_method` is a good structural invariant test.

### PASS-3: GROUP B -- Starship shell detection covers all three major shells

Bash, zsh, and fish are the three shells that Starship supports. The profile file paths are correct. The duplicate guard (`contains("starship init")`) is simple and effective.

### PASS-4: GROUP C -- Format-preserving TOML edit is the right approach

Using `toml_edit` instead of deserialize-modify-serialize avoids destroying comments and formatting in the user's `great.toml`. This is the correct architectural choice.

### PASS-5: GROUP E -- `CURRENT_VERSION` from `env!("CARGO_PKG_VERSION")` is correct

This matches the existing code in `update.rs` line 7.

### PASS-6: GROUP G -- Backup-before-overwrite pattern is correct

Creating `great.toml.bak` before overwriting is a good safety measure. The verify-after-write pattern (trying to parse the restored file) is also good.

### PASS-7: GROUP I -- `.unwrap()` audit is thorough

The spec correctly identifies every `unwrap_or("")` in production code and proposes `unwrap_or_default()` where appropriate. The `.expect("valid regex")` calls on compile-time-constant regexes are correctly assessed as acceptable.

### PASS-8: GROUP I -- Dead code analysis matches the actual codebase

`SyncBlob` and `SyncStatus` in `sync/mod.rs` are indeed unused. `GreatError::Network` is indeed only needed once GROUP E lands. The analysis is accurate.

### PASS-9: GROUP J -- Test list exceeds the backlog minimum of 12

17 tests proposed vs 12 required. The fixture-based approach using `tempfile::TempDir` is correct for isolation.

### PASS-10: GROUP K -- Docker containers correctly avoid KVM requirement

The Ubuntu and Fedora Dockerfiles use standard base images and do not require hardware virtualization. The heavyweight VM containers are correctly commented out.

### PASS-11: Build order respects dependency graph

The three-phase build order is correct. GROUP D depends on GROUP A (for `find_install_spec`), GROUP B depends on GROUP A (for starship being installable), GROUP H depends on GROUP E (for async/HTTP patterns), and GROUP K depends on GROUP J (for tests to exist).

### PASS-12: Error handling follows project conventions

All proposed error handling uses `anyhow::Result`, `context()`, and `bail!()` consistently with the existing codebase. No `.unwrap()` calls in proposed production code (the test code uses `.unwrap()` which is acceptable per convention).

---

## Summary

| Category | Count | Items |
|----------|-------|-------|
| BLOCK    | 5     | BLOCK-1 through BLOCK-5 |
| WARN     | 11    | WARN-1 through WARN-11 |
| PASS     | 12    | PASS-1 through PASS-12 |

**Required actions before building:**

1. **BLOCK-1:** Clarify `install_with_spec` parameter usage -- does it call `PackageManager::install()` or spawn raw commands? Reconcile the function signature with its algorithm.
2. **BLOCK-2:** Fix `doc["mcp"].as_table_mut()` to use `doc.get_mut("mcp")` for correct mutable access in `toml_edit`.
3. **BLOCK-3:** Add `#[derive(Default)]` to `DiagnosticResult` and clarify that `cmd` in `FixAction::InstallTool` refers to the CLI command name.
4. **BLOCK-4:** Remove `.await` syntax from the synchronous `run()` function in GROUP E. Show only the `rt.block_on()` wrapper.
5. **BLOCK-5:** Document which vault providers support `list()` for import, or handle the `Err` from providers that do not support listing with a distinct error message.

Once these five items are resolved, the spec is buildable.

---

*"The unexamined spec ships bugs by default."*
