#!/usr/bin/env bash
# Slice 8 e2e — todos mutation, browser driven via playwright-cli.
#
# Boots the ship-free deck-conformance server, loads /_deck/view/todos in a browser,
# fills in the "New todo…" input, submits the form, follows the 303 redirect,
# and asserts the new item appears in the rendered list.
# Proves the full loop: SSR -> hydrate -> form submit -> progressive-enhancement mutation -> 303
# -> live SSE update -> DOM shows new item.
#
# Requirements: `cargo build -p deck-conformance` + `playwright-cli` in PATH.
# If playwright-cli is unavailable this script exits 0 with a skip notice.
#
# Usage: crates/deck/deck-conformance/e2e/mutation.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BIN="$ROOT/target/debug/deck-conformance"
ADDR="127.0.0.1:7892"
URL="http://$ADDR/_deck/view/todos"
PW="playwright-cli -s=deck-conformance-mutation"
STAMP="$(date -u +%Y-%m-%dT%H-%M-%S)"
ART="$ROOT/.artifacts/web-qa/deck-conformance/mutation/$STAMP"
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
  echo "SKIP: playwright-cli not found — mutation e2e requires a browser."
  echo "  Rust no-browser gate (tests/mutation.rs) covers this path."
  exit 0
fi
# Probe browser launch (e.g. WSL2 without a display will fail here).
if ! playwright-cli -s=deck-conformance-mut-probe open 2>/dev/null; then
  playwright-cli -s=deck-conformance-mut-probe close 2>/dev/null || true
  echo "SKIP: playwright-cli cannot launch a browser (no display / WSL2)."
  echo "  Rust no-browser gate (tests/mutation.rs) covers this path."
  exit 0
fi
playwright-cli -s=deck-conformance-mut-probe close 2>/dev/null || true

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

# 2. SSR is present (pre-JS curl).
curl -s "$URL" | grep -q "Todos" || fail "SSR missing 'Todos' heading"
echo "ok: SSR served (pre-JS)"

# 3. Browser: load + hydrate.
$PW open >/dev/null
$PW goto "$URL" >/dev/null
hydrated=""
for _ in $(seq 1 60); do
  v="$($PW --raw eval "(() => !!document.querySelector('.todos-demo form'))()" 2>/dev/null | tr -d '"')"
  [ "$v" = "true" ] && { hydrated=1; break; }
  sleep 0.25
done
$PW screenshot >"$ART/screenshots/before-submit.png" 2>/dev/null || true
[ -n "$hydrated" ] || fail "todos route never hydrated (form not found)"
echo "ok: todos route loaded"

# 4. Fill in the input and submit the form.
TODO_TEXT="e2e-todo-$(date +%s)"
$PW fill "css=.todos-demo input[name=text]" "$TODO_TEXT" >/dev/null
$PW click "css=.todos-demo button[type=submit]" >/dev/null

# 5. After 303 redirect + live SSE update, the new item must appear.
appeared=""
for _ in $(seq 1 60); do
  items="$($PW --raw eval "(() => {
    const items = Array.from(document.querySelectorAll('.todos-demo li'));
    return items.map(i => i.textContent.trim()).join('||');
  })()" 2>/dev/null | tr -d '"')"
  echo "$items" | grep -qF "$TODO_TEXT" && { appeared=1; break; }
  sleep 0.25
done
$PW screenshot >"$ART/screenshots/after-submit.png" 2>/dev/null || true
[ -n "$appeared" ] || fail "new todo '$TODO_TEXT' never appeared in the list"
echo "ok: '$TODO_TEXT' appears in todos after form submit"

$PW console >"$ART/console.log" 2>/dev/null || true
$PW requests >"$ART/network.log" 2>/dev/null || true

echo "E2E PASS: SSR -> hydrate -> submit '$TODO_TEXT' -> appears in list"
echo "evidence: $ART"
