#!/usr/bin/env bash
# Sync bench standards from sail repo into the lab.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

SYNC="$REPO/bench/scripts/sync-to-lab.sh"
[[ -x "$SYNC" ]] || chmod +x "$SYNC"
SAIL_LAB="$LAB" "$SYNC"
