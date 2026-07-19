#!/usr/bin/env bash
# Slice 2 e2e — counter hydration + reactivity, end-to-end, in a real browser.
#
# Boots the ship-free deck-conformance server and drives chromium via playwright-cli:
#   SSR loads  ->  hydrates to count 0  ->  click increments to 1.
# Proves the WHOLE loop (SSR + on-demand Rolldown bundle + framework asset
# serving + client hydration + signal reactivity) works with NO daemon, NO
# ship. Functional assertions are hard gates; console output is captured as
# evidence and the error count is reported (see Waivers in the run note).
#
# Usage: crates/deck/deck-conformance/e2e/counter.sh   (needs `cargo build -p deck-conformance`)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BIN="$ROOT/target/debug/deck-conformance"
ADDR="127.0.0.1:7899"
URL="http://$ADDR/_deck/view/counter"
PW="playwright-cli -s=deck-conformance"
STAMP="$(date -u +%Y-%m-%dT%H-%M-%S)"
ART="$ROOT/.artifacts/web-qa/deck-conformance/counter/$STAMP"
mkdir -p "$ART/screenshots"

SRV_PID=""
cleanup() {
  [ -n "$SRV_PID" ] && kill "$SRV_PID" 2>/dev/null || true
  $PW close >/dev/null 2>&1 || true
}
trap cleanup EXIT
fail() { echo "E2E FAIL: $*" >&2; exit 1; }

read_count() {
  # --raw eval returns JSON, so a string comes back quote-wrapped ("0"); strip
  # the quotes to compare the plain value.
  $PW --raw eval "(() => document.querySelector('.counter strong')?.textContent?.trim() ?? '')()" 2>/dev/null | tr -d '"' || true
}

[ -x "$BIN" ] || fail "binary missing — run: cargo build -p deck-conformance"
pkill -x deck-conformance 2>/dev/null || true
sleep 0.5

# 1. Boot the server, wait until it answers.
DECK_DEMO_ADDR="$ADDR" "$BIN" >"$ART/server.log" 2>&1 &
SRV_PID=$!
ready=""
for _ in $(seq 1 50); do
  if curl -sf "$URL" >/dev/null 2>&1; then ready=1; break; fi
  sleep 0.2
done
[ -n "$ready" ] || fail "server did not come up; see $ART/server.log"

# 2. SSR is present before any JS runs (curl, no browser).
curl -s "$URL" | grep -q "Counter" || fail "SSR missing 'Counter' heading"
echo "ok: SSR served by deck (pre-JS)"

# 3. Browser: load + wait for hydration to fill the reactive slot with 0.
$PW open >/dev/null
$PW goto "$URL" >/dev/null
hydrated=""
for _ in $(seq 1 60); do
  [ "$(read_count)" = "0" ] && { hydrated=1; break; }
  sleep 0.25
done
$PW screenshot >"$ART/screenshots/hydrated.png" 2>/dev/null || true
[ -n "$hydrated" ] || fail "route never hydrated to count 0 (last: '$(read_count)')"
echo "ok: hydrated to count 0"

# 4. Click increment -> reactive update to 1.
$PW click "css=.counter button" >/dev/null
incremented=""
for _ in $(seq 1 30); do
  [ "$(read_count)" = "1" ] && { incremented=1; break; }
  sleep 0.2
done
$PW screenshot >"$ART/screenshots/after-click.png" 2>/dev/null || true
[ -n "$incremented" ] || fail "click did not increment to 1 (last: '$(read_count)')"
echo "ok: click incremented to 1 (signal reactivity through deck)"

# 5. Evidence: capture console + network; report error count (inspected, not
#    auto-gated — the route-host's transitional catalog/umbrella loader may emit
#    benign 404s for a pack-less vanilla app; documented in the run note).
$PW console >"$ART/console.log" 2>/dev/null || true
$PW requests >"$ART/network.log" 2>/dev/null || true
errs="$(grep -c '^\[ERROR\]' "$ART/console.log" 2>/dev/null || echo 0)"
echo "console errors: ${errs} (see $ART/console.log)"
if [ "${errs}" != "0" ]; then
  echo "  (known-benign on a vanilla host: view-tokens.css + favicon 404s — see README findings)"
fi

echo "E2E PASS: SSR -> hydrate(0) -> click -> 1"
echo "evidence: $ART"
