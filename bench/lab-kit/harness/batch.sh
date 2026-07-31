#!/usr/bin/env bash
# Full baseline batch: 5 tasks × 2 arms × n=3 sequential.
# Usage: batch.sh [--tasks a,b] [--arms sail,react-tw] [--n 3] [--dry-run]
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

TASKS="adjustable-shell,parallax-landing,dependent-form,brand-transplant,mini-app-routing"
ARMS="sail,react-tw"
N=3
DRY=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tasks) TASKS="$2"; shift 2 ;;
    --arms) ARMS="$2"; shift 2 ;;
    --n) N="$2"; shift 2 ;;
    --dry-run) DRY=1; shift ;;
    -h|--help)
      echo "usage: batch.sh [--tasks a,b] [--arms sail,react-tw] [--n 3] [--dry-run]"
      exit 0
      ;;
    *) die "unknown arg: $1" ;;
  esac
done

IFS=',' read -ra TASK_ARR <<< "$TASKS"
IFS=',' read -ra ARM_ARR <<< "$ARMS"

PLAN=()
for t in "${TASK_ARR[@]}"; do
  for a in "${ARM_ARR[@]}"; do
    for ((i=1; i<=N; i++)); do
      PLAN+=("$t $a $i")
    done
  done
done

echo "tasks: $TASKS"
echo "arms: $ARMS"
echo "n: $N"
echo "planned runs: ${#PLAN[@]}"

if [[ "$DRY" -eq 1 ]]; then
  # Pure plan — no sync, toolchain, mkdir, or side effects
  printf '%s\n' "${PLAN[@]}"
  exit 0
fi

"$SCRIPT_DIR/sync.sh"
if [[ ! -d "$LAB/toolchains" ]] || [[ -z "$(ls -A "$LAB/toolchains" 2>/dev/null || true)" ]]; then
  "$SCRIPT_DIR/build_toolchain.sh"
fi
STAMP="$(latest_toolchain)"

if [[ "$ARMS" == *react-tw* ]]; then
  if [[ ! -d "$LAB/payload/react-base/node_modules" ]]; then
    "$SCRIPT_DIR/prepare_react_base.sh"
  fi
fi

BATCH_TS="$(date +%Y%m%d-%H%M%S)"
BATCH_DIR="$LAB/reports/${BATCH_TS}-baseline"
mkdir -p "$BATCH_DIR"
export SAIL_LAB_BATCH_DIR="$BATCH_DIR"

echo "batch: $BATCH_DIR"
echo "toolchain: $STAMP"

: > "$BATCH_DIR/run_dirs.txt"
FAIL=0
for item in "${PLAN[@]}"; do
  # shellcheck disable=SC2086
  set -- $item
  t="$1"; a="$2"; i="$3"
  echo ""
  echo "######## $t / $a / n=$i ########"
  if "$SCRIPT_DIR/run_one.sh" "$t" "$a" "$i" "$STAMP"; then
    RD="$(ls -1dt "$LAB/runs/"*-"${t}"-"${a}"-"${i}" 2>/dev/null | head -1 || true)"
    if [[ -n "$RD" ]]; then
      echo "$RD" >> "$BATCH_DIR/run_dirs.txt"
    fi
  else
    echo "RUN FAILED: $t $a $i" | tee -a "$BATCH_DIR/failures.txt"
    FAIL=$((FAIL + 1))
  fi
done

python3 "$SCRIPT_DIR/report_batch.py" "$BATCH_DIR"
assert_repo_clean_of_lab_artifacts

echo ""
echo "batch complete: $BATCH_DIR (failures=$FAIL)"
exit "$FAIL"
