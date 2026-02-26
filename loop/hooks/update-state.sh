#!/usr/bin/env bash
# update-state.sh -- Claude Code hook handler for great-loop state tracking
# Receives JSON on stdin from Claude Code. Writes session-scoped state file.
# Dependencies: jq (required)
set -euo pipefail

# --- Read stdin ---
INPUT="$(cat)"

# --- Extract common fields ---
SESSION_ID="$(echo "$INPUT" | jq -r '.session_id // empty')"
EVENT="$(echo "$INPUT" | jq -r '.hook_event_name // empty')"

# Bail silently if missing critical fields (do not block Claude Code)
if [[ -z "$SESSION_ID" || -z "$EVENT" ]]; then
  exit 0
fi

# --- Validate session_id (defense-in-depth against path traversal) ---
if [[ ! "$SESSION_ID" =~ ^[a-zA-Z0-9._-]+$ ]]; then
  exit 0
fi
if [[ ${#SESSION_ID} -gt 200 ]]; then
  exit 0
fi

# --- Derive paths ---
STATE_DIR="/tmp/great-loop/${SESSION_ID}"
STATE_FILE="${STATE_DIR}/state.json"

# --- Handle SessionEnd: cleanup and exit ---
if [[ "$EVENT" == "SessionEnd" ]]; then
  rm -rf "$STATE_DIR"
  exit 0
fi

# --- Ensure state directory exists ---
mkdir -p "$STATE_DIR"

# --- Initialize state file if absent ---
NOW="$(date +%s)"
if [[ ! -f "$STATE_FILE" ]]; then
  INIT_TMP="${STATE_DIR}/state.json.init.$$"
  echo "{\"loop_id\":\"${SESSION_ID}\",\"started_at\":${NOW},\"agents\":[]}" > "$INIT_TMP"
  mv "$INIT_TMP" "$STATE_FILE"
fi

# --- Determine agent identity and status ---
case "$EVENT" in
  SubagentStart)
    AGENT_KEY="$(echo "$INPUT" | jq -r '.agent_id // empty')"
    NEW_STATUS="running"
    ;;
  SubagentStop)
    AGENT_KEY="$(echo "$INPUT" | jq -r '.agent_id // empty')"
    NEW_STATUS="done"
    ;;
  TeammateIdle)
    AGENT_KEY="$(echo "$INPUT" | jq -r '.teammate_name // empty')"
    NEW_STATUS="idle"
    ;;
  TaskCompleted)
    # Use teammate_name if present (team context), else task_id
    AGENT_KEY="$(echo "$INPUT" | jq -r '.teammate_name // .task_id // empty')"
    NEW_STATUS="done"
    ;;
  Stop)
    AGENT_KEY="main"
    NEW_STATUS="done"
    ;;
  *)
    # Unknown event -- ignore silently
    exit 0
    ;;
esac

# Bail if we could not determine an agent key
if [[ -z "$AGENT_KEY" ]]; then
  exit 0
fi

# --- Atomic state update (serialized with flock on Linux) ---
# Read current state, upsert agent, write to temp, then mv.
TMPFILE="${STATE_DIR}/state.json.tmp.$$"

# Clean up temp file on any exit (including set -e failures)
trap 'rm -f "$TMPFILE"' EXIT

# flock is Linux-only (util-linux). On macOS it is absent unless installed
# via Homebrew; we degrade gracefully to the racy-but-mostly-correct path.
# -w 5 bounds the wait; || true prevents set -e from aborting on timeout.
if command -v flock >/dev/null 2>&1; then
  exec 9>"${STATE_DIR}/.lock"
  flock -w 5 9 || true
fi

# If state.json is corrupt (not valid JSON), re-initialize it so jq can proceed.
if ! jq empty "$STATE_FILE" 2>/dev/null; then
  INIT_TMP="${STATE_DIR}/state.json.init.$$"
  echo "{\"loop_id\":\"${SESSION_ID}\",\"started_at\":${NOW},\"agents\":[]}" > "$INIT_TMP"
  mv "$INIT_TMP" "$STATE_FILE"
fi

jq --arg key "$AGENT_KEY" \
   --arg status "$NEW_STATUS" \
   --argjson now "$NOW" \
   '
   # Find existing agent index by matching .name == $key
   (.agents | map(.name) | index($key)) as $idx |
   if $idx != null then
     # Update existing agent
     .agents[$idx].status = $status |
     .agents[$idx].updated_at = $now
   else
     # Append new agent with next sequential id
     .agents += [{
       "id": ((.agents | map(.id) | max // 0) + 1),
       "name": $key,
       "status": $status,
       "updated_at": $now
     }]
   end
   ' "$STATE_FILE" > "$TMPFILE" && mv "$TMPFILE" "$STATE_FILE"

exit 0
