#!/usr/bin/env bash
# Start a shipd instance pointed at the framework fixture workspace
# for E2E testing. Uses SHIP_DEV_PROJECT_DIR to short-circuit the
# worktree-aware resolver — without that, the daemon resolves up the
# git tree and ends up serving views from the wrong checkout.
#
# Usage:
#   bash packages/sail/runtime/fixtures/e2e/start-daemon.sh [PORT]
#
# Tears down any prior daemon on the same port first.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
PORT="${1:-9385}"
HOME_DIR="${SHIP_HOME:-/tmp/ship-fixtures-home}"
LOG="${SHIPD_LOG:-/tmp/shipd-fixtures.log}"
WORKSPACE="$ROOT/packages/sail/runtime/fixtures"

# Tear down any prior daemon on this port.
pkill -f "shipd.*--port $PORT" 2>/dev/null || true
sleep 1

mkdir -p "$HOME_DIR"
echo "starting shipd: workspace=$WORKSPACE port=$PORT home=$HOME_DIR"
SHIP_HOME="$HOME_DIR" \
  SHIP_DEV_PROJECT_DIR="$WORKSPACE" \
  "$ROOT/target/debug/shipd" \
    --port "$PORT" \
    --dev \
    --workspace "$WORKSPACE" \
  > "$LOG" 2>&1 &

# Wait for HTTP readiness.
for _ in $(seq 1 20); do
  if curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:$PORT/shell/" 2>/dev/null | grep -q '200'; then
    echo "ready: http://127.0.0.1:$PORT/"
    exit 0
  fi
  sleep 0.5
done
echo "daemon did not become ready within 10s — see $LOG" >&2
tail -20 "$LOG" >&2
exit 1
