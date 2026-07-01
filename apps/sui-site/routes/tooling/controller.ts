// /tooling controller.

import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const code = {
  cli: [
    '# Compile a single .sui file. Non-zero exit on error.',
    'sui compile src/my-card/my-card.sui',
    '',
    '# Walk a project; reports every diagnostic, line+col, severity.',
    'sui compile "src/**/*.sui"',
    '',
    '# Useful flags',
    'sui compile --watch   src/      # incremental, fast',
    'sui compile --json    src/      # machine-readable diagnostic stream',
    'sui compile --strict  src/      # promote warnings to errors',
  ].join('\n'),
  lspJson: [
    '{',
    '  "lsp": {',
    '    "server": "sui-lsp",',
    '    "rootMarkers": [".git"],',
    '    "filePatterns": ["**/*.sui", "**/*.controller.ts"],',
    '    "settings": {',
    '      "sui": {',
    '        "diagnostics": { "lint": true, "semantic": true },',
    '        "formatOnSave": true',
    '      }',
    '    }',
    '  }',
    '}',
  ].join('\n'),
  browser: [
    '// In a controller mounted to <sui-code-editor>\'s host:',
    'import initWasm, { analyze as wasmAnalyze } from "&#64;ship/sui-lsp-browser/diagnostics"',
    'import { allSources } from "&#64;ship/sui-lsp-browser/completions"',
    '',
    '// On the host\'s mount(), prime both:',
    'await initWasm("/sui_compiler_bg.wasm")',
    'host.setCompletions(allSources())',
    '',
    '// On every debounced value change, push fresh diagnostics:',
    'host.setDiagnostics((await wasmAnalyze(text)).diagnostics ?? [])',
  ].join('\n'),
  agent: [
    '# Treat sui compile as a deterministic lint pass. The diagnostic',
    '# format is stable; grep on it.',
    'sui compile "src/**/*.sui" 2>&1 | tee /tmp/sui-lint.log',
    '',
    '# A non-zero exit means the edit introduced a regression.',
    'if [ $? -ne 0 ]; then',
    '  echo "sui compile failed; revise before continuing"',
    '  exit 1',
    'fi',
  ].join('\n'),
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ code }),
})
