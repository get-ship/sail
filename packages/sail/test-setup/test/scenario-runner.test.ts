// Proves the behavioral scenario runner — all 5 verbs (see/notSee/click/fill/
// choose) against a real mounted view, queried by what a user perceives.

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
import { runScenarios, type BehaviorSpec } from '../src/scenario-runner'

// A tiny two-step wizard exercising navigation, an input→preview binding, and a
// single-select group — the same shapes create-space.behavior.jsonc asserts.
const WIZARD = `
{#if step() === 0}
  <h2>Welcome</h2>
  <button on:click={() => step.set(1)}>Start</button>
{:else}
  <h2>Details</h2>
  <p>{name()}</p>
  <label for="nm">Name</label>
  <input id="nm" on:input={(e) => name.set(e.target.value)} />
  <h3>Theme</h3>
  <div>
    <button on:click={() => theme.set('Light')}>Light</button>
    <button on:click={() => theme.set('Dark')}>Dark</button>
  </div>
  <p>{theme()}</p>
  <button on:click={() => step.set(0)}>Back</button>
{/if}
`

const controller = () => ({
  step: signal(0),
  name: signal('Untitled'),
  theme: signal('Light'),
})

const spec: BehaviorSpec = {
  view: 'wizard',
  scenarios: [
    {
      name: 'welcome shows then advances',
      steps: [
        { see: 'Welcome' },
        { notSee: 'Details' },
        { click: 'Start' },
        { see: 'Details' },
        { notSee: 'Welcome' },
      ],
    },
    {
      name: 'typing a name updates the preview',
      steps: [
        { click: 'Start' },
        { fill: { field: 'Name', value: 'My Space' } },
        { see: 'My Space' },
      ],
    },
    {
      name: 'choosing a theme reflects it',
      steps: [
        { click: 'Start' },
        { choose: { option: 'Dark', in: 'Theme' } },
        { see: 'Dark' },
      ],
    },
    {
      name: 'back returns to welcome',
      steps: [
        { click: 'Start' },
        { click: 'Back' },
        { see: 'Welcome' },
      ],
    },
  ],
}

describe('scenario-runner', () => {
  it('passes every scenario for a faithful build', async () => {
    const results = await runScenarios(WIZARD, controller, spec)
    const failures = results.filter((r) => !r.passed)
    expect(failures, JSON.stringify(failures, null, 2)).toHaveLength(0)
  })

  it('fails the scenario and reports the step when behavior is wrong', async () => {
    const broken: BehaviorSpec = {
      view: 'wizard',
      scenarios: [
        {
          name: 'expects a control that does not exist',
          steps: [{ click: 'Start' }, { see: 'role:button name:Submit' }],
        },
      ],
    }
    const [r] = await runScenarios(WIZARD, controller, broken)
    expect(r!.passed).toBe(false)
    expect(r!.failedStep).toBe(1)
    expect(r!.error).toContain('Submit')
  })
})
