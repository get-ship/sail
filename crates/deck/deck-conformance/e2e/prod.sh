#!/usr/bin/env bash
# Slice 7 e2e — production mode: immutable Cache-Control + ETag, curl-only.
#
# Boots the ship-free deck-conformance server with DECK_DEMO_MODE=prod and verifies:
#   1. GET /_deck/runtime.js → Cache-Control contains "immutable" + max-age
#   2. GET /_deck/runtime.js → ETag header is present
#
# No browser. No ~/.ship. curl-only.
# Usage: crates/deck/deck-conformance/e2e/prod.sh   (needs `cargo build -p deck-conformance`)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BIN="$ROOT/target/debug/deck-conformance"
ADDR="127.0.0.1:7895"
BASE="http://$ADDR"

SRV_PID=""
cleanup() { [ -n "$SRV_PID" ] && kill "$SRV_PID" 2>/dev/null || true; }
trap cleanup EXIT
fail() { echo "E2E FAIL: $*" >&2; exit 1; }

[ -x "$BIN" ] || fail "binary missing — run: cargo build -p deck-conformance"
pkill -x deck-conformance 2>/dev/null || true

# 1. Boot the server in production mode.
DECK_DEMO_ADDR="$ADDR" DECK_DEMO_MODE=prod "$BIN" >/dev/null 2>&1 &
SRV_PID=$!
ready=""
for _ in $(seq 1 50); do
  if curl -sf "$BASE/_deck/runtime.js" >/dev/null 2>&1; then ready=1; break; fi
  sleep 0.2
done
[ -n "$ready" ] || fail "server did not come up"

# 2. Fetch runtime.js headers.
HEADERS=$(curl -sI "$BASE/_deck/runtime.js")

# 3. Cache-Control must contain "immutable" and "max-age".
echo "$HEADERS" | grep -i 'cache-control' | grep -qi 'immutable' \
  || fail "Cache-Control must contain 'immutable'; got: $(echo "$HEADERS" | grep -i cache-control)"
echo "$HEADERS" | grep -i 'cache-control' | grep -qi 'max-age' \
  || fail "Cache-Control must contain 'max-age'; got: $(echo "$HEADERS" | grep -i cache-control)"
echo "ok: Cache-Control is immutable + max-age"

# 4. ETag header must be present.
echo "$HEADERS" | grep -qi '^etag:' \
  || fail "ETag header must be present in production mode"
ETAG=$(echo "$HEADERS" | grep -i '^etag:' | head -1 | tr -d '\r')
echo "ok: ETag present: $ETAG"

echo "E2E PASS: prod mode → immutable Cache-Control + ETag verified (ship-free)"
