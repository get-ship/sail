#!/usr/bin/env bash
# Slice 5 e2e — dynamic param sub-route, end-to-end, curl-only.
#
# Boots the ship-free deck-conformance server and verifies:
#   1. GET /_deck/views/greet/zelda → 200 JSON
#   2. Payload params.id == "zelda"
#   3. Payload html contains "Hello" (static scaffold from the template)
#   4. GET /_deck/view/greet/zelda → 200 HTML (route-host resolves the route)
#
# No browser. No ~/.ship. curl-only.
# Usage: crates/deck/deck-conformance/e2e/param.sh   (needs `cargo build -p deck-conformance`)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BIN="$ROOT/target/debug/deck-conformance"
ADDR="127.0.0.1:7897"
BASE="http://$ADDR"

SRV_PID=""
cleanup() { [ -n "$SRV_PID" ] && kill "$SRV_PID" 2>/dev/null || true; }
trap cleanup EXIT
fail() { echo "E2E FAIL: $*" >&2; exit 1; }

[ -x "$BIN" ] || fail "binary missing — run: cargo build -p deck-conformance"
pkill -x deck-conformance 2>/dev/null || true

# 1. Boot the server.
DECK_DEMO_ADDR="$ADDR" "$BIN" >/dev/null 2>&1 &
SRV_PID=$!
ready=""
for _ in $(seq 1 50); do
  if curl -sf "$BASE/_deck/runtime.js" >/dev/null 2>&1; then ready=1; break; fi
  sleep 0.2
done
[ -n "$ready" ] || fail "server did not come up"

# 2. Payload endpoint resolves greet/zelda → 200 JSON.
PAYLOAD=$(curl -sf "$BASE/_deck/views/greet/zelda") \
  || fail "GET /_deck/views/greet/zelda returned non-2xx"
echo "ok: /_deck/views/greet/zelda → 200 JSON"

# 3. params.id == "zelda".
ID=$(echo "$PAYLOAD" | python3 -c "import sys,json; print(json.load(sys.stdin)['params']['id'])" 2>/dev/null) \
  || fail "could not parse params.id from payload"
[ "$ID" = "zelda" ] || fail "params.id must be 'zelda', got '$ID'"
echo "ok: params.id == \"zelda\""

# 4. SSR html contains "Hello" (static template text).
echo "$PAYLOAD" | python3 -c "import sys,json; html=json.load(sys.stdin)['html']; exit(0 if 'Hello' in html else 1)" \
  || fail "SSR html does not contain 'Hello'"
echo "ok: SSR html contains 'Hello'"

# 5. Route-host page returns 200 for the sub-path route.
curl -sf "$BASE/_deck/view/greet/zelda" >/dev/null \
  || fail "GET /_deck/view/greet/zelda returned non-2xx"
echo "ok: /_deck/view/greet/zelda → 200 HTML"

echo "E2E PASS: param sub-route resolver → capture → SSR verified (ship-free)"
