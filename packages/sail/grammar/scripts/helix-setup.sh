#!/usr/bin/env bash
# Overlay this checkout's queries + grammar into Helix. Re-run after generate.
#
# Never replace ~/.config/helix/runtime. If that path is a symlink to the
# Helix install (common), write into it. Creating a new empty runtime dir
# hides stock themes/grammars.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
grammar=$(cd "$here/.." && pwd)
repo=$(cd "$grammar/../../.." && pwd)
helix_cfg="${HELIX_CONFIG:-$HOME/.config/helix}"
runtime="$helix_cfg/runtime"

if [ -L "$runtime" ]; then
  runtime=$(readlink -f "$runtime")
  echo "using Helix runtime symlink target $runtime"
elif [ ! -d "$runtime" ]; then
  # Do not mkdir a blank overlay — Helix would treat it as the whole runtime.
  if [ -d "${HELIX_RUNTIME:-}" ]; then
    runtime="$HELIX_RUNTIME"
  elif [ -d "$(dirname "$(command -v hx)")/../lib/helix" ]; then
    runtime=$(cd "$(dirname "$(command -v hx)")/../lib/helix" && pwd)
  else
    echo "no Helix runtime found (no $helix_cfg/runtime, no HELIX_RUNTIME)"
    exit 1
  fi
  echo "using Helix install runtime $runtime"
fi

mkdir -p "$runtime/queries/sail" "$runtime/grammars"
ln -sfn "$grammar/queries/highlights.scm" "$runtime/queries/sail/highlights.scm"
ln -sfn "$grammar/queries/injections.scm" "$runtime/queries/sail/injections.scm"
echo "linked queries → $runtime/queries/sail"

langs="$helix_cfg/languages.toml"
marker="# sail-grammar (m-sail-public helix-setup)"
block=$(
  cat <<EOF

$marker
[[grammar]]
name = "sail"
source = { path = "$grammar" }
EOF
)
if [ -f "$langs" ] && grep -qF "$marker" "$langs"; then
  echo "sail grammar already in $langs"
elif [ -f "$langs" ]; then
  printf '%s\n' "$block" >>"$langs"
  echo "appended sail grammar to $langs"
else
  printf '%s\n' "$block" >"$langs"
  echo "wrote $langs"
fi

if ! command -v hx >/dev/null 2>&1; then
  echo "hx not on PATH. Then: hx --grammar build"
  exit 0
fi

hx --grammar build >/tmp/helix-grammar-build.log 2>&1 || true
if [ -f "$runtime/grammars/sail.so" ]; then
  echo "built $runtime/grammars/sail.so"
  echo "done. Open a .sail from $repo (sail on PATH). Theme is Helix's, not this script."
  exit 0
fi
echo "sail.so missing — hx --grammar build log:"
tail -30 /tmp/helix-grammar-build.log
exit 1
