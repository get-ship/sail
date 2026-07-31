#!/usr/bin/env bash
# Ensure payload/react-base has a full vite+react+tailwind tree + bun install.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

require_cmd bun
BASE="$LAB/payload/react-base"
mkdir -p "$BASE/src"

# Prefer cannibalizing sail-probe react-base when present
PROBE_BASE="$HOME/dev/sail-probe/react-base"
if [[ -d "$PROBE_BASE" && -f "$PROBE_BASE/package.json" ]]; then
  echo "seeding react-base from $PROBE_BASE"
  # package + lock first
  cp -f "$PROBE_BASE/package.json" "$BASE/"
  [[ -f "$PROBE_BASE/bun.lock" ]] && cp -f "$PROBE_BASE/bun.lock" "$BASE/"
fi

# Write scaffold files if missing (from eval scaffold patterns)
write_if_missing() {
  local path="$1"
  shift
  if [[ ! -f "$path" ]]; then
    mkdir -p "$(dirname "$path")"
    cat > "$path"
  fi
}

write_if_missing "$BASE/package.json" <<'EOF'
{
  "name": "sail-lab-react-tw",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "@vitejs/plugin-react": "^4.3.4",
    "autoprefixer": "^10.4.20",
    "postcss": "^8.4.49",
    "tailwindcss": "^3.4.17",
    "vite": "^6.0.3"
  }
}
EOF

write_if_missing "$BASE/index.html" <<'EOF'
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Untitled</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.jsx"></script>
  </body>
</html>
EOF

write_if_missing "$BASE/vite.config.js" <<'EOF'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
})
EOF

write_if_missing "$BASE/postcss.config.js" <<'EOF'
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
EOF

write_if_missing "$BASE/tailwind.config.js" <<'EOF'
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,jsx,ts,tsx}'],
  theme: { extend: {} },
  plugins: [],
}
EOF

write_if_missing "$BASE/src/main.jsx" <<'EOF'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.jsx'

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
EOF

write_if_missing "$BASE/src/App.jsx" <<'EOF'
// lab control arm: react-tw — implement the task prompt here
export default function App() {
  return (
    <div className="p-6 flex flex-col gap-4">
      <h1 className="text-2xl font-semibold">Untitled</h1>
      <p className="text-slate-600">Author your page here.</p>
    </div>
  )
}
EOF

write_if_missing "$BASE/src/index.css" <<'EOF'
@tailwind base;
@tailwind components;
@tailwind utilities;
EOF

write_if_missing "$BASE/AGENTS.md" <<'EOF'
# React + Tailwind control arm

Implement the task prompt in `src/App.jsx` using React and Tailwind utility
classes. Run `bun run dev` if you need a local preview. There is no Sail
toolchain in this workspace.
EOF

echo "== bun install in $BASE =="
(cd "$BASE" && bun install)
echo "react-base ready"
