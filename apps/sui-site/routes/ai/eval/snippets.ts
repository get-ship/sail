// Code snippets for the /ai/eval page. The original 11-sui-eval lesson
// shows `import ... from '@ship/sail-eval'` lines; we omit the `import`
// statement here so the bundler doesn't try to externalize the
// `@ship/sail-eval` bare specifier (this site doesn't depend on it).
// The API call shapes are unchanged.

export const evalExample = `// runEval + loadCatalog come from the sail-eval package.
const task: EvalTask = {
  description: 'A counter with +1/-1 buttons and a reset.',
  catalog: await loadCatalog(),       // the component catalog
  spec: {
    interactions: [
      { action: 'click', selector: 'button[data-test="bump-up"]' },
      { assert: 'text',  selector: 'strong',                value: '1' },
      { action: 'click', selector: 'button[data-test="reset"]' },
      { assert: 'text',  selector: 'strong',                value: '0' },
    ],
  },
}

const result = await runEval(task, { model: 'sonnet' })
// → {
//     score: 0.92,
//     sub: { compile: 1, scope: 1, behaviour: 0.83 },
//     report: '…',
//     output: { template, controller },
//   }`

export const hatchExample = `// sniffEscapeHatches() comes from the sail-eval package.
const hatches = sniffEscapeHatches(result.output)
// → [
//     { kind: 'raw-script', line: 14, msg: '<script> in template' },
//     { kind: 'ts-ignore', line: 22, msg: '@ts-ignore in controller' },
//   ]`
