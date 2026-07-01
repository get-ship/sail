#!/usr/bin/env bash
# lane-watch.sh — peer into each lane's tmux session and detect when the
# agent has gone idle, WITHOUT depending on the agent committing.
#
# Why: waiting on `git commit` is fragile — agents often finish the work but
# never commit, and a commit-based watcher then reads "no work done" and the
# cohort gets cleaned up. The tmux pane is ground truth: when Claude Code stops
# printing and its "esc to interrupt" working indicator is gone, the agent is
# done (or waiting on us). We watch the pane, not the branch.
#
# dispatch-eval.sh names each lane's tmux session after its slug
# (`eval-<model>-<RUN_ID>`), so we find the cohort by RUN_ID. When every lane is
# idle (or the global timeout hits), we grade + record automatically — one
# foreground command, no /loop, no commit race.
#
# Usage:
#   bash eval/scripts/lane-watch.sh <RUN_ID>
#   bash eval/scripts/lane-watch.sh <RUN_ID> --no-grade   # just signal

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
RUN_ID="${1:?usage: lane-watch.sh <RUN_ID> [--no-grade]}"
shift || true
DO_GRADE=1
[[ "${1:-}" == "--no-grade" ]] && DO_GRADE=0

# Resolve a lane's worktree path from git itself — robust against whatever base
# dir dispatch.sh used (~/dev/ship-worktrees vs ~/.ship/worktrees differ per
# machine). git knows where every worktree lives; we just match by slug.
wt_path() {
  git -C "$REPO_ROOT" worktree list --porcelain 2>/dev/null \
    | awk '/^worktree /{print $2}' | grep -E "/$1$" | head -1
}

POLL_SECS=30          # how often to sample each pane
QUIET_POLLS=4         # consecutive unchanged + no-spinner samples → idle (120s)
MAX_PRODS=2           # times to nudge an idle-but-uncommitted lane to commit
TIMEOUT_SECS=2400     # global cap (40m) — never block forever
ACTIVE_MARKER='esc to interrupt'  # Claude Code's working indicator

# Nudge sent to an idle lane that has uncommitted work — the exact case that
# burns us downstream (idle agent, real work on disk, no commit → cleanup
# assumes nothing happened). send-keys it into the pane and give it another
# round to comply before we grade the dirty tree anyway.
PROD_MSG='Your work looks complete but is not committed. Stage and commit it now: git add -A && git commit. Then stop.'

# Find this run's lane sessions. dispatch-eval.sh sessions are `eval-*-<RUN_ID>`.
mapfile -t SESSIONS < <(tmux list-sessions -F '#{session_name}' 2>/dev/null \
  | grep -E "^eval-.*-${RUN_ID}$" || true)

if [[ ${#SESSIONS[@]} -eq 0 ]]; then
  echo "lane-watch: no tmux sessions match eval-*-${RUN_ID}" >&2
  echo "lane-watch: (lanes still booting? or RUN_ID wrong?)" >&2
  exit 1
fi

echo "lane-watch: RUN_ID=$RUN_ID watching ${#SESSIONS[@]} lane(s): ${SESSIONS[*]}"

declare -A STABLE PREV DONE PRODS
for s in "${SESSIONS[@]}"; do STABLE[$s]=0; PREV[$s]=""; DONE[$s]=0; PRODS[$s]=0; done

SENTINEL_DIR="$REPO_ROOT/.artifacts/sui-eval-runs/.watch/$RUN_ID"
mkdir -p "$SENTINEL_DIR"

# A lane worktree has uncommitted work if `git status --porcelain` is non-empty.
has_uncommitted() {
  local wt; wt="$(wt_path "$1")"
  [[ -n "$wt" && -d "$wt" ]] && [[ -n "$(git -C "$wt" status --porcelain 2>/dev/null)" ]]
}

start=$SECONDS
while :; do
  all_done=1
  for s in "${SESSIONS[@]}"; do
    [[ ${DONE[$s]} -eq 1 ]] && continue

    # Session gone = agent process exited = done.
    if ! tmux has-session -t "$s" 2>/dev/null; then
      echo "lane-watch: $s — session ended (agent exited)"
      DONE[$s]=1; touch "$SENTINEL_DIR/$s.done"; continue
    fi

    pane="$(tmux capture-pane -t "$s" -p 2>/dev/null || true)"
    hash="$(printf '%s' "$pane" | cksum | cut -d' ' -f1)"
    active=0
    grep -qF "$ACTIVE_MARKER" <<<"$pane" && active=1

    if [[ $active -eq 1 || "$hash" != "${PREV[$s]}" ]]; then
      STABLE[$s]=0
    else
      STABLE[$s]=$(( STABLE[$s] + 1 ))
    fi
    PREV[$s]="$hash"

    if [[ ${STABLE[$s]} -lt $QUIET_POLLS ]]; then
      all_done=0; continue
    fi

    # Idle. If there's uncommitted work, prod it to commit and give it another
    # round — don't let a finished-but-uncommitted lane look like "did nothing".
    if has_uncommitted "$s"; then
      if [[ ${PRODS[$s]} -lt $MAX_PRODS ]]; then
        PRODS[$s]=$(( PRODS[$s] + 1 ))
        echo "lane-watch: $s — idle with uncommitted work, prodding to commit (${PRODS[$s]}/$MAX_PRODS)"
        tmux send-keys -t "$s" "$PROD_MSG" && tmux send-keys -t "$s" Enter
        STABLE[$s]=0; all_done=0; continue
      fi
      echo "lane-watch: $s — still uncommitted after $MAX_PRODS prods; grading dirty tree" >&2
    fi

    echo "lane-watch: $s — idle (${QUIET_POLLS}×${POLL_SECS}s quiet, no spinner)"
    DONE[$s]=1; touch "$SENTINEL_DIR/$s.done"
  done

  [[ $all_done -eq 1 ]] && break
  if (( SECONDS - start > TIMEOUT_SECS )); then
    echo "lane-watch: TIMEOUT after ${TIMEOUT_SECS}s — grading what's there" >&2
    break
  fi
  sleep "$POLL_SECS"
done

echo "lane-watch: all lanes settled."
if [[ $DO_GRADE -eq 0 ]]; then
  echo "lane-watch: --no-grade — sentinels in $SENTINEL_DIR"
  exit 0
fi

# grade.ts discovers every lane worktree itself (via git worktree list) and
# writes one jsonl to the main worktree's .artifacts — so run it ONCE from the
# main repo, not per-lane. Then record once.
echo "lane-watch: grading…"
( cd "$REPO_ROOT" && SHIP_PIPELINE_RUN_ID="$RUN_ID" bun "$REPO_ROOT/eval/scripts/grade.ts" )
( cd "$REPO_ROOT" && SHIP_PIPELINE_RUN_ID="$RUN_ID" bun "$REPO_ROOT/eval/scripts/record.ts" )
echo "lane-watch: done — scores in views/eval + .artifacts/sui-eval-runs/$RUN_ID.jsonl"
