# Release Notes: Task 0036 — Fix Site Sync Misinformation

**Date:** 2026-02-28
**Scope:** `site/src/data/features.ts`, `site/src/data/comparison.ts`, `site/src/data/commands.ts`, `site/src/components/sections/HowItWorks.tsx`, `site/src/components/sections/OpenSource.tsx`, `site/src/components/sections/Features.tsx`

---

## What Changed

The marketing site contained several claims about credential and sync behaviour that did not match the actual CLI. This update corrects those claims. No CLI code was modified.

### 1. Feature card: "Cloud-Synced Credentials" renamed to "Credential Vault"

**Before:** "Store API keys securely with client-side encryption and cross-machine sync."

**After:** "Store API keys in your system keychain, import from .env files, and snapshot config locally. BYO credentials — cloud sync coming soon."

The vault stores secrets in the OS system keychain and reads `.env` files. It does not perform cloud sync or client-side encryption in any current release. The new description reflects what the `great vault` command actually does.

### 2. Features section subtitle: "cross-machine sync" corrected to "config sync"

The subtitle listed "cross-machine sync" as one of the five layers great.sh covers. Config sync is local only. The phrase is now "config sync", which accurately describes the `great sync` snapshot behaviour.

### 3. Comparison table: "Cross-machine sync" row for great.sh changed from checkmark to "Local only"

**Before:** great.sh showed `true` (checkmark) for "Cross-machine sync".

**After:** great.sh shows `"Local only"` for that row.

chezmoi and Nix retain their "Git-based" values, which are accurate. great.sh does not offer cloud or Git-based sync at this time.

### 4. HowItWorks step 4: "Sync" renamed to "Snapshot"

**Before:** title "Sync", description referenced pushing config to the cloud and pulling on a new machine.

**After:** title "Snapshot", description: "Save a local config snapshot. Restore it anytime, or on a fresh install."

The `great sync push` command writes a local snapshot file. It does not push to any remote service. The step title and description now reflect this.

### 5. OpenSource section: removed false paid-tier and client-side encryption claims

**Before:** The section described a future paid tier and stated that API keys were protected by client-side encryption.

**After:** "The CLI is free and open source under the Apache 2.0 license. Every feature works without an account. No paywalls, no telemetry — the tool is yours to keep, forever." The secondary line now reads: "BYO credentials. We never see your API keys. Secrets stay in your system keychain."

There is no paid tier, no cloud back-end, and no client-side encryption in the current release. The revised copy is accurate.

### 6. Commands demo: "encrypted vault" replaced with "system keychain"

The `great init` interactive output referred to an "encrypted vault". The vault delegates to the OS keychain (Keychain on macOS, Secret Service on Linux, Credential Manager on Windows). That mechanism is accurately described as "system keychain". The demo output has been updated accordingly.

---

## Why It Matters

Incorrect feature claims on the marketing site mislead prospective users about what the tool does before they install it. A user who installs great.sh expecting cloud sync or client-side encryption will find neither. Accurate copy prevents frustration and sets correct expectations from the first contact with the project.

---

## No Migration Required

This is a site content update only. No CLI behaviour changed, no configuration format changed, and no existing installation is affected.
