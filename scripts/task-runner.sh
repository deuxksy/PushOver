#!/usr/bin/env bash
# scripts/task-runner.sh — Reusable Makefile task wrapper
# Usage: bash scripts/task-runner.sh <task-name> bash -c '<command>'
set -uo pipefail

TASK_NAME="$1"; shift

LOG_FILE="MAKE.log"
LOG_MAX_BYTES=10485760  # 10 MB

# ── Log rotation ──
if [ -f "$LOG_FILE" ]; then
  size=$(stat -f%z "$LOG_FILE" 2>/dev/null || stat -c%s "$LOG_FILE" 2>/dev/null || echo 0)
  if [ "$size" -gt "$LOG_MAX_BYTES" ]; then
    gzip -c "$LOG_FILE" > "${LOG_FILE}.1.gz" 2>/dev/null || true
    : > "$LOG_FILE"
  fi
fi

# ── Timing ──
START_EPOCH=$(date +%s)
START_FMT=$(date -u +"%Y-%m-%d %H:%M:%S UTC")

# ── Execute ──
set +e
"$@"
EXIT_CODE=$?
set -e

END_EPOCH=$(date +%s)
END_FMT=$(date -u +"%Y-%m-%d %H:%M:%S UTC")

ELAPSED=$((END_EPOCH - START_EPOCH))
ELAPSED_FMT=$(printf "%02dm %02ds" $((ELAPSED / 60)) $((ELAPSED % 60)))

# ── Status line ──
if [ "$EXIT_CODE" -eq 0 ]; then
  STATUS="[✓]"
  LINE="${STATUS} ${TASK_NAME} | START: ${START_FMT} | END: ${END_FMT} | ELAPSED: ${ELAPSED_FMT}"
else
  STATUS="[✗]"
  LINE="${STATUS} ${TASK_NAME} | START: ${START_FMT} | END: ${END_FMT} | ELAPSED: ${ELAPSED_FMT} | EXIT: ${EXIT_CODE}"
fi

echo "$LINE"
echo "$LINE" >> "$LOG_FILE"
exit "$EXIT_CODE"
