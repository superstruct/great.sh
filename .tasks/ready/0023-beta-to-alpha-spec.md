# Spec 0023: Replace "beta" with "alpha" across marketing site and README

**Size:** XS (5 text replacements, 4 files, zero logic changes)

## Summary

The project is pre-release and should be labeled "alpha," not "beta." This task performs five exact string replacements across four files to correct the stage label. No functional code, styles, or build configuration changes.

## Changes

### 1. `site/src/components/layout/Nav.tsx` -- line 38

```diff
-            beta
+            alpha
```

The badge pill next to "great.sh" in the navigation bar.

### 2. `site/src/components/sections/Hero.tsx` -- line 27

```diff
-            beta — open to testing &amp; feedback
+            alpha — open to testing &amp; feedback
```

The status pill above the hero heading. Note: `&amp;` is the JSX-escaped ampersand and must remain unchanged.

### 3. `site/src/components/sections/OpenSource.tsx` -- line 50

```diff
-            great.sh is in beta. Found a bug or have a suggestion?{' '}
+            great.sh is in alpha. Found a bug or have a suggestion?{' '}
```

Inline sentence in the open-source section footer.

### 4. `README.md` -- line 3

```diff
-**The managed AI dev environment.** Beta — open to testing and feedback.
+**The managed AI dev environment.** Alpha — open to testing and feedback.
```

Sentence-initial, so capital "A" in "Alpha."

### 5. `README.md` -- line 77

```diff
-Beta (v0.1.0). Core features work. We welcome bug reports and feedback.
+Alpha (v0.1.0). Core features work. We welcome bug reports and feedback.
```

Sentence-initial, so capital "A" in "Alpha."

## Implementation Approach

1. Open each file.
2. Apply the exact string replacement shown above.
3. No other occurrences of "beta" exist in these files that should be left untouched (confirmed by read).

**Build order:** Files are independent; all five edits can be made in any order or in parallel.

## Files to Modify

| File | Type |
|------|------|
| `site/src/components/layout/Nav.tsx` | TSX (React component) |
| `site/src/components/sections/Hero.tsx` | TSX (React component) |
| `site/src/components/sections/OpenSource.tsx` | TSX (React component) |
| `README.md` | Markdown |

No files to create or delete.

## Edge Cases

- **Case sensitivity:** Lines 3 and 77 of README.md and line 50 of OpenSource.tsx start with or contain sentence-initial "Beta" which becomes "Alpha" (capital A). Lines 27 of Hero.tsx and 38 of Nav.tsx use lowercase "beta" becoming lowercase "alpha."
- **HTML entities:** Hero.tsx line 27 contains `&amp;` -- only the word "beta" is replaced; the entity must not be touched.
- **Trailing JSX expressions:** OpenSource.tsx line 50 ends with `{' '}` -- only the word "beta" within the sentence is replaced.

## Error Handling

Not applicable -- these are static text changes with no runtime behavior.

## Security Considerations

None. No user input, no secrets, no network calls affected.

## Testing Strategy

1. **`pnpm build:site`** -- confirms the site still compiles with no TSX errors.
2. **Visual spot-check** -- run `pnpm dev`, confirm the three site locations (nav badge, hero pill, open-source footer) all read "alpha."
3. **`grep -ri "beta" site/src/components/ README.md`** -- confirm zero remaining occurrences of the word "beta" in these four files.

## Acceptance Criteria

- [ ] All five occurrences replaced as specified.
- [ ] `pnpm build:site` passes.
- [ ] `grep -ri "beta" site/src/components/layout/Nav.tsx site/src/components/sections/Hero.tsx site/src/components/sections/OpenSource.tsx README.md` returns no matches.
