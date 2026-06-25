import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-graph-canvas/sui-graph-canvas.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

const nodesPositioned = JSON.stringify([
  { id: 'a1', kind: 'agent', title: 'Alpha', sub: 'primary', x: 200, y: 300 },
  { id: 's1', kind: 'skill', title: 'Compile', x: 700, y: 200 },
  { id: 's2', kind: 'skill', title: 'Deploy', x: 700, y: 400 },
])
const edgesPositioned = JSON.stringify([
  { from: 'a1', to: 's1', kind: 'skill' },
  { from: 'a1', to: 's2', kind: 'skill', dashed: true },
])

describe('<sui-graph-canvas>', () => {

  it('renders all nodes and edge paths', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: edgesPositioned,
    })
    await flush()
    const nodes = shadow(el).querySelectorAll('.node')
    const paths = shadow(el).querySelectorAll('svg.edges path')
    expect(nodes.length).toBe(3)
    expect(paths.length).toBe(2)
  })

  it('positioned layout places nodes by their relative x/y ordering within the surface', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      width: '1000',
      height: '600',
      nodes: nodesPositioned,
      edges: '[]',
    })
    await flush()
    const at = (id: string) => {
      const n = shadow(el).querySelector(`.node[data-id="${id}"]`) as HTMLElement
      return { left: parseFloat(n.style.left), top: parseFloat(n.style.top) }
    }
    // a1=(200,300) s1=(700,200) s2=(700,400) over a 1000x600 surface.
    const a1 = at('a1')
    const s1 = at('s1')
    const s2 = at('s2')
    // All nodes sit inside the surface bounds (0–100% of width/height).
    for (const p of [a1, s1, s2]) {
      expect(p.left).toBeGreaterThanOrEqual(0)
      expect(p.left).toBeLessThanOrEqual(100)
      expect(p.top).toBeGreaterThanOrEqual(0)
      expect(p.top).toBeLessThanOrEqual(100)
    }
    // Relative geometry is preserved: the agent (smaller x) is left of the
    // skills, which share a column; s1 (smaller y) is above s2.
    expect(a1.left).toBeLessThan(s1.left)
    expect(s1.left).toBe(s2.left)
    expect(s1.top).toBeLessThan(s2.top)
    // The agent's x is 20% of the way across, y halfway down — assert the
    // proportional mapping without pinning an exact string format.
    expect(a1.left).toBeCloseTo(20, 5)
    expect(a1.top).toBeCloseTo(50, 5)
  })

  it('falls back to radial layout when positions are missing', async () => {
    const nodes = JSON.stringify([
      { id: 'a1', kind: 'agent', title: 'Alpha' },
      { id: 'sk1', kind: 'skill', title: 'Compile' },
      { id: 'mc1', kind: 'mcp', title: 'Fs' },
    ])
    const { el } = mount(reg, 'sui-graph-canvas', { nodes, edges: '[]' })
    await flush()
    const agent = shadow(el).querySelector('.node[data-id="a1"]') as HTMLElement
    const skill = shadow(el).querySelector('.node[data-id="sk1"]') as HTMLElement
    const agentLeft = parseFloat(agent.style.left)
    const skillLeft = parseFloat(skill.style.left)
    expect(agentLeft).toBeLessThan(skillLeft)
    expect(agentLeft).toBeGreaterThan(0)
  })

  it('fires sui-node-select with {id, kind} on click', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: '[]',
    })
    await flush()
    let detail: { id: string; kind: string } | null = null
    el.addEventListener('sui-node-select', (e: Event) => {
      detail = (e as CustomEvent).detail
    })
    const node = shadow(el).querySelector('.node[data-id="s1"]') as HTMLElement
    node.click()
    expect(detail).toEqual({ id: 's1', kind: 'skill' })
  })

  it('fires sui-node-select on Enter keypress', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: '[]',
    })
    await flush()
    let fired = 0
    el.addEventListener('sui-node-select', () => fired++)
    const node = shadow(el).querySelector('.node[data-id="a1"]') as HTMLElement
    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(fired).toBe(1)
  })

  it('fires sui-node-select on Space keypress', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: '[]',
    })
    await flush()
    let fired = 0
    el.addEventListener('sui-node-select', () => fired++)
    const node = shadow(el).querySelector('.node[data-id="s1"]') as HTMLElement
    node.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(fired).toBe(1)
  })

  it('selected-id attribute highlights the matching node via [data-selected]', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      'selected-id': 's2',
      nodes: nodesPositioned,
      edges: '[]',
    })
    await flush()
    const selected = shadow(el).querySelector('.node[data-selected]') as HTMLElement
    expect(selected).toBeTruthy()
    expect(selected.getAttribute('data-id')).toBe('s2')
  })

  it('clicking a node updates selected-id attribute on host', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: '[]',
    })
    await flush()
    const node = shadow(el).querySelector('.node[data-id="s1"]') as HTMLElement
    node.click()
    await flush()
    expect(el.getAttribute('selected-id')).toBe('s1')
  })

  it('per-kind template substitutes ${title}, ${sub}, ${id}', async () => {
    const { el } = mount(
      reg,
      'sui-graph-canvas',
      { nodes: nodesPositioned, edges: '[]' },
      `<template slot="node" data-kind="skill">
        <div class="skill-card">
          <span class="sid">\${id}</span>
          <span class="stitle">\${title}</span>
        </div>
      </template>`,
    )
    await flush()
    const nodeS1 = shadow(el).querySelector('.node[data-id="s1"]') as HTMLElement
    expect(nodeS1.querySelector('.stitle')?.textContent).toBe('Compile')
    expect(nodeS1.querySelector('.sid')?.textContent).toBe('s1')
  })

  it('renders edge path with kind attribute and dashed flag', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: edgesPositioned,
    })
    await flush()
    const paths = shadow(el).querySelectorAll('svg.edges path')
    const kinds = Array.from(paths).map((p) => p.getAttribute('data-kind'))
    expect(kinds).toEqual(['skill', 'skill'])
    expect(paths[1].hasAttribute('data-dashed')).toBe(true)
    expect(paths[0].hasAttribute('data-dashed')).toBe(false)
  })

  it('sets tabindex=0, role=button, and a useful aria-label on each node', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: '[]',
    })
    await flush()
    const first = shadow(el).querySelector('.node') as HTMLElement
    expect(first.getAttribute('tabindex')).toBe('0')
    expect(first.getAttribute('role')).toBe('button')
    expect(first.getAttribute('aria-label')).toContain('Alpha')
  })

  it('fires sui-node-hover on mouseover', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: '[]',
    })
    await flush()
    let detail: { id: string; kind: string } | null = null
    el.addEventListener('sui-node-hover', (e: Event) => {
      detail = (e as CustomEvent).detail
    })
    const node = shadow(el).querySelector('.node[data-id="a1"]') as HTMLElement
    node.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }))
    expect(detail).toEqual({ id: 'a1', kind: 'agent' })
  })

  it('renders svg.edges inside .canvas with part="edges"', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: edgesPositioned,
    })
    await flush()
    const canvas = shadow(el).querySelector('.canvas[part="canvas"]')
    expect(canvas).not.toBeNull()
    const svg = canvas!.querySelector('svg.edges[part="edges"]')
    expect(svg).not.toBeNull()
  })

  it('renders default node chrome when no matching template', async () => {
    const nodes = JSON.stringify([
      { id: 'r1', kind: 'rule', title: 'NoDebug', sub: 'enforce', x: 500, y: 300 },
    ])
    const { el } = mount(reg, 'sui-graph-canvas', { nodes, edges: '[]' })
    await flush()
    const node = shadow(el).querySelector('.node[data-id="r1"]') as HTMLElement
    expect(node.querySelector('.node-title')?.textContent).toBe('NoDebug')
    expect(node.querySelector('.node-sub')?.textContent).toBe('enforce')
    expect(node.querySelector('.tag')?.textContent).toBe('rule')
  })
})
