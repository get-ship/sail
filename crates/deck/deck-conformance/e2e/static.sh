#!/usr/bin/env bash
# Slice 4 e2e — STATIC deploy (deck's `next export` analog).
#
# Build the SPA with `deck build`, serve dist/ with a DUMB static file server
# (no daemon, no deck, no ship), and prove the counter renders + increments in
# a real browser. This is the "static" rendering mode.
#
# It hand-stages the 4 framework singletons into dist/_deck/ — exactly what
# `deck build --bundle_framework` should do for an island-free app (today it
# `bail`s without a built island dir; see README finding F-static). Everything
# else is deck's real `deck build` output.
#
# Usage: crates/deck/deck-conformance/e2e/static.sh  (needs `cargo build -p deck-cli`)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SPA="$ROOT/crates/deck/deck-conformance/spa"
ASSETS="$ROOT/crates/deck/deck-core/assets"
DECK="$ROOT/target/debug/deck"
SERVER="$ROOT/crates/deck/deck-conformance/e2e/static-server.py"
WORK="$(mktemp -d)"
DIST="$WORK/dist"
PORT=7901
URL="http://127.0.0.1:$PORT/"
PW="playwright-cli -s=deck-conformance-static"

SRV_PID=""
cleanup() {
  # SIGTERM can be slow to reap python's serve_forever; force it so the harness
  # never leaks a static server (a reliability defect in a reliability harness).
  if [ -n "$SRV_PID" ]; then
    kill "$SRV_PID" 2>/dev/null || true
    sleep 0.3
    kill -9 "$SRV_PID" 2>/dev/null || true
  fi
  $PW close >/dev/null 2>&1 || true
  rm -rf "$WORK" 2>/dev/null || true
  true
}
trap cleanup EXIT
fail() { echo "STATIC E2E FAIL: $*" >&2; exit 1; }
read_count() {
  $PW --raw eval "(() => document.querySelector('#root strong')?.textContent?.trim() ?? '')()" 2>/dev/null | tr -d '"' || true
}

[ -x "$DECK" ] || fail "build the deck CLI: cargo build -p deck-cli"

# 1. deck build — SPA mode (framework left external via the importmap).
"$DECK" build --shell-root "$SPA" --out "$DIST" --no-minify >/dev/null || fail "deck build failed"
[ -f "$DIST/index.html" ] || fail "no dist/index.html"
echo "ok: deck build → $DIST"

# 2. Stage the framework singletons a daemon would otherwise serve at /_deck/*.
mkdir -p "$DIST/_deck"
cp "$ASSETS/sail-runtime.js"    "$DIST/_deck/runtime.js"
cp "$ASSETS/sail-dom.js"        "$DIST/_deck/dom.js"
cp "$ASSETS/sail-sdk-client.js" "$DIST/_deck/sdk-client.js"
cp "$ASSETS/sail-components.js" "$DIST/_deck/components.js"

# 3. Serve statically — NO daemon.
python3 "$SERVER" "$PORT" "$DIST" &
SRV_PID=$!
ready=""
for _ in $(seq 1 50); do
  if curl -sf "$URL" >/dev/null 2>&1; then ready=1; break; fi
  sleep 0.2
done
[ -n "$ready" ] || fail "static server did not come up"

# 4. Browser: the SPA renders the counter client-side (no daemon, no SSR).
$PW open >/dev/null
$PW goto "$URL" >/dev/null
rendered=""
for _ in $(seq 1 50); do
  [ "$(read_count)" = "0" ] && { rendered=1; break; }
  sleep 0.2
done
[ -n "$rendered" ] || fail "SPA did not render the counter (last: '$(read_count)')"
echo "ok: static SPA rendered count 0 (no daemon)"

# 5. Click increments — reactivity in the static deploy.
$PW click "css=#root button" >/dev/null
inc=""
for _ in $(seq 1 30); do
  [ "$(read_count)" = "1" ] && { inc=1; break; }
  sleep 0.2
done
[ -n "$inc" ] || fail "click did not increment (last: '$(read_count)')"
echo "ok: click incremented to 1 (reactivity, no daemon)"

echo "STATIC E2E PASS: deck build → static serve (no daemon) → render → react"
