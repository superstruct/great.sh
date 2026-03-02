# Scout Report: 0023 — Replace 'beta' with 'alpha'

**Scout:** Alexander von Humboldt
**Date:** 2026-02-26
**Size:** XS — 4 files, 5 text replacements

---

## 1. Confirmed Replacement Sites

### `site/src/components/layout/Nav.tsx` — line 38

```tsx
// Current (line 38):
            beta
// Replace with:
            alpha
```

Context: text content of a `<span>` badge in the site logo. No class name or ID — pure display text. The surrounding CSS classes (`text-accent`, `border-accent/30`, `bg-accent-muted`, `rounded-full`) are generic and do not reference "beta".

---

### `site/src/components/sections/Hero.tsx` — line 27

```tsx
// Current (line 27):
            beta — open to testing &amp; feedback
// Replace with:
            alpha — open to testing &amp; feedback
```

Context: text content of a `<motion.span>` badge at the top of the Hero section. No attribute, class, or key references "beta".

---

### `site/src/components/sections/OpenSource.tsx` — line 50

```tsx
// Current (line 50):
            great.sh is in beta. Found a bug or have a suggestion?{' '}
// Replace with:
            great.sh is in alpha. Found a bug or have a suggestion?{' '}
```

Context: plain text inside a `<p>` element. No surrounding class or ID references "beta".

---

### `README.md` — line 3

```md
// Current:
**The managed AI dev environment.** Beta — open to testing and feedback.
// Replace with:
**The managed AI dev environment.** Alpha — open to testing and feedback.
```

Note: "Beta" is capitalised here. Replacement must also be capitalised: "Alpha".

---

### `README.md` — line 77

```md
// Current:
Beta (v0.1.0). Core features work. We welcome bug reports and feedback.
// Replace with:
Alpha (v0.1.0). Core features work. We welcome bug reports and feedback.
```

Note: "Beta" is capitalised here. Replacement must also be capitalised: "Alpha".

---

## 2. Do-Not-Change: "beta" References in Rust Source

The following occurrences of "beta" in Rust source are test data / technical identifiers. They must NOT be changed.

| File | Line | Content | Reason |
|------|------|---------|--------|
| `src/mcp/mod.rs` | 231 | `config.add_server("beta", &mcp_b)` | test fixture — MCP server name string |
| `src/mcp/mod.rs` | 237 | `assert_eq!(names[1], "beta")` | test assertion against that fixture |
| `src/cli/apply.rs` | 960 | `std::env::set_var("GREAT_TEST_B", "beta")` | test fixture — env var value |
| `src/cli/apply.rs` | 962 | `assert_eq!(result, "alpha and beta")` | test assertion — "beta" is the env var value, not a release stage label |
| `src/platform/runtime.rs` | 169 | `"2.0.0-beta" does NOT match "2.0.0"` | doc comment — semver pre-release suffix |
| `src/platform/runtime.rs` | 310 | `version_matches("stable", "2.0.0-beta")` | test case — semver pre-release suffix |

None of these are release-stage labels. All must be left untouched.

---

## 3. Build System

`site/package.json` scripts:
```json
"dev":     "vite",
"build":   "tsc -b && vite build",
"preview": "vite preview"
```

**There is no `pnpm build:site` script in `site/package.json`.** The root-level `CLAUDE.md` documents `pnpm build:site` as the build command — that script must be defined in the root `package.json` (not the site sub-package). Da Vinci should run `pnpm build:site` from the repo root to verify the site builds cleanly after edits. If that command does not exist at root, fall back to `cd site && pnpm build`.

---

## 4. Dependency Map

All 5 replacements are isolated display text. No component imports "beta" as a constant or prop. No CSS utility class is named "beta". No test ID or `data-testid` contains "beta". Changes are independent — any order is safe.

---

## 5. Recommended Build Order

1. Edit `site/src/components/layout/Nav.tsx` line 38
2. Edit `site/src/components/sections/Hero.tsx` line 27
3. Edit `site/src/components/sections/OpenSource.tsx` line 50
4. Edit `README.md` lines 3 and 77 (two replacements, same file)
5. Run `pnpm build:site` (from repo root) or `pnpm build` (from `site/`) — verify no TypeScript or Vite errors
6. Commit

---

## 6. Risks

- **Capitalisation mismatch** — README lines 3 and 77 use "Beta" (capital B). The replacement must be "Alpha" (capital A), not "alpha". All site component replacements are lowercase and should remain lowercase.
- **No other "beta" badge in other site components** — confirmed by full-repo grep. The three site components enumerated above are the complete set.
- **Build script naming** — `pnpm build:site` is documented in CLAUDE.md but not present in `site/package.json`. This is pre-existing technical debt; it does not affect these edits.
