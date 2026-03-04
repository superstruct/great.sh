# Scout Report: 0039 -- Docker-on-WSL2 Container Falsely Detected as WSL

**Date:** 2026-03-04
**Scout:** Alexander von Humboldt
**File surveyed:** `/home/isaac/src/sh.great/src/platform/detection.rs` (603 lines total)

---

## 1. Exact Change Sites in Production Code

### `is_wsl()` -- lines 169-181

```
169: fn is_wsl() -> bool {
170:     if std::env::var("WSL_DISTRO_NAME").is_ok() {
171:         return true;
172:     }
173:
174:     if std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists() {
175:         return true;
176:     }
177:
178:     std::fs::read_to_string("/proc/version")
179:         .map(|v| v.to_lowercase().contains("microsoft"))
180:         .unwrap_or(false)
181: }
```

**Change:** Insert `if is_container() { return false; }` as the very first statement (before line 170).

---

### `is_wsl2()` -- lines 187-189

```
187: fn is_wsl2() -> bool {
188:     std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists()
189: }
```

**Change:** Insert `if is_container() { return false; }` before the path check (line 188 pushes down).

---

### New function `is_container()` -- insert after line 189, before line 191

Line 190 is blank. Line 191 begins `/// Parse the \`ID=\` field from...` (the `detect_linux_distro` doc comment). The insertion point is the blank line 190 -- expand it into the new function block.

No new imports are required. The function uses only:
- `std::path::Path::new(...).exists()` -- already used in `is_wsl()` at line 174
- `std::env::var(...)` -- already used in `is_wsl()` at line 170
- `std::fs::read_to_string(...)` -- already used in `is_wsl()` at line 178

All three are in scope via the crate root (`std::`). No `use` statements needed.

---

## 2. `OsProbe` Trait -- lines 294-300

```
294: #[cfg(test)]
295: trait OsProbe {
296:     fn read_file(&self, path: &str) -> Option<String>;
297:     fn env_var(&self, name: &str) -> Option<String>;
298:     fn path_exists(&self, path: &str) -> bool;
299:     fn command_output(&self, cmd: &str, args: &[&str]) -> Option<String>;
300: }
```

**All three methods needed by `is_container_with_probe()` already exist:**
- `path_exists` -- covers `/.dockerenv`
- `env_var` -- covers `DOCKER_CONTAINER` and `container`
- `read_file` -- covers `/proc/1/cgroup`

**No trait extension required.** The spec's section 2.3 is confirmed correct.

---

## 3. `MockProbe` -- lines 368-403

Defined inside `mod tests` (line 364), using `use super::*` (line 365). Fields:

```
368: struct MockProbe {
369:     files: HashMap<String, String>,
370:     env_vars: HashMap<String, String>,
371:     paths: HashMap<String, bool>,
372:     commands: HashMap<String, String>,
373: }
```

`impl MockProbe::new()` at line 376. `impl OsProbe for MockProbe` at line 386.

`path_exists` returns `self.paths.get(path).copied().unwrap_or(false)` (line 395-397) -- inserting `("/.dockerenv".into(), true)` into `probe.paths` is correct for container tests.

---

## 4. `is_wsl_with_probe()` -- lines 302-316

```
302: #[cfg(test)]
303: fn is_wsl_with_probe(probe: &dyn OsProbe) -> bool {
304:     if probe.env_var("WSL_DISTRO_NAME").is_some() {
305:         return true;
306:     }
307:
308:     if probe.path_exists("/proc/sys/fs/binfmt_misc/WSLInterop") {
309:         return true;
310:     }
311:
312:     probe
313:         .read_file("/proc/version")
314:         .map(|v| v.to_lowercase().contains("microsoft"))
315:         .unwrap_or(false)
316: }
```

**Change:** Insert `if is_container_with_probe(probe) { return false; }` before line 304.

---

## 5. `is_wsl2_with_probe()` -- lines 318-321

```
318: #[cfg(test)]
319: fn is_wsl2_with_probe(probe: &dyn OsProbe) -> bool {
320:     probe.path_exists("/proc/sys/fs/binfmt_misc/WSLInterop")
321: }
```

**Change:** Insert `if is_container_with_probe(probe) { return false; }` before line 320, converting the single-expression body into a block.

---

## 6. Insertion Point for `is_container_with_probe()`

The `_with_probe` helpers live in the outer module scope (not inside `mod tests`) between the `OsProbe` trait definition (line 294) and the `mod tests` block (line 363). The sequence is:

- Line 294: `OsProbe` trait
- Line 302: `is_wsl_with_probe`
- Line 318: `is_wsl2_with_probe`
- Line 323: `detect_linux_distro_with_probe`
- Line 346: `is_root_with_probe`
- Line 354: `detect_shell_with_probe`
- Line 359: separator comment / line 363: `mod tests`

**Insert `is_container_with_probe()` after line 321 (closing `}` of `is_wsl2_with_probe`), before line 323 (`detect_linux_distro_with_probe`).** This keeps container helpers adjacent to WSL helpers.

The function must be annotated `#[cfg(test)]` to match all other helpers in this block.

---

## 7. Test Module -- lines 363-603

```
363: #[cfg(test)]
364: mod tests {
365:     use super::*;
366:     use std::collections::HashMap;
```

Closes at line 603 with `}`.

**Machine-dependent tests (10):** lines 409-485.
**Mock-based tests (14):** lines 491-602.
  - First mock test: `test_wsl_detected_from_env_var` at line 491.
  - Last existing mock test: `test_detect_shell_unset` ends at line 601.
  - Line 602 is `}` closing `mod tests`.

**New tests (12, tests 25-36) insert at line 602, before the closing `}`.**

Exact insertion: after line 601 (`assert_eq!(detect_shell_with_probe(&probe), "unknown");`), before line 602 (`}`), before line 603 (`}`).

All 12 new test functions follow the `#[test]` / `fn test_*` pattern used consistently through the module. No additional imports are needed -- `MockProbe`, `is_container_with_probe`, `is_wsl_with_probe`, `is_wsl2_with_probe` are all pulled in by `use super::*` at line 365.

---

## 8. Callers of `is_wsl()` and `is_wsl2()`

Both functions are private (`fn`, not `pub fn`). Grep across all of `src/` confirms two call sites, both within `detection.rs` itself:

| Line | Call | Context |
|------|------|---------|
| 94 | `is_wsl()` | `detect_platform()` -- decides `Platform::Wsl` vs `Platform::Linux` |
| 157 | `is_wsl2()` | `detect_capabilities()` -- sets `PlatformCapabilities::is_wsl2` |

**No other files call these functions.** Adding the container guard fixes both downstream consumers automatically:
- `detect_platform()` will return `Platform::Linux` instead of `Platform::Wsl` in a Docker-on-WSL2 container.
- `detect_capabilities()` will set `is_wsl2: false` in the same environment.
- All callers of `detect_platform()` and `detect_platform_info()` (apply, status, doctor) inherit the fix without modification.

**Behavior impact on existing tests:** The machine-dependent `test_detect_capabilities` (line 468) checks `if !matches!(platform, Platform::Wsl { .. }) { assert!(!caps.is_wsl2); }`. This test runs on the CI Linux host (not WSL, not a Docker-on-WSL2 scenario), so the container guard will not trigger and behavior is unchanged.

---

## 9. Dependency Map

```
detect_platform()  line 83  ──calls──> is_wsl()       line 169  ──calls──> [NEW] is_container()  after line 189
detect_capabilities() line 149 ──calls──> is_wsl2()    line 187  ──calls──> [NEW] is_container()

[test only]
is_wsl_with_probe()   line 303  ──calls──> [NEW] is_container_with_probe()  after line 321
is_wsl2_with_probe()  line 319  ──calls──> [NEW] is_container_with_probe()
```

Cross-module consumers (read-only, no changes needed):
- `src/cli/status.rs` -- calls `detect_platform_info()`
- `src/cli/doctor.rs` -- calls `detect_platform_info()`
- `src/cli/apply.rs` -- calls `detect_platform()` / `detect_platform_info()`

---

## 10. Import Requirements

**None.** The new code uses only:
- `std::path::Path` -- already used in `is_wsl()` (line 174) and `is_wsl2()` (line 188)
- `std::env::var` -- already used in `is_wsl()` (line 170)
- `std::fs::read_to_string` -- already used in `is_wsl()` (line 178)

No `use` statements need to be added anywhere in the file.

---

## 11. Risks and Technical Debt

**Risk 1 (low): `is_container_with_probe` scope.**
The `_with_probe` helpers live outside `mod tests` but are gated `#[cfg(test)]`. This is the existing pattern (lines 302-357) and is intentional. Da Vinci must annotate `is_container_with_probe` with `#[cfg(test)]` -- the production `is_container()` has no `#[cfg(test)]` annotation and is not inside `mod tests`.

**Risk 2 (low): `is_container()` visibility from `is_wsl()` and `is_wsl2()`.**
Both `is_wsl()` and `is_wsl2()` are defined before `is_container()` in the file (lines 169, 187 vs insertion after line 189). Rust does not require forward declarations -- functions in the same module can call each other regardless of definition order. No issue.

**Risk 3 (low): Existing test regression on line 472-475.**
`test_detect_capabilities` on line 468 calls the real `detect_capabilities()` which calls the real `is_wsl2()`. On the CI Linux host (not WSL, not in a Docker-on-WSL2 container), `/.dockerenv` does not exist, no container env vars are set, and `/proc/1/cgroup` does not contain "docker" (PID 1 is init). The container guard returns `false`, WSL check proceeds normally, WSLInterop does not exist, `is_wsl2()` returns `false`. Test assertion `assert!(!caps.is_wsl2)` passes. No regression.

**Technical debt noted:** `is_wsl2()` at line 187 is used in `detect_capabilities()` to set `PlatformCapabilities::is_wsl2` but does NOT call `is_wsl()` first. On a hypothetical WSL1 system, `is_wsl()` would be `true` but `is_wsl2()` would be `false` (correct). Post-fix: in a Docker-on-WSL2 container, both return `false` (correct). This is pre-existing design, not introduced by this task.

---

## 12. Recommended Build Order

Matches spec section 5, confirmed by dependency analysis:

1. Add `is_container()` production function -- insert after line 189.
2. Add `is_container_with_probe()` test helper -- insert after line 321 (after `is_wsl2_with_probe`).
3. Modify `is_wsl()` -- add guard as first statement inside line 169 function body.
4. Modify `is_wsl2()` -- add guard as first statement inside line 187 function body.
5. Modify `is_wsl_with_probe()` -- add guard as first statement inside line 303 function body.
6. Modify `is_wsl2_with_probe()` -- add guard as first statement inside line 319 function body.
7. Append 12 new test functions inside `mod tests`, before the closing `}` at line 602.
8. Run `cargo test --lib platform::detection -- --nocapture` -- all 36 tests must pass.
9. Run `cargo clippy` -- no new warnings.

Steps 1-2 are independent and can be written together. Steps 3-6 depend on steps 1-2 being present. Step 7 depends on steps 1-6.

---

## 13. File Snapshot Summary

| Element | Location | Action |
|---------|----------|--------|
| `is_wsl()` | lines 169-181 | Modify: prepend container guard |
| `is_wsl2()` | lines 187-189 | Modify: prepend container guard |
| `is_container()` [new] | after line 189 | Insert: private production fn |
| `OsProbe` trait | lines 294-300 | No change needed |
| `is_wsl_with_probe()` | lines 302-316 | Modify: prepend container guard |
| `is_wsl2_with_probe()` | lines 318-321 | Modify: prepend container guard |
| `is_container_with_probe()` [new] | after line 321 | Insert: `#[cfg(test)]` fn |
| `MockProbe` struct | lines 368-403 | No change needed |
| New tests 25-36 | after line 601 | Insert: 12 `#[test]` fns |

Total lines added: approximately 80 (one new production fn ~12 lines, one new test helper ~12 lines, two guard statements ~6 lines each, 12 new test functions ~50 lines).
