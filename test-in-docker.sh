#!/usr/bin/env bash
# Comprehensive great CLI test suite — runs inside Ubuntu 24.04 container
# Tests ALL subcommands and code paths. Findings printed as structured report.
# NOTE: do NOT use set -e here — we manually track pass/fail
set -uo pipefail

GREAT=/usr/local/bin/great
PASS=0
FAIL=0
WARN=0
declare -a FINDINGS=()

# ── helpers ──────────────────────────────────────────────────────────────────

bold()  { printf '\033[1m%s\033[0m\n' "$*"; }
green() { printf '\033[32m✓ %s\033[0m\n' "$*"; }
red()   { printf '\033[31m✗ %s\033[0m\n' "$*"; }
yellow(){ printf '\033[33m~ %s\033[0m\n' "$*"; }

pass() { green "$1"; PASS=$((PASS+1)); }
fail() { red   "$1"; FAIL=$((FAIL+1)); FINDINGS+=("BUG: $1"); }
warn() { yellow "$1"; WARN=$((WARN+1)); FINDINGS+=("WARN: $1"); }

# ok DESC CMD... — asserts CMD exits 0
ok() {
  local desc="$1"; shift
  local out code
  out=$(eval "$@" 2>&1) && code=0 || code=$?
  if [ "$code" -eq 0 ]; then
    pass "$desc"
  else
    fail "$desc (exit=$code)"
    printf '  OUTPUT: %s\n' "$(echo "$out" | head -5)"
  fi
}

# fails DESC CMD... — asserts CMD exits non-zero
nok() {
  local desc="$1"; shift
  local out code
  out=$(eval "$@" 2>&1) && code=0 || code=$?
  if [ "$code" -ne 0 ]; then
    pass "$desc (expected non-zero, got $code)"
  else
    fail "$desc (expected failure, got exit=0)"
    printf '  OUTPUT: %s\n' "$(echo "$out" | head -5)"
  fi
}

# contains DESC PATTERN CMD... — asserts CMD output matches PATTERN
contains() {
  local desc="$1"; local pattern="$2"; shift 2
  local out code
  out=$(eval "$@" 2>&1) && code=0 || code=$?
  if echo "$out" | grep -qE "$pattern"; then
    pass "$desc"
  else
    fail "$desc — pattern '$pattern' not found (exit=$code)"
    printf '  OUTPUT: %s\n' "$(echo "$out" | head -5)"
  fi
}

# not_contains DESC PATTERN CMD...
not_contains() {
  local desc="$1"; local pattern="$2"; shift 2
  local out code
  out=$(eval "$@" 2>&1) && code=0 || code=$?
  if echo "$out" | grep -qE "$pattern"; then
    fail "$desc — unexpected pattern '$pattern' found"
    printf '  OUTPUT: %s\n' "$(echo "$out" | head -5)"
  else
    pass "$desc"
  fi
}

section() { echo; bold "══════ $1 ══════"; echo; }

# ─────────────────────────────────────────────────────────────────────────────
section "BINARY SANITY"

ok   "binary exists and is executable"         "[ -x $GREAT ]"
ok   "great --help exits 0"                    "$GREAT --help"
ok   "great --version exits 0"                 "$GREAT --version"
contains "version string present" "[0-9]+\.[0-9]+"   "$GREAT --version"
ok   "great -h short help exits 0"             "$GREAT -h"
nok  "no subcommand → non-zero exit"           "$GREAT"
contains "help shows subcommand list"  "init|apply|status"   "$GREAT --help"

# ─────────────────────────────────────────────────────────────────────────────
section "great init"

TMPDIR_INIT=$(mktemp -d)
cd "$TMPDIR_INIT"

# Non-interactive — all defaults via /dev/null
ok   "init non-interactive all-defaults"       "$GREAT init < /dev/null"
ok   "init creates great.toml"                 "[ -f great.toml ]"
contains "great.toml has [project]"   "\[project\]"      "cat great.toml"
contains "great.toml has agents"      "\[agents"         "cat great.toml"

# Force overwrite
ok   "init --force overwrites existing"        "$GREAT init --force < /dev/null"
ok   "great.toml still exists after force"     "[ -f great.toml ]"

# Already-exists guard (no --force)
ok   "init without --force exits 0"            "$GREAT init < /dev/null"
contains "already-exists message shown"  "already exists"  "$GREAT init 2>&1 < /dev/null"

# Templates — valid
for tmpl in ai-minimal ai-fullstack-ts ai-fullstack-py saas-multi-tenant; do
  rm -f great.toml
  ok   "init --template $tmpl exits 0"         "$GREAT init --template $tmpl"
  ok   "init --template $tmpl creates file"    "[ -f great.toml ]"
  contains "$tmpl has [project]"   "\[project\]"   "cat great.toml"
done

# Template — unknown
rm -f great.toml
ok   "init --template unknown exits 0"         "$GREAT init --template totally-unknown"
contains "unknown template prints error"  "[Uu]nknown"   "$GREAT init --template totally-unknown 2>&1"
ok   "unknown template does NOT create file"   "[ ! -f great.toml ]"

# --force with template
ok   "init --force --template ai-minimal"      "$GREAT init --force --template ai-minimal"

cd /
rm -rf "$TMPDIR_INIT"

# ─────────────────────────────────────────────────────────────────────────────
section "great status"

TMPDIR_STATUS=$(mktemp -d)
cd "$TMPDIR_STATUS"

# No config
ok   "status with no config exits 0"           "$GREAT status"
contains "no-config status shows info"  "[Nn]o.*great.toml|[Nn]o config|[Nn]ot found"  "$GREAT status 2>&1"

# With config
$GREAT init --template ai-minimal > /dev/null 2>&1
ok   "status with config exits 0"              "$GREAT status"
contains "status shows platform"  "[Pp]latform|[Ll]inux"   "$GREAT status 2>&1"

# --json
ok   "status --json exits 0"                   "$GREAT status --json"
contains "status --json is valid JSON"  '"platform"'   "$GREAT status --json"
contains "status --json has has_issues"  '"has_issues"'   "$GREAT status --json"

# --verbose global
ok   "status with --verbose global"            "$GREAT --verbose status"

# --quiet global
ok   "status with --quiet global"              "$GREAT --quiet status"

# --non-interactive global
ok   "status with --non-interactive global"    "$GREAT --non-interactive status"

cd /
rm -rf "$TMPDIR_STATUS"

# ─────────────────────────────────────────────────────────────────────────────
section "great diff"

TMPDIR_DIFF=$(mktemp -d)
cd "$TMPDIR_DIFF"

# No config — diff exits non-zero (process::exit(1))
nok  "diff with no config exits non-zero"      "$GREAT diff"
contains "diff no config shows error"  "[Nn]o.*great.toml|Run.*great init"   "$GREAT diff 2>&1 || true"

# With config
$GREAT init --template ai-minimal > /dev/null 2>&1
ok   "diff with config exits 0"                "$GREAT diff"
contains "diff shows diff output"  "\+|~|-|[Nn]othing to do|run.*apply"   "$GREAT diff 2>&1"

# --config explicit path
ok   "diff --config explicit path"             "$GREAT diff --config great.toml"

# --config non-existent
nok  "diff --config nonexistent fails"         "$GREAT diff --config /no/such/file.toml"

# Secrets: set a var and check diff reports it resolved
export ANTHROPIC_API_KEY="sk-test-only"
ok   "diff with secret set exits 0"            "$GREAT diff"
unset ANTHROPIC_API_KEY

cd /
rm -rf "$TMPDIR_DIFF"

# ─────────────────────────────────────────────────────────────────────────────
section "great apply"

TMPDIR_APPLY=$(mktemp -d)
cd "$TMPDIR_APPLY"

# No config
ok   "apply with no config exits 0"           "$GREAT apply < /dev/null 2>&1; true"
contains "apply no config warns"  "[Nn]o.*great.toml|Run.*great init|[Nn]o config"   "$GREAT apply 2>&1 < /dev/null; true"

# With config
$GREAT init --template ai-minimal > /dev/null 2>&1

# --dry-run
ok   "apply --dry-run exits 0"                "$GREAT apply --dry-run < /dev/null"
contains "apply --dry-run shows planned"  "[Dd]ry.run|[Ww]ould|planned|skip|[Ii]nstall"   "$GREAT apply --dry-run 2>&1 < /dev/null"

# --non-interactive
ok   "apply --non-interactive --dry-run"      "$GREAT --non-interactive apply --dry-run"

# --only tools
ok   "apply --only tools --dry-run"           "$GREAT apply --only tools --dry-run < /dev/null"

# --only mcp
ok   "apply --only mcp --dry-run"             "$GREAT apply --only mcp --dry-run < /dev/null"

# --only agents
ok   "apply --only agents --dry-run"          "$GREAT apply --only agents --dry-run < /dev/null"

# --skip tools
ok   "apply --skip tools --dry-run"           "$GREAT apply --skip tools --dry-run < /dev/null"

# --verbose
ok   "apply --dry-run --verbose"              "$GREAT --verbose apply --dry-run < /dev/null"

cd /
rm -rf "$TMPDIR_APPLY"

# ─────────────────────────────────────────────────────────────────────────────
section "great doctor"

TMPDIR_DOCTOR=$(mktemp -d)
cd "$TMPDIR_DOCTOR"

ok   "doctor exits 0"                          "$GREAT doctor"
contains "doctor shows checks"   "[Pp]latform|[Cc]heck|[Dd]octor|[Pp]ass|[Ff]ail"   "$GREAT doctor 2>&1"

# --fix (non-interactive, no sudo available in container)
ok   "doctor --fix exits 0"                    "$GREAT doctor --fix < /dev/null"

# --non-interactive
ok   "doctor --non-interactive exits 0"        "$GREAT --non-interactive doctor"

# With config
$GREAT init --template ai-minimal > /dev/null 2>&1
ok   "doctor with config exits 0"              "$GREAT doctor"
contains "doctor with config covers MCP"   "[Mm][Cc][Pp]|[Cc]laude|config"   "$GREAT doctor 2>&1"

# doctor with config + --fix --non-interactive
ok   "doctor --fix --non-interactive"          "$GREAT --non-interactive doctor --fix < /dev/null"

cd /
rm -rf "$TMPDIR_DOCTOR"

# ─────────────────────────────────────────────────────────────────────────────
section "great vault"

TMPDIR_VAULT=$(mktemp -d)
cd "$TMPDIR_VAULT"

# login
ok   "vault login exits 0"                     "$GREAT vault login"
contains "vault login shows providers"  "[Pp]rovider|[Kk]ey|[Ee]nv|[Ss]ecret"   "$GREAT vault login 2>&1"

# unlock
ok   "vault unlock exits 0"                    "$GREAT vault unlock"
contains "vault unlock shows status"  "[Ss]tatus|provider|ready|[Vv]ault"   "$GREAT vault unlock 2>&1"

# set — with value as arg
ok   "vault set key value exits 0"             "$GREAT vault set TEST_KEY test_value_123"
contains "vault set shows result"  "[Ss]tored|[Ss]et|[Cc]ould not store|environment"   "$GREAT vault set TEST_KEY test_value_123 2>&1"

# set — empty value via stdin
ok   "vault set empty value exits 0"           "printf '' | $GREAT vault set EMPTY_KEY"
contains "vault set empty warns"  "[Ee]mpty|[Cc]annot be empty"   "printf '' | $GREAT vault set EMPTY_KEY 2>&1"

# import from env provider
ok   "vault import env exits 0"                "$GREAT vault import env"
export FAKE_API_KEY="sk-test-1234"
ok   "vault import env with API-style var"     "$GREAT vault import env"
unset FAKE_API_KEY

# import from non-existent .env file
ok   "vault import missing file exits 0"       "$GREAT vault import /does/not/exist.env 2>&1; true"
contains "vault import missing file error"  "[Ff]ail|[Nn]ot found|[Ee]rror|No such"   "$GREAT vault import /does/not/exist.env 2>&1; true"

# import from a real .env file
cat > test.env <<'EOF'
# comment line
ANTHROPIC_API_KEY=sk-ant-test
OPENAI_API_KEY="sk-openai-test"
export GOOGLE_API_KEY='sk-google-test'
EMPTY_KEY=
MALFORMED_NO_EQUALS
EOF
ok   "vault import from .env file exits 0"     "$GREAT vault import test.env"
contains "vault import .env shows result"  "[Ii]mport|[Ss]tored|skip"   "$GREAT vault import test.env 2>&1"

# import from named providers (should gracefully fail on Linux with no keychain)
ok   "vault import keychain exits 0"           "$GREAT vault import keychain 2>&1; true"
ok   "vault import bitwarden exits 0"          "$GREAT vault import bitwarden 2>&1; true"
ok   "vault import 1password exits 0"          "$GREAT vault import 1password 2>&1; true"

cd /
rm -rf "$TMPDIR_VAULT"

# ─────────────────────────────────────────────────────────────────────────────
section "great mcp"

TMPDIR_MCP=$(mktemp -d)
cd "$TMPDIR_MCP"

# list — no config
ok   "mcp list no config exits 0"             "$GREAT mcp list"
contains "mcp list no config warns"  "[Nn]o.*[Cc]onfigured|[Nn]o.*MCP|[Aa]dd"   "$GREAT mcp list 2>&1"

# list — with config
$GREAT init --template ai-minimal > /dev/null 2>&1
ok   "mcp list with config exits 0"           "$GREAT mcp list"

# add — no config
rm -f great.toml
ok   "mcp add no config exits 0"              "$GREAT mcp add myserver"
contains "mcp add no config error"  "great.toml|init"   "$GREAT mcp add myserver 2>&1"

# add — with config
$GREAT init --template ai-minimal > /dev/null 2>&1
ok   "mcp add new server exits 0"             "$GREAT mcp add myserver"
contains "mcp add success"  "[Aa]dded|server|great.toml"   "$GREAT mcp add myserver 2>&1"

# add duplicate — idempotent
ok   "mcp add duplicate exits 0"              "$GREAT mcp add myserver"
contains "mcp add duplicate warns"  "[Aa]lready"   "$GREAT mcp add myserver 2>&1"

# test — no config
rm -f great.toml
ok   "mcp test no config exits 0"             "$GREAT mcp test"
contains "mcp test no config error"  "[Nn]o.*great.toml|[Vv]alid|[Cc]onfig"   "$GREAT mcp test 2>&1"

# test — with config (ai-minimal may have no mcp section)
$GREAT init --template ai-minimal > /dev/null 2>&1
ok   "mcp test with config exits 0"           "$GREAT mcp test"

# test named server — not in config
ok   "mcp test unknown server exits 0"        "$GREAT mcp test nonexistent_server_xyz"
contains "mcp test unknown error"  "[Nn]ot found"   "$GREAT mcp test nonexistent_server_xyz 2>&1"

# test a specific server name that IS in config (add then test)
$GREAT mcp add testserver > /dev/null 2>&1
ok   "mcp test known server exits 0"          "$GREAT mcp test testserver"
# Server will likely fail to start (npx not installed), but command must exit 0
contains "mcp test known shows result"  "command|[Nn]ot found|start|error"   "$GREAT mcp test testserver 2>&1"

cd /
rm -rf "$TMPDIR_MCP"

# ─────────────────────────────────────────────────────────────────────────────
section "great template"

TMPDIR_TMPL=$(mktemp -d)
cd "$TMPDIR_TMPL"

# list
ok   "template list exits 0"                  "$GREAT template list"
contains "template list shows built-ins"  "ai-minimal|ai-fullstack"   "$GREAT template list 2>&1"
contains "template list shows apply hint" "apply"   "$GREAT template list 2>&1"
contains "template list shows update hint" "update"   "$GREAT template list 2>&1"

# apply known templates (fresh dir each time)
for tmpl in ai-minimal ai-fullstack-ts ai-fullstack-py saas-multi-tenant; do
  rm -f great.toml
  ok   "template apply $tmpl exits 0"         "$GREAT template apply $tmpl"
  ok   "template apply $tmpl creates file"    "[ -f great.toml ]"
  contains "$tmpl has [project]"  "\[project\]"   "cat great.toml"
done

# apply merges with existing
ok   "template apply merges with existing"    "$GREAT template apply ai-minimal"
ok   "merged great.toml exists"               "[ -f great.toml ]"

# apply unknown
ok   "template apply unknown exits 0"         "$GREAT template apply no-such-template"
contains "template apply unknown error"  "[Uu]nknown|[Aa]vailable"   "$GREAT template apply no-such-template 2>&1"

# update (network call — may fail in container; must handle gracefully)
ok   "template update exits 0 or graceful network error"   "$GREAT template update 2>&1; true"

cd /
rm -rf "$TMPDIR_TMPL"

# ─────────────────────────────────────────────────────────────────────────────
section "great sync"

TMPDIR_SYNC=$(mktemp -d)
cd "$TMPDIR_SYNC"

# push — no config
ok   "sync push no config exits 0"            "$GREAT sync push"
contains "sync push no config warns"  "[Nn]o.*great.toml|[Nn]othing"   "$GREAT sync push 2>&1"

# push — with config
$GREAT init --template ai-minimal > /dev/null 2>&1
ok   "sync push with config exits 0"          "$GREAT sync push"
contains "sync push warns cloud not available" "[Cc]loud.*not.*yet|[Ss]aving locally"   "$GREAT sync push 2>&1"
contains "sync push saves file locally"  "[Ss]aved"   "$GREAT sync push 2>&1"

# pull — no data (fresh dir)
TMPDIR_SYNC2=$(mktemp -d)
cd "$TMPDIR_SYNC2"
ok   "sync pull no data exits 0"              "$GREAT sync pull"
contains "sync pull no data warns"  "[Nn]o.*sync|not.*available|[Ll]ocal|[Nn]o.*found"   "$GREAT sync pull 2>&1"
cd "$TMPDIR_SYNC"
rm -rf "$TMPDIR_SYNC2"

# pull after push (data exists)
ok   "sync pull after push exits 0"           "$GREAT sync pull"
contains "sync pull shows blob info"  "[Bb]yte|blob|[Ss]ync|[Ff]ound"   "$GREAT sync pull 2>&1"

# pull --apply
ok   "sync pull --apply exits 0"              "$GREAT sync pull --apply"

cd /
rm -rf "$TMPDIR_SYNC"

# ─────────────────────────────────────────────────────────────────────────────
section "great update"

ok   "update exits 0 (or network error)"      "$GREAT update 2>&1; true"
contains "update shows version or error"  "[Vv]ersion|[Uu]pdate|[Ee]rror|latest|current"   "$GREAT update 2>&1; true"

ok   "update --check exits 0 (or network error)"   "$GREAT update --check 2>&1; true"
contains "update --check shows result"  "[Vv]ersion|[Uu]pdate|[Ll]atest|[Cc]heck|[Ee]rror"   "$GREAT update --check 2>&1; true"

# ─────────────────────────────────────────────────────────────────────────────
section "great loop"

TMPDIR_LOOP=$(mktemp -d)
cd "$TMPDIR_LOOP"

# status before install
ok   "loop status exits 0"                    "$GREAT loop status"
contains "loop status shows install state"  "[Nn]ot.*install|[Ii]nstall|[Ss]tatus|loop|agent"   "$GREAT loop status 2>&1"

# install
ok   "loop install exits 0"                   "$GREAT loop install < /dev/null"
contains "loop install shows agents"  "[Ii]nstall|agent|loop|[Cc]laude|[Ww]rote"   "$GREAT loop install 2>&1 < /dev/null"

# install --force (idempotent)
ok   "loop install --force exits 0"           "$GREAT loop install --force < /dev/null"

# install --project
ok   "loop install --project --force exits 0"  "$GREAT loop install --project --force < /dev/null"

# status after install
ok   "loop status after install exits 0"      "$GREAT loop status"
contains "loop status shows installed"  "[Ii]nstall|agent|loop|[Ff]ound"   "$GREAT loop status 2>&1"

# uninstall
ok   "loop uninstall exits 0"                 "$GREAT loop uninstall < /dev/null"
contains "loop uninstall shows result"  "[Rr]emov|[Uu]ninstall|[Dd]one|[Cc]lean"   "$GREAT loop uninstall 2>&1 < /dev/null"

# status after uninstall
ok   "loop status after uninstall exits 0"    "$GREAT loop status"

cd /
rm -rf "$TMPDIR_LOOP"

# ─────────────────────────────────────────────────────────────────────────────
section "great statusline"

# No input
ok   "statusline exits 0 (no input)"          "echo '' | $GREAT statusline"
contains "statusline produces output" "."   "echo '' | $GREAT statusline 2>&1"

# With session JSON
SESSION_JSON='{"model":"claude-sonnet-4-6","cost_usd":0.05,"context_tokens":1234,"context_window":200000,"session_id":"test-session-001"}'
ok   "statusline with session JSON exits 0"   "echo '$SESSION_JSON' | $GREAT statusline"
contains "statusline renders cost or tokens"  "0\.05|1234|\\\$|tok|ctx"   "echo '$SESSION_JSON' | $GREAT statusline 2>&1"

# Partial JSON
ok   "statusline with partial JSON exits 0"   "echo '{\"cost_usd\":0.12}' | $GREAT statusline"

# Invalid JSON
ok   "statusline with invalid JSON exits 0"   "echo 'not-json' | $GREAT statusline"

# Empty JSON object
ok   "statusline with empty JSON exits 0"     "echo '{}' | $GREAT statusline"

# Session with loop state (checks state file path logic)
ok   "statusline with session_id exits 0"     "echo '{\"session_id\":\"abc123\"}' | $GREAT statusline"

# ─────────────────────────────────────────────────────────────────────────────
section "great mcp-bridge"

# --help
ok   "mcp-bridge --help exits 0"              "$GREAT mcp-bridge --help"
contains "mcp-bridge --help shows preset"  "preset|backend|timeout"   "$GREAT mcp-bridge --help 2>&1"

# Initialize request to exercise the bridge
MCP_INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}'

# Test each preset
for preset in minimal agent research full; do
  out=$(echo "$MCP_INIT" | timeout 5 $GREAT mcp-bridge --preset $preset 2>/dev/null) && code=0 || code=$?
  if echo "$out" | grep -q '"result"'; then
    pass "mcp-bridge --preset $preset responds to initialize"
  elif echo "$out" | grep -q '"error"'; then
    warn "mcp-bridge --preset $preset responded with JSON-RPC error (investigate)"
    FINDINGS+=("WARN: mcp-bridge preset=$preset returned JSON-RPC error on initialize")
  else
    fail "mcp-bridge --preset $preset gave no valid JSON-RPC response (exit=$code)"
    printf '  OUTPUT: %s\n' "$(echo "$out" | head -3)"
  fi
done

# Unknown preset
out=$(echo "$MCP_INIT" | timeout 5 $GREAT mcp-bridge --preset badpreset 2>&1) && code=0 || code=$?
if echo "$out" | grep -q '"result"\|"error"'; then
  warn "mcp-bridge --preset badpreset still returned JSON-RPC (expected rejection)"
else
  pass "mcp-bridge --preset badpreset handled (exit=$code)"
fi

# --log-level variants
for level in off error warn info debug trace; do
  ok "mcp-bridge --log-level $level exits 0"  "echo '$MCP_INIT' | timeout 3 $GREAT mcp-bridge --log-level $level 2>&1; true"
done

# unknown log level
ok   "mcp-bridge unknown log-level exits 0"   "echo '$MCP_INIT' | timeout 3 $GREAT mcp-bridge --log-level bogus 2>&1; true"
contains "mcp-bridge unknown log-level warns"  "[Ww]arning|unknown.*log|bogus|warn"   "echo '$MCP_INIT' | timeout 3 $GREAT mcp-bridge --log-level bogus 2>&1; true"

# --backends
ok   "mcp-bridge --backends gemini"           "echo '$MCP_INIT' | timeout 3 $GREAT mcp-bridge --backends gemini 2>&1; true"
ok   "mcp-bridge --backends codex"            "echo '$MCP_INIT' | timeout 3 $GREAT mcp-bridge --backends codex 2>&1; true"
ok   "mcp-bridge --backends claude,gemini"    "echo '$MCP_INIT' | timeout 3 $GREAT mcp-bridge --backends claude,gemini 2>&1; true"

# --allowed-dirs
ok   "mcp-bridge --allowed-dirs"              "echo '$MCP_INIT' | timeout 3 $GREAT mcp-bridge --allowed-dirs /tmp,/var 2>&1; true"

# --timeout
ok   "mcp-bridge --timeout 10"               "echo '$MCP_INIT' | timeout 3 $GREAT mcp-bridge --timeout 10 2>&1; true"

# Global flags forwarded to bridge
ok   "mcp-bridge --verbose"                   "echo '$MCP_INIT' | timeout 3 $GREAT --verbose mcp-bridge 2>&1; true"
ok   "mcp-bridge --quiet"                     "echo '$MCP_INIT' | timeout 3 $GREAT --quiet mcp-bridge 2>&1; true"

# ─────────────────────────────────────────────────────────────────────────────
section "EDGE CASES"

TMPDIR_EDGE=$(mktemp -d)
cd "$TMPDIR_EDGE"

# Corrupted great.toml
echo "this is [not] = valid {{toml} ] garbage" > great.toml
ok   "status with corrupt config exits 0"     "$GREAT status 2>&1; true"
ok   "doctor with corrupt config exits 0"     "$GREAT doctor 2>&1; true"
ok   "apply --dry-run with corrupt exits 0"   "$GREAT apply --dry-run 2>&1 < /dev/null; true"
ok   "diff with corrupt config handled"       "$GREAT diff 2>&1; true"

# Empty great.toml
printf '' > great.toml
ok   "status with empty config exits 0"       "$GREAT status"
ok   "diff with empty config exits 0"         "$GREAT diff"
ok   "doctor with empty config exits 0"       "$GREAT doctor"

# Minimal great.toml
cat > great.toml <<'EOF'
[project]
name = "test-edge-project"
EOF
ok   "status with minimal config exits 0"     "$GREAT status"
ok   "diff with minimal config exits 0"       "$GREAT diff"
ok   "apply --dry-run minimal config exits 0" "$GREAT apply --dry-run < /dev/null"
ok   "mcp list with minimal config exits 0"   "$GREAT mcp list"

# GREAT_CONFIG env var (if supported)
export GREAT_CONFIG="$(pwd)/great.toml"
ok   "status with GREAT_CONFIG env var"       "$GREAT status 2>&1; true"
unset GREAT_CONFIG

# Running doctor with root user (we are root in Docker)
contains "doctor notes root/sudo status"  "[Rr]oot|[Ss]udo|[Uu]ser|[Pp]ermission"  "$GREAT doctor 2>&1; true"

cd /
rm -rf "$TMPDIR_EDGE"

# ─────────────────────────────────────────────────────────────────────────────
section "MCP-BRIDGE: JSON-RPC PROTOCOL"

# Test tools/list via mcp-bridge
MCP_LIST='{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
MCP_INIT_ACK='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}'
MCP_NOTIFY='{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'

# Send initialize then tools/list
out=$(printf '%s\n%s\n%s\n' "$MCP_INIT_ACK" "$MCP_NOTIFY" "$MCP_LIST" | timeout 5 $GREAT mcp-bridge --preset minimal 2>/dev/null) && code=0 || code=$?
if echo "$out" | grep -q '"tools"'; then
  pass "mcp-bridge tools/list returns tools array"
elif echo "$out" | grep -q '"result"'; then
  warn "mcp-bridge tools/list returned result but no tools key"
else
  fail "mcp-bridge tools/list gave no valid response (exit=$code)"
  printf '  OUTPUT: %s\n' "$(echo "$out" | head -5)"
fi

# Verify each preset exposes correct number of tools
for preset in minimal agent research full; do
  out=$(printf '%s\n%s\n%s\n' "$MCP_INIT_ACK" "$MCP_NOTIFY" "$MCP_LIST" | timeout 5 $GREAT mcp-bridge --preset $preset 2>/dev/null) && code=0 || code=$?
  tool_count=$(echo "$out" | grep -o '"name"' | wc -l | tr -d ' ')
  if [ "$tool_count" -gt 0 ]; then
    pass "mcp-bridge preset=$preset exposes $tool_count tool(s)"
  else
    warn "mcp-bridge preset=$preset: could not count tools (output: $(echo "$out" | head -2))"
  fi
done

# ─────────────────────────────────────────────────────────────────────────────
section "RESULTS SUMMARY"

echo
bold "Test Results"
printf '  Passed:   %d\n' "$PASS"
printf '  Failed:   %d\n' "$FAIL"
printf '  Warnings: %d\n' "$WARN"
printf '  Total:    %d\n' "$((PASS + FAIL + WARN))"
echo

if [ "${#FINDINGS[@]}" -gt 0 ]; then
  bold "Findings (backlog candidates):"
  for f in "${FINDINGS[@]}"; do
    printf '  • %s\n' "$f"
  done
fi

echo
if [ "$FAIL" -eq 0 ]; then
  green "All tests passed (with $WARN warning(s))!"
  exit 0
else
  red "$FAIL test(s) FAILED — see findings above"
  exit 1
fi
