#!/usr/bin/env bash
# #164 slice 1 — repeatable real-browser SES smoke. Builds the harness, serves
# it, drives a headless browser via playwright-cli, and asserts SES lockdown +
# confinement + ambient-authority denial all hold in a real engine (which jsdom
# cannot do — see #133 slice 5). Exit 0 = pass.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PORT="${HARNESS_PORT:-4599}"

# Resolve playwright-cli (mirrors the web-qa skill's fallback walk).
if command -v playwright-cli &>/dev/null; then PW="playwright-cli"
elif command -v bunx &>/dev/null; then PW="bunx playwright-cli"
else PW="npx playwright-cli"; fi

bun "$HERE/build.ts"

python3 -m http.server "$PORT" --directory "$HERE/dist" >/tmp/harness-serve.log 2>&1 &
SERVE_PID=$!
trap 'kill "$SERVE_PID" 2>/dev/null || true' EXIT
sleep 1.5

$PW goto "http://127.0.0.1:$PORT/" >/dev/null 2>&1
sleep 2

# Diagnostics (escaped JSON — display only).
echo "ses-smoke: $($PW --raw eval "JSON.stringify(window.__sesSmoke)" 2>/dev/null | tail -1)"

# Assert via a single boolean eval — robust to playwright's string escaping.
PASS="$($PW --raw eval "(() => { const r = window.__sesSmoke || {}; return r.lockdown === 'ok' && r.harden === 'ok' && r.ambientDenied === true; })()" 2>/dev/null | tail -1)"
if [[ "$PASS" != *true* ]]; then
  echo "FAIL: SES gates did not all hold (got: $PASS)" >&2
  exit 1
fi
echo "✓ SES lockdown + confinement + ambient-denial verified in a real browser"

# Slice 2 — a REAL compiled view confines + evaluates in-browser under lockdown.
$PW goto "http://127.0.0.1:$PORT/view-confine.html" >/dev/null 2>&1
sleep 2
echo "view-confine: $($PW --raw eval "JSON.stringify(window.__viewConfine)" 2>/dev/null | tail -1)"
VPASS="$($PW --raw eval "(() => { const r = window.__viewConfine || {}; return r.lockdown === 'ok' && r.confined === 'ok' && r.hasSetup === true && r.ambientProbeDenied === true; })()" 2>/dev/null | tail -1)"
if [[ "$VPASS" != *true* ]]; then
  echo "FAIL: real view did not confine cleanly in-browser (got: $VPASS)" >&2
  exit 1
fi
echo "✓ real compiled view confines to {default:{setup}} in-browser, ambient denied"

# Slice 6 — confinement overhead bench (informational; not a pass/fail gate).
$PW goto "http://127.0.0.1:$PORT/bench.html" >/dev/null 2>&1
sleep 2
echo "bench: $($PW --raw eval "JSON.stringify(window.__bench)" 2>/dev/null | tail -1)"
