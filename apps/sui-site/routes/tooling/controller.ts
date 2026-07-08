// /tooling controller.

import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const code = {
  cli: [
    '# Compile a single .sui file. Non-zero exit on error.',
    'sui-compiler build src/my-card/my-card.sui',
    '',
    '# Walk a directory; reports every diagnostic (line, col, severity).',
    'sui-compiler build src/',
    '',
    '# Useful flags',
    'sui-compiler build src/ --clean                    # wipe stale output first',
    'sui-compiler build greeting.sui --component-name sui-greeting  # compile as a component',
    'sui-compiler build src/ --out dist                 # emit into one directory',
  ].join('\n'),
  lspJson: [
    '{',
    '  "sui": {',
    '    "command": "sui-lsp",',
    '    "extensionToLanguage": {',
    '      ".sui": "sui"',
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
    '# Treat sui-compiler build as a deterministic lint pass. The diagnostic',
    '# format is stable; grep on it.',
    'sui-compiler build src/ 2>&1 | tee /tmp/sui-lint.log',
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
