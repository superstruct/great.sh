VERDICT: PASS

Measurements:
- artifact_size: 9,037,376 bytes / 8.619 MiB (delta: 0% — no build required; change is structural refactor only)
- benchmark: none configured
- new_dependencies: 0

Summary: Pure control-flow restructure in `run_json()` (`/home/isaac/src/sh.great/src/cli/status.rs` lines 311–355) — closure chain replaced with explicit `if let` loops to permit mutable borrow of `issues` Vec; same iteration count, same data, one `format!()` per missing tool on the error path only; no new allocations, no new dependencies, no regression.
