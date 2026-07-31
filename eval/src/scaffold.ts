/**
 * Scaffold a workdir for agent authoring.
 * - blank / kitchen-sink: sail app templates (app-level blank lives here —
 *   deck CLI scaffolds route/component/pack/theme, not a full app shell).
 * - react-tw: Vite + React + Tailwind control-arm baseline.
 * - vanilla: bare static html/css/js control arm.
 */

import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  writeFileSync,
} from 'node:fs'
import { dirname, join, relative } from 'node:path'
import type { ScaffoldKind } from './types.ts'
import { repoRoot } from './paths.ts'

export type { ScaffoldKind }

const BLANK_PAGE = `<!-- eval scaffold: blank — replace with your solution -->
<column gap="4" sx={style.pad(6)}>
  <h1>Untitled</h1>
  <p>Author your page here.</p>
</column>
`

const KITCHEN_SINK_PAGE = `<!-- eval scaffold: kitchen-sink starter -->
<column gap="6" sx={style.pad(6)}>
  <row gap="3" align="center">
    <h1>Kitchen sink</h1>
    <badge tone="info">scaffold</badge>
  </row>
  <card>
    <column gap="3" sx={style.pad(4)}>
      <h2>Section</h2>
      <p>Use L1 elements and layout keywords. Prefer card/button/input over raw div/span.</p>
      <row gap="2">
        <button type="button" tone="primary">Primary</button>
        <button type="button" tone="default">Default</button>
      </row>
    </column>
  </card>
</column>
`

const SAIL_CONFIG = `{
  "theme": {
    "default": "ship",
    "defaultMode": "light",
    "enabled": ["ship"]
  },
  "render": "csr"
}
`

const PACKAGE_JSON = `{
  "name": "sail-eval-workdir",
  "private": true,
  "type": "module"
}
`

/** Vite + React + Tailwind control-arm shell (identical prompts, different stack). */
const REACT_TW_APP = `// eval control arm: react-tw — implement the task prompt here
export default function App() {
  return (
    <div className="p-6 flex flex-col gap-4">
      <h1 className="text-2xl font-semibold">Untitled</h1>
      <p className="text-slate-600">Author your page here.</p>
    </div>
  )
}
`

const REACT_TW_MAIN = `import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.jsx'

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
`

const REACT_TW_CSS = `@tailwind base;
@tailwind components;
@tailwind utilities;
`

const REACT_TW_INDEX_HTML = `<!doctype html>
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
`

const REACT_TW_VITE_CONFIG = `import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
})
`

const REACT_TW_POSTCSS = `export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
`

const REACT_TW_TAILWIND = `/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,jsx,ts,tsx}'],
  theme: {
    extend: {},
  },
  plugins: [],
}
`

const REACT_TW_PACKAGE = `{
  "name": "sail-eval-control-react-tw",
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
`

const REACT_TW_README = `# Control arm: Vite + React + Tailwind

Same task prompt as the sail arm. Implement the UI in \`src/App.jsx\` using React and Tailwind utility classes.

\`\`\`bash
npm install
npm run dev
\`\`\`

This arm exists so sail scores are falsifiable upward — without controls, numbers are unfalsifiable. There is no Sail toolchain here.
`

/** Bare static HTML/CSS/JS control arm — no framework, no sail. */
const VANILLA_HTML = `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Untitled</title>
    <link rel="stylesheet" href="./styles.css" />
  </head>
  <body>
    <main class="page">
      <h1>Untitled</h1>
      <p>Author your page here.</p>
    </main>
    <script type="module" src="./main.js"></script>
  </body>
</html>
`

const VANILLA_CSS = `/* eval control arm: vanilla */
:root {
  color-scheme: light dark;
  font-family: system-ui, sans-serif;
  line-height: 1.5;
}
body {
  margin: 0;
}
.page {
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
`

const VANILLA_JS = `// eval control arm: vanilla — implement the task prompt here
console.info('vanilla arm ready')
`

const VANILLA_README = `# Control arm: vanilla HTML/CSS/JS

Same task prompt as the sail arm. Implement the UI with static \`index.html\`, \`styles.css\`, and \`main.js\`. No React, no Tailwind toolchain, no Sail.
`

/** Write scaffold files into workdir. Returns relative paths created. */
export function scaffoldWorkdir(
  workdir: string,
  kind: ScaffoldKind,
): string[] {
  if (kind === 'react-tw') {
    return scaffoldReactTw(workdir)
  }
  if (kind === 'vanilla') {
    return scaffoldVanilla(workdir)
  }

  mkdirSync(join(workdir, 'routes'), { recursive: true })
  const files: string[] = []

  const page = kind === 'kitchen-sink' ? KITCHEN_SINK_PAGE : BLANK_PAGE
  writeFileSync(join(workdir, 'routes', 'page.sail'), page)
  files.push('routes/page.sail')

  writeFileSync(join(workdir, 'sail.config.jsonc'), SAIL_CONFIG)
  files.push('sail.config.jsonc')

  writeFileSync(join(workdir, 'package.json'), PACKAGE_JSON)
  files.push('package.json')

  // Drop a short skill/README so the agent has surface guidance.
  const skill = buildSkillStub()
  writeFileSync(join(workdir, 'SAIL_SKILL.md'), skill)
  files.push('SAIL_SKILL.md')

  return files
}

function scaffoldReactTw(workdir: string): string[] {
  const files: string[] = []
  mkdirSync(join(workdir, 'src'), { recursive: true })

  writeFileSync(join(workdir, 'index.html'), REACT_TW_INDEX_HTML)
  files.push('index.html')

  writeFileSync(join(workdir, 'vite.config.js'), REACT_TW_VITE_CONFIG)
  files.push('vite.config.js')

  writeFileSync(join(workdir, 'postcss.config.js'), REACT_TW_POSTCSS)
  files.push('postcss.config.js')

  writeFileSync(join(workdir, 'tailwind.config.js'), REACT_TW_TAILWIND)
  files.push('tailwind.config.js')

  writeFileSync(join(workdir, 'src', 'main.jsx'), REACT_TW_MAIN)
  files.push('src/main.jsx')

  writeFileSync(join(workdir, 'src', 'App.jsx'), REACT_TW_APP)
  files.push('src/App.jsx')

  writeFileSync(join(workdir, 'src', 'index.css'), REACT_TW_CSS)
  files.push('src/index.css')

  writeFileSync(join(workdir, 'package.json'), REACT_TW_PACKAGE)
  files.push('package.json')

  writeFileSync(join(workdir, 'README.md'), REACT_TW_README)
  files.push('README.md')

  return files
}

function scaffoldVanilla(workdir: string): string[] {
  const files: string[] = []
  writeFileSync(join(workdir, 'index.html'), VANILLA_HTML)
  files.push('index.html')
  writeFileSync(join(workdir, 'styles.css'), VANILLA_CSS)
  files.push('styles.css')
  writeFileSync(join(workdir, 'main.js'), VANILLA_JS)
  files.push('main.js')
  writeFileSync(join(workdir, 'README.md'), VANILLA_README)
  files.push('README.md')
  return files
}

function buildSkillStub(): string {
  return `# Sail authoring skill (eval harness)

You are authoring a Sail UI page. Write \`.sail\` files under \`routes/\`.

## Layout keywords (prefer these over div/span)
- \`<column gap="4">\`, \`<row gap="2">\`, \`<grid cols="3" gap="4">\`, \`<stack>\`, \`<center>\`

## L1 elements (use vocabulary, not primitives)
- card, button, input, textarea, label, field, badge, navbar, table, tabs, dialog, …

## Styling
- Use \`sx={style.pad(4).radius(lg)}\` style builder — not utility-string classes.
- Tones: default, primary, info, success, warning, destructive, accent, muted.

## Constraints
- Prefer L1 elements over raw HTML div/span for structure.
- Query-visible text and labels matter: behavioral specs check what users see.
- Keep the page self-contained in \`routes/page.sail\` unless the task needs more files.
`
}

/**
 * Copy golden (or fixture) tree into workdir, preserving relative structure.
 * Only copies regular files; skips node_modules/.git.
 */
export function copyTreeInto(srcDir: string, destDir: string): string[] {
  if (!existsSync(srcDir)) return []
  const written: string[] = []
  const walk = (dir: string) => {
    for (const name of readdirSync(dir, { withFileTypes: true })) {
      if (name.name === 'node_modules' || name.name === '.git' || name.name === 'runs') continue
      const from = join(dir, name.name)
      const rel = relative(srcDir, from)
      const to = join(destDir, rel)
      if (name.isDirectory()) {
        mkdirSync(to, { recursive: true })
        walk(from)
      } else if (name.isFile()) {
        mkdirSync(dirname(to), { recursive: true })
        copyFileSync(from, to)
        written.push(rel)
      }
    }
  }
  mkdirSync(destDir, { recursive: true })
  walk(srcDir)
  return written
}

/** Optional: copy a committed author-surface skill if present in the monorepo. */
export function injectSkillFile(workdir: string): string | null {
  const candidates = [
    join(repoRoot(), 'packages/sail/runtime/author-surface.json'),
  ]
  for (const c of candidates) {
    if (existsSync(c)) {
      const dest = join(workdir, 'author-surface.json')
      cpSync(c, dest)
      return 'author-surface.json'
    }
  }
  return null
}
