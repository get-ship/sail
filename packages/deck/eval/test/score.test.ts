import { describe, expect, it } from 'vitest'
import { scoreOutput, sniffEscapeHatches } from '../src/score'

describe('sniffEscapeHatches', () => {
  it('returns empty for clean templates', () => {
    expect(sniffEscapeHatches('<div>hi</div>')).toEqual([])
  })

  it('catches inline scripts', () => {
    const hits = sniffEscapeHatches('<div><script>alert(1)</script></div>')
    expect(hits.some((h) => h.includes('script'))).toBe(true)
  })

  it('catches eval, new Function, document.write, customElements.define', () => {
    expect(sniffEscapeHatches('<x on:click={() => eval("x")} />')).toContain(
      'eval() in template',
    )
    expect(
      sniffEscapeHatches('<x />', 'const f = new Function("return 1")'),
    ).toContain('new Function() in setup')
    expect(sniffEscapeHatches('<x on:click={() => document.write("x")} />')).toContain(
      'document.write in template',
    )
    expect(
      sniffEscapeHatches('<x />', 'customElements.define("x-y", class {})'),
    ).toContain('customElements.define in setup')
  })

  it('does not flag the harmless word "evaluate"', () => {
    expect(sniffEscapeHatches('<p>evaluate this</p>')).toEqual([])
  })
})

describe('scoreOutput — compile path', () => {
  it('compiled=false on a malformed template', async () => {
    // The <state> directive was retired (reactive state lives in the
    // controller's `scope:` factory now), so a top-level <state> is a
    // parse error.
    const report = await scoreOutput({
      template: '<state name="n" value="0" /><span>{n()}</span>',
      behavioralTests: 'expect(true)',
    })
    expect(report.compiled).toBe(false)
    expect(report.behavioral).toBe(false)
    expect(report.diagnostics.compile).toBeTruthy()
  })

  it('compiled=true for a valid static template', async () => {
    const report = await scoreOutput({
      template: '<h1 data-test="t">hello</h1>',
      behavioralTests:
        'expect(container.querySelector(\'[data-test="t"]\').textContent === "hello");',
    })
    expect(report.compiled).toBe(true)
    expect(report.boundedScope).toBe(true)
    expect(report.behavioral).toBe(true)
  })
})

describe('scoreOutput — boundedScope path', () => {
  it('boundedScope=false when the template embeds a <script>', async () => {
    const report = await scoreOutput({
      template: '<div>ok</div><script>console.log(1)</script>',
      behavioralTests: 'expect(true)',
    })
    expect(report.boundedScope).toBe(false)
    expect(report.diagnostics.boundedScope?.[0]).toMatch(/script/)
  })
})

describe('scoreOutput — behavioral path', () => {
  it('behavioral=true when assertions hold', async () => {
    // Template-only behavioral check — the runtime evaluates mustache
    // expressions at mount. (Client reactive state lives in a controller
    // `scope:` factory now; the harness mounts templates without a setup,
    // so the old inline-<state> counter is no longer expressible here. A
    // setup-mounting path would restore reactive-state behavioral scoring.)
    const report = await scoreOutput({
      template: '<span data-test="v">{2 + 2}</span>',
      behavioralTests:
        'expect(container.querySelector(\'[data-test="v"]\').textContent === "4");',
    })
    expect(report.behavioral).toBe(true)
  })

  it('behavioral=false when assertions throw', async () => {
    const report = await scoreOutput({
      template: '<span data-test="v">hi</span>',
      behavioralTests:
        'expect(container.querySelector(\'[data-test="v"]\').textContent === "bye", "expected bye");',
    })
    expect(report.behavioral).toBe(false)
    expect(report.diagnostics.behavioral).toMatch(/expected bye/)
  })
})
