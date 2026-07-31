#!/usr/bin/env bash
# Smoke: dependent-form × sail × n=1 end-to-end instrumentation check.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

"$SCRIPT_DIR/sync.sh"
if [[ ! -d "$LAB/toolchains" ]] || [[ -z "$(ls -A "$LAB/toolchains" 2>/dev/null || true)" ]]; then
  "$SCRIPT_DIR/build_toolchain.sh"
fi

BATCH_TS="$(date +%Y%m%d-%H%M%S)"
BATCH_DIR="$LAB/reports/${BATCH_TS}-smoke"
mkdir -p "$BATCH_DIR"
export SAIL_LAB_BATCH_DIR="$BATCH_DIR"

echo "== smoke: dependent-form / sail / n=1 =="
"$SCRIPT_DIR/run_one.sh" dependent-form sail 1

RD="$(ls -1dt "$LAB/runs/"*-dependent-form-sail-1 2>/dev/null | head -1 || true)"
if [[ -n "$RD" ]]; then
  echo "$RD" > "$BATCH_DIR/run_dirs.txt"
  # Verify harvest artifacts
  for f in .lab-harvest.json .lab-meta.json; do
    [[ -f "$RD/$f" ]] || die "smoke missing $RD/$f"
  done
  if [[ ! -f "$RD/.probe-events.jsonl" ]]; then
    echo "WARN: no events jsonl (hooks may not have fired)" | tee -a "$BATCH_DIR/flags.txt"
  fi
fi

python3 "$SCRIPT_DIR/report_batch.py" "$BATCH_DIR"
assert_repo_clean_of_lab_artifacts

# Require live instrumentation for smoke to count
if [[ -n "${RD:-}" ]]; then
  if [[ ! -s "$RD/.probe-events.jsonl" ]]; then
    die "smoke failed: empty/missing .probe-events.jsonl (hooks did not fire)"
  fi
  EV_N="$(wc -l < "$RD/.probe-events.jsonl" | tr -d ' ')"
  echo "events lines: $EV_N"
  if [[ "$EV_N" -lt 2 ]]; then
    die "smoke failed: too few hook events ($EV_N)"
  fi
  # one-pager should exist under batch dir
  ONE="$(ls -1 "$BATCH_DIR"/*-dependent-form-sail-1.md 2>/dev/null | head -1 || true)"
  [[ -n "$ONE" ]] || die "smoke failed: missing one-pager in $BATCH_DIR"
fi

echo "smoke ok → $BATCH_DIR"
