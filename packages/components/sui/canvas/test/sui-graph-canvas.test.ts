import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-graph-canvas/sui-graph-canvas.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// Light-DOM components: the template renders into the host itself.
const root = (el: HTMLElement) => (el.shadowRoot ?? el)

// Anatomy (no classes — the language bans them): every node is a role=button
// card carrying data-id; the card hangs in a <center> inside the 0×0 anchor
// that holds its position, so the anchor is the card's grandparent.
const card = (el: HTMLElement, id: string) =>
  root(el).querySelector(`[role="button"][data-id="${id}"]`) as HTMLElement
const anchorOf = (c: HTMLElement) => c.parentElement!.parentElement! as HTMLElement
const cards = (el: HTMLElement) => root(el).querySelectorAll('[role="button"][data-id]')

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
    const nodes = cards(el)
    const paths = root(el).querySelectorAll('svg path')
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
      const anchor = anchorOf(card(el, id))
      return { left: parseFloat(anchor.style.left), top: parseFloat(anchor.style.top) }
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
    const agentLeft = parseFloat(anchorOf(card(el, 'a1')).style.left)
    const skillLeft = parseFloat(anchorOf(card(el, 'sk1')).style.left)
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
    card(el, 's1').click()
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
    card(el, 'a1').dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
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
    card(el, 's1').dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(fired).toBe(1)
  })

  it('selected-id attribute marks the matching node aria-current', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      'selected-id': 's2',
      nodes: nodesPositioned,
      edges: '[]',
    })
    await flush()
    const selected = root(el).querySelector('[role="button"][aria-current="true"]') as HTMLElement
    expect(selected).toBeTruthy()
    expect(selected.getAttribute('data-id')).toBe('s2')
  })

  it('clicking a node updates selected-id attribute on host', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: '[]',
    })
    await flush()
    card(el, 's1').click()
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
    const nodeS1 = card(el, 's1')
    expect(nodeS1.querySelector('.stitle')?.textContent).toBe('Compile')
    expect(nodeS1.querySelector('.sid')?.textContent).toBe('s1')
  })

  it('renders edge path with kind attribute and dashed flag', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: edgesPositioned,
    })
    await flush()
    const paths = root(el).querySelectorAll('svg path')
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
    const first = cards(el)[0] as HTMLElement
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
    card(el, 'a1').dispatchEvent(new MouseEvent('mouseover', { bubbles: true }))
    expect(detail).toEqual({ id: 'a1', kind: 'agent' })
  })

  it('renders the edge svg over the graph coordinate space', async () => {
    const { el } = mount(reg, 'sui-graph-canvas', {
      nodes: nodesPositioned,
      edges: edgesPositioned,
    })
    await flush()
    const svg = root(el).querySelector('svg')
    expect(svg).not.toBeNull()
    // The viewBox IS the coordinate space (and the surface's aspect ratio).
    expect(svg!.getAttribute('viewBox')).toBe('0 0 1000 600')
  })

  it('renders default node chrome when no matching template', async () => {
    const nodes = JSON.stringify([
      { id: 'r1', kind: 'rule', title: 'NoDebug', sub: 'enforce', x: 500, y: 300 },
    ])
    const { el } = mount(reg, 'sui-graph-canvas', { nodes, edges: '[]' })
    await flush()
    const node = card(el, 'r1')
    // Default chrome: the kind eyebrow, the title, and the sub-line.
    const text = node.textContent ?? ''
    expect(text).toContain('rule')
    expect(text).toContain('NoDebug')
    expect(text).toContain('enforce')
    expect(node.getAttribute('data-state')).toBe('rule')
  })
})
