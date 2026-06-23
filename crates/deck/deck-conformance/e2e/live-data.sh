#!/usr/bin/env bash
# Slice 7 e2e — live-clock view, browser driven via playwright-cli.
#
# Boots the ship-free deck-conformance server, loads /_deck/view/clock in a browser,
# reads the tick value, waits ~1.5 s, reads again, asserts it advanced.
# Proves the full loop: SSR -> hydrate -> SSE connect -> live tick -> DOM update.
#
# Requirements: `cargo build -p deck-conformance` + `playwright-cli` in PATH.
# If playwright-cli is unavailable this script exits 0 with a skip notice.
#
# Usage: crates/deck/deck-conformance/e2e/live-data.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BIN="$ROOT/target/debug/deck-conformance"
ADDR="127.0.0.1:7891"
URL="http://$ADDR/_deck/view/clock"
PW="playwright-cli -s=deck-conformance-live"
STAMP="$(date -u +%Y-%m-%dT%H-%M-%S)"
ART="$ROOT/.artifacts/web-qa/deck-conformance/live-data/$STAMP"
mkdir -p "$ART/screenshots"

SRV_PID=""
cleanup() {
  [ -n "$SRV_PID" ] && kill "$SRV_PID" 2>/dev/null || true
  $PW close >/dev/null 2>&1 || true
}
trap cleanup EXIT
fail() { echo "E2E FAIL: $*" >&2; exit 1; }

# Skip gracefully if playwright-cli is not available or cannot launch a browser.
if ! command -v playwright-cli >/dev/null 2>&1; then
  echo "SKIP: playwright-cli not found — live-data e2e requires a browser."
  echo "  Rust no-browser gate (tests/live_data.rs) covers this path."
  exit 0
fi
# Probe browser launch (e.g. WSL2 without a display will fail here).
if ! playwright-cli -s=deck-conformance-live-probe open 2>/dev/null; then
  playwright-cli -s=deck-conformance-live-probe close 2>/dev/null || true
  echo "SKIP: playwright-cli cannot launch a browser (no display / WSL2)."
  echo "  Rust no-browser gate (tests/live_data.rs) covers this path."
  exit 0
fi
playwright-cli -s=deck-conformance-live-probe close 2>/dev/null || true

[ -x "$BIN" ] || fail "binary missing — run: cargo build -p deck-conformance"
pkill -x deck-conformance 2>/dev/null || true
sleep 0.5

# 1. Boot the server.
DECK_DEMO_ADDR="$ADDR" "$BIN" >"$ART/server.log" 2>&1 &
SRV_PID=$!
ready=""
for _ in $(seq 1 50); do
  if curl -sf "$URL" >/dev/null 2>&1; then ready=1; break; fi
  sleep 0.2
done
[ -n "$ready" ] || fail "server did not come up; see $ART/server.log"

# 2. Verify the SSE endpoint returns an event-stream immediately (no browser needed).
SSE_CT=$(curl -s -o /dev/null -w '%{content_type}' \
  -H 'Accept: text/event-stream' \
  "http://$ADDR/_deck/views/clock/sse")
echo "$SSE_CT" | grep -q "text/event-stream" \
  || fail "SSE endpoint wrong Content-Type: $SSE_CT"
echo "ok: SSE endpoint Content-Type = text/event-stream"

# 3. SSR is present (pre-JS curl).
curl -s "$URL" | grep -q "Live Clock" || fail "SSR missing 'Live Clock' heading"
echo "ok: SSR served (pre-JS)"

read_tick() {
  $PW --raw eval "(() => {
    const el = document.querySelector('.tick strong');
    return el ? Number(el.textContent.trim()) : null;
  })()" 2>/dev/null | tr -d '"' || true
}

# 4. Browser: load + hydrate.
$PW open >/dev/null
$PW goto "$URL" >/dev/null
hydrated=""
tick0=""
for _ in $(seq 1 60); do
  v="$(read_tick)"
  if [ -n "$v" ] && [ "$v" != "null" ]; then
    hydrated=1; tick0="$v"; break
  fi
  sleep 0.25
done
$PW screenshot >"$ART/screenshots/t0.png" 2>/dev/null || true
[ -n "$hydrated" ] || fail "clock view never hydrated (no tick visible)"
echo "ok: hydrated with tick=$tick0"

# 5. Wait ~1.5 s and read again; tick must have advanced.
sleep 1.5
tick1="$(read_tick)"
$PW screenshot >"$ART/screenshots/t1.png" 2>/dev/null || true
[ -n "$tick1" ] && [ "$tick1" != "null" ] \
  || fail "tick not readable after wait"
[ "$tick1" -gt "$tick0" ] 2>/dev/null \
  || fail "tick did not advance: $tick0 -> $tick1"
echo "ok: tick advanced $tick0 -> $tick1 (live SSE update)"

$PW console >"$ART/console.log" 2>/dev/null || true
$PW requests >"$ART/network.log" 2>/dev/null || true

echo "E2E PASS: SSR -> hydrate(tick=$tick0) -> live -> tick=$tick1"
echo "evidence: $ART"
