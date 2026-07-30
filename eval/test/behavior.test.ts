import { describe, expect, it } from 'vitest'
import {
  gradeBehavior,
  runPlaywrightScenarioSteps,
  runScenariosOnHtml,
} from '../src/graders/behavior.ts'
import type { BehaviorSpec } from '../src/types.ts'

const LOGIN_HTML = `
  <h1>Sign in</h1>
  <label for="email">Email</label>
  <input id="email" name="email" />
  <label for="password">Password</label>
  <input id="password" type="password" name="password" />
  <button type="submit">Sign in</button>
  <a href="/forgot">Forgot password?</a>
`

const spec: BehaviorSpec = {
  view: 'login',
  scenarios: [
    {
      name: 'chrome',
      steps: [
        { see: 'Sign in' },
        { see: 'Email' },
        { see: 'role:button name:Sign in' },
        { see: 'Forgot password?' },
      ],
    },
    {
      name: 'fill email',
      steps: [
        { fill: { field: 'Email', value: 'a@b.co' } },
        { see: 'a@b.co' },
      ],
    },
  ],
}

describe('behavior scenarios', () => {
  it('passes faithful HTML', () => {
    const results = runScenariosOnHtml(LOGIN_HTML, spec.scenarios)
    expect(results.every((r) => r.passed)).toBe(true)
  })

  it('fails missing control and reports step index', () => {
    const broken: BehaviorSpec = {
      scenarios: [
        {
          name: 'missing',
          steps: [{ see: 'Sign in' }, { see: 'role:button name:Submit' }],
        },
      ],
    }
    const [r] = runScenariosOnHtml(LOGIN_HTML, broken.scenarios)
    expect(r!.passed).toBe(false)
    expect(r!.failedStep).toBe(1)
    expect(r!.error).toMatch(/Submit/)
  })

  it('notSee fails when text present', () => {
    const results = runScenariosOnHtml(LOGIN_HTML, [
      { name: 'x', steps: [{ notSee: 'Sign in' }] },
    ])
    expect(results[0]!.passed).toBe(false)
  })

  it('gradeBehavior skips without html/baseUrl when allowSkip', async () => {
    const g = await gradeBehavior({ spec, allowSkip: true })
    expect(g.skipped).toBe(true)
    expect(g.ok).toBe(true)
  })

  it('gradeBehavior defaults strict — no html/baseUrl fails', async () => {
    const g = await gradeBehavior({ spec })
    expect(g.ok).toBe(false)
    expect(g.skipped).toBeFalsy()
  })

  it('gradeBehavior runs on html', async () => {
    const g = await gradeBehavior({ spec, html: LOGIN_HTML })
    expect(g.ok).toBe(true)
    expect(g.passed).toBe(2)
  })

  it('gradeBehavior with empty spec is skipped', async () => {
    const g = await gradeBehavior({ spec: null })
    expect(g.skipped).toBe(true)
  })

  it('rejects fill when field missing', () => {
    const results = runScenariosOnHtml('<h1>Only title</h1>', [
      { name: 'f', steps: [{ fill: { field: 'Email', value: 'x' } }] },
    ])
    expect(results[0]!.passed).toBe(false)
    expect(results[0]!.error).toMatch(/Email/)
  })
})

describe('playwright scenario step control flow', () => {
  it('returns a single fail result on mid-scenario step failure (no pass)', async () => {
    let step = 0
    const page = {
      getByText: () => ({
        first: () => ({
          isVisible: async () => {
            step++
            // first see ok, second fails
            return step === 1
          },
          click: async () => {},
        }),
      }),
      getByRole: () => ({
        first: () => ({
          isVisible: async () => false,
          click: async () => {},
          fill: async () => {},
        }),
      }),
      getByLabel: () => ({
        first: () => ({
          fill: async () => {},
          isVisible: async () => false,
        }),
      }),
    }
    const result = await runPlaywrightScenarioSteps(page, {
      name: 'mid-fail',
      steps: [{ see: 'First' }, { see: 'Second' }, { see: 'Third' }],
    })
    expect(result.passed).toBe(false)
    expect(result.failedStep).toBe(1)
    expect(result.name).toBe('mid-fail')
    // Only one result object — caller must not also push pass
  })

  it('returns pass when all steps succeed', async () => {
    const page = {
      getByText: () => ({
        first: () => ({
          isVisible: async () => true,
          click: async () => {},
        }),
      }),
      getByRole: () => ({
        first: () => ({
          isVisible: async () => true,
          click: async () => {},
          fill: async () => {},
        }),
      }),
      getByLabel: () => ({
        first: () => ({
          fill: async () => {},
          isVisible: async () => true,
        }),
      }),
    }
    const result = await runPlaywrightScenarioSteps(page, {
      name: 'ok',
      steps: [{ see: 'Hello' }],
    })
    expect(result).toEqual({ name: 'ok', passed: true })
  })
})
