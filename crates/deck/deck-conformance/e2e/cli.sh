#!/usr/bin/env bash
# GAP 8 — deck CLI coverage (ship-free).
#
# Exercises the deck CLI against the demo app and asserts each subcommand:
#   1. `deck routes --shell-root <spa>` → exit 0, prints route count
#   2. `deck build --shell-root <spa>` → exit 0, produces dist/
#   3. `deck build-assets` → exit 0, framework bundles present
#   4. `deck scaffold view <tmpname>` → exit 0, emits page.sui
#
# Commands requiring a running daemon are skipped with a note.
# This script runs ship-free: no shipd, no SHIP_HOME, no auth.
#
# Usage: crates/deck/deck-conformance/e2e/cli.sh
#   Requires: cargo build -p deck-cli
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
DEMO="$ROOT/crates/deck/deck-conformance"
SPA="$DEMO/spa"
DECK="$ROOT/target/debug/deck"
ASSETS="$ROOT/crates/deck/deck-core/assets"
WORK="$(mktemp -d)"

cleanup() { rm -rf "$WORK" 2>/dev/null || true; }
trap cleanup EXIT
fail() { echo "CLI E2E FAIL: $*" >&2; exit 1; }
ok()   { echo "ok: $*"; }

if [ ! -x "$DECK" ]; then
  echo "building deck CLI..."
  cargo build -p deck-cli --quiet || fail "cargo build -p deck-cli failed"
fi

# ── 1. routes ─────────────────────────────────────────────────────────────────
# The demo spa/ has no src/studio → 0 dynamic routes, exit 0.
echo "--- deck routes ---"
OUT=$("$DECK" routes --shell-root "$SPA" 2>&1) \
  || fail "deck routes exited non-zero: $OUT"
echo "$OUT" | grep -q "deck routes" || fail "deck routes: unexpected output: $OUT"
ok "deck routes --shell-root spa/ → exit 0: $OUT"

# ── 2. deck build ─────────────────────────────────────────────────────────────
# Build the SPA entry to dist/. Framework stays external (importmap).
echo "--- deck build ---"
DIST="$WORK/dist"
OUT=$("$DECK" build --shell-root "$SPA" --out "$DIST" --no-minify 2>&1) \
  || fail "deck build exited non-zero: $OUT"
[ -f "$DIST/index.html" ] || fail "deck build: missing dist/index.html"
[ -f "$DIST/index.html" ] && grep -q "importmap" "$DIST/index.html" \
  || fail "deck build: index.html missing importmap"
ok "deck build --shell-root spa/ → dist/ with index.html + importmap"

# Hashed entry chunk must exist.
JS_CHUNK=$(find "$DIST/assets" -name "main-*.js" 2>/dev/null | head -1 || true)
[ -n "$JS_CHUNK" ] || fail "deck build: no hashed main-*.js in dist/assets/"
ok "deck build: hashed entry chunk $(basename "$JS_CHUNK")"

# ── 3. build-assets ───────────────────────────────────────────────────────────
# Bakes the framework singletons. Must exit 0 and produce non-empty bundles.
echo "--- deck build-assets ---"
cd "$ROOT"
OUT=$("$DECK" build-assets 2>&1) \
  || fail "deck build-assets exited non-zero: $OUT"
cd "$DEMO"

# Verify the four framework bundles are non-empty.
for f in sui-runtime.js sui-dom.js sui-sdk-client.js sui-components.js; do
  [ -s "$ASSETS/$f" ] || fail "deck build-assets: $f missing or empty"
  ok "framework bundle: $f ($(wc -c < "$ASSETS/$f") bytes)"
done
ok "deck build-assets → exit 0, all framework bundles non-empty"

# ── 4. scaffold view ──────────────────────────────────────────────────────────
# `deck scaffold view` locates the Cargo workspace root from cwd and writes
# views/<name>/ there. Run from the deck-conformance directory so the workspace is
# found, then clean up the scaffolded view. The name must not conflict with an
# existing view; use a timestamp-suffixed name to be safe.
echo "--- deck scaffold view ---"
SCAFFOLD_NAME="cli-test-$$"
cd "$ROOT"
OUT=$("$DECK" scaffold view "$SCAFFOLD_NAME" 2>&1) \
  || fail "deck scaffold view $SCAFFOLD_NAME exited non-zero: $OUT"

PAGE="$ROOT/views/$SCAFFOLD_NAME/page.sui"
MANIFEST_FILE="$ROOT/views/$SCAFFOLD_NAME/manifest.jsonc"
[ -f "$PAGE" ] || fail "deck scaffold view: page.sui not emitted at $PAGE"
grep -q "div\|h1\|<" "$PAGE" || fail "deck scaffold view: page.sui appears empty"
ok "deck scaffold view $SCAFFOLD_NAME → page.sui emitted"

[ -f "$MANIFEST_FILE" ] || fail "deck scaffold view: manifest.jsonc not emitted"
grep -q "$SCAFFOLD_NAME" "$MANIFEST_FILE" || fail "deck scaffold view: manifest.jsonc missing view name"
ok "deck scaffold view $SCAFFOLD_NAME → manifest.jsonc emitted"

# Clean up: remove the scaffolded view (it's in the monorepo root views/ dir).
rm -rf "$ROOT/views/$SCAFFOLD_NAME"
ok "scaffolded view cleaned up"
cd "$DEMO"

# Note: `deck routes --shell-root <demo-app>` on the live app dir (not spa/).
# The demo app has `views/` but no `src/studio/` → 0 routes, exit 0.
echo "--- deck routes (demo app dir) ---"
APP="$DEMO/app"
OUT2=$("$DECK" routes --shell-root "$APP" 2>&1) \
  || fail "deck routes --shell-root app/ exited non-zero: $OUT2"
ok "deck routes --shell-root app/ → exit 0: $OUT2"

echo "CLI E2E PASS"
