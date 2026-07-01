#!/usr/bin/env bash
# dispatch-eval.sh — fire a sui-eval run via dispatch.sh instead of the
# daemon's fanout path.
#
# Why: the daemon's fanout dispatch currently leaks badly (#193) and
# loses lane worktrees if the daemon is bounced mid-run. dispatch.sh
# spawns each agent as a one-off in a real PTY (interactive claude,
# NOT `-p --output-format stream-json`), so:
#   - lanes are fully observable (real TUI, not raw JSON)
#   - lanes survive daemon hiccups (no fanout coordinator)
#   - dispatch.sh opens a wt tab per lane automatically
#
# What we give up: the automatic step-done → grade → record chain.
# Score by hand after both lanes finish (or write a wrapper that
# polls for both worktrees to be quiet, then runs grade.ts +
# record.ts against each).
#
# Usage:
#   bash eval/scripts/dispatch-eval.sh
#   bash eval/scripts/dispatch-eval.sh --models haiku,sonnet,opus

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# Default model set. Test/smoke runs default to haiku ONLY — sonnet (and opus)
# are reserved for live graded runs to keep iteration cheap. Pass
# `--models haiku,sonnet[,opus]` explicitly for a live run. Each model becomes
# one lane with its own dispatched worktree.
MODELS_CSV="haiku"
SPEC="eval/task-spec.md"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --models) MODELS_CSV="$2"; shift 2 ;;
        --spec)   SPEC="$2"; shift 2 ;;
        *) echo "unknown arg: $1" >&2; exit 1 ;;
    esac
done

if [[ ! -f "$SPEC" ]]; then
    echo "spec file missing: $SPEC" >&2
    exit 1
fi

# Map short labels to actual model IDs the dispatched agent profile honors.
model_id() {
    case "$1" in
        haiku)  echo "claude-haiku-4-5-20251001" ;;
        sonnet) echo "claude-sonnet-4-6" ;;
        opus)   echo "claude-opus-4-8" ;;
        *) echo "$1" ;;  # pass through anything custom
    esac
}

# Run id is a short timestamp + entropy — same shape as `ship run`
# pipeline_run_id but visibly hand-rolled (we're not going through the
# daemon's run.started). Lanes share this prefix so grade.ts can find
# the cohort.
RUN_ID="$(date -u +%y%m%d%H%M%S)-$(head -c 4 /dev/urandom | xxd -p)"
echo "dispatch-eval: RUN_ID=$RUN_ID"
echo "dispatch-eval: spec    =$SPEC"
echo "dispatch-eval: models  =$MODELS_CSV"
echo

IFS=',' read -ra MODELS <<< "$MODELS_CSV"
SLUGS=()
for label in "${MODELS[@]}"; do
    label="${label// /}"  # trim spaces
    [[ -z "$label" ]] && continue
    slug="eval-${label}-${RUN_ID}"
    SLUGS+=("$slug")
    model="$(model_id "$label")"
    echo "── dispatching $label ($model) → $slug"
    # SHIP_DEFAULT_TERMINAL=wt forces dispatch.sh's wt branch even when
    # invoked from a non-wt shell (e.g. a Claude Code bash session
    # where WT_SESSION isn't propagated). That branch creates a tmux
    # session AND opens a wt tab attached to it via `wt.exe -w 0
    # new-tab` — targeting the focused wt window so all lanes land
    # as tabs in the same window, no copy/paste required.
    SHIP_DEFAULT_TERMINAL=wt \
    SHIP_AGENT_MODEL="$model" \
        bash "$REPO_ROOT/.ship/skills/spawn-agent/scripts/dispatch.sh" \
            --slug "$slug" \
            --agent ship-designer \
            --spec "$SPEC"
    echo
done

echo "════════════════════════════════════════════════════════════════"
echo "RUN_ID:    $RUN_ID"
echo "lanes:     ${#SLUGS[@]}  (${SLUGS[*]})"
echo "wt tabs:   should now be open in your focused wt window"
echo
echo "Score (after both finish) — grade per worktree, then record:"
for slug in "${SLUGS[@]}"; do
    echo "  (cd $HOME/.ship/worktrees/$slug && SHIP_PIPELINE_RUN_ID='$RUN_ID' bun $REPO_ROOT/eval/scripts/grade.ts)"
done
echo "  SHIP_PIPELINE_RUN_ID='$RUN_ID' bun $REPO_ROOT/eval/scripts/record.ts"
echo "════════════════════════════════════════════════════════════════"
