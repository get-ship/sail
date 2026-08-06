// E2E: hand-crafted modules mirroring what the sail compiler emits
// for {#if} and {#for} blocks.

import { describe, expect, it } from 'vitest'
import { defineComponent, signal } from '../src/index'
import { bindFor, bindIf, bindText, on } from '../src/internal'

describe('e2e: {#if} compiled shape', () => {
  it('swaps then/else on signal change', () => {
    const tag = 'sail-e2e-if'
    defineComponent(tag, { observedAttrs: [], props: {} }, (host) => {
      const cleanups: Array<() => void> = []
      const ok = signal(true)
      host.innerHTML = `<div><template data-si="0"></template></div>`
      {
        const _slot = host.querySelector('[data-si="0"]')!
        const _anchor = document.createComment('si')
        _slot.replaceWith(_anchor)
        cleanups.push(
          bindIf({
            anchor: _anchor,
            cond: () => ok(),
            then_: () => {
              const tpl = document.createElement('template')
              tpl.innerHTML = `<p>YES</p>`
              return { nodes: Array.from(tpl.content.childNodes) }
            },
            else_: () => {
              const tpl = document.createElement('template')
              tpl.innerHTML = `<p>NO</p>`
              return { nodes: Array.from(tpl.content.childNodes) }
            },
          }),
        )
      }
      // Stash signal for the test to manipulate
      ;(host as any).__ok = ok
      return () => cleanups.forEach((c) => c())
    })

    const el = document.createElement(tag) as HTMLElement & { __ok: any }
    document.body.appendChild(el)
    expect(el.textContent).toBe('YES')
    el.__ok.set(false)
    expect(el.textContent).toBe('NO')
    el.__ok.set(true)
    expect(el.textContent).toBe('YES')
    el.remove()
  })
})

describe('e2e: {#for} compiled shape', () => {
  it('renders, updates reused rows, and reorders', () => {
    const tag = 'sail-e2e-for'
    defineComponent(tag, { observedAttrs: [], props: {} }, (host) => {
      const cleanups: Array<() => void> = []
      const list = signal<Array<{ id: string; name: string }>>([
        { id: 'a', name: 'Alice' },
        { id: 'b', name: 'Bob' },
      ])
      host.innerHTML = `<ul><template data-sf="0"></template></ul>`
      {
        const _slot = host.querySelector('[data-sf="0"]')!
        const _anchor = document.createComment('sf')
        _slot.replaceWith(_anchor)
        cleanups.push(
          bindFor({
            anchor: _anchor,
            items: () => list(),
            key: (it: any) => it.id,
            render: (initialItem: any, initialIdx: number) => {
              const it = signal(initialItem)
              const _i = signal(initialIdx)
              const tpl = document.createElement('template')
              tpl.innerHTML = `<li><span data-st="0"></span></li>`
              const _root = tpl.content
              const _nodes = Array.from(_root.childNodes) as Node[]
              const _c: Array<() => void> = []
              {
                const _n = (_nodes[0] as Element).querySelector('[data-st="0"]')
                if (_n) _c.push(bindText(_n, () => String(it().name), [it]))
              }
              return {
                nodes: _nodes,
                cleanup: () => _c.forEach((c) => c()),
                update: (nextItem: any, nextIdx: number) => {
                  it.set(nextItem)
                  _i.set(nextIdx)
                },
              }
            },
          }),
        )
      }
      ;(host as any).__list = list
      return () => cleanups.forEach((c) => c())
    })

    const el = document.createElement(tag) as HTMLElement & { __list: any }
    document.body.appendChild(el)
    const ul = el.querySelector('ul')!
    expect(ul.textContent).toBe('AliceBob')

    // Update content of reused row (same key, new name).
    el.__list.set([
      { id: 'a', name: 'Alicia' },
      { id: 'b', name: 'Bob' },
    ])
    expect(ul.textContent).toBe('AliciaBob')

    // Reorder.
    el.__list.set([
      { id: 'b', name: 'Bob' },
      { id: 'a', name: 'Alicia' },
    ])
    expect(ul.textContent).toBe('BobAlicia')

    // Insert.
    el.__list.set([
      { id: 'b', name: 'Bob' },
      { id: 'c', name: 'Carol' },
      { id: 'a', name: 'Alicia' },
    ])
    expect(ul.textContent).toBe('BobCarolAlicia')

    el.remove()
  })

  it('version-signal pattern: item replacement updates DOM text (compiler approach B)', () => {
    // This test mirrors the exact JS the compiler emits for {#for} bodies
    // using approach B (version signal). The render closure seeds a per-row
    // __v = signal(0) and binding closures void-read it. The update function
    // mutates $scope.row and increments __v, causing effects to re-run.
    const host = document.createElement('div')
    document.body.appendChild(host)
    const anchor = document.createComment('for')
    host.appendChild(anchor)

    const $scope: any = Object.create(null)
    const list = signal([
      { id: 'a', label: 'Alpha' },
      { id: 'b', label: 'Beta' },
    ])

    bindFor({
      anchor,
      items: () => list(),
      key: (it: any) => it.id,
      render: (row: any, __i: number) => {
        const childScope: any = Object.create($scope)
        childScope.row = row
        childScope.__i = __i
        childScope.__v = signal(0)
        return (($s: any) => {
          const span = document.createElement('span')
          span.setAttribute('data-id', $s.row.id)
          const disposeText = bindText(span, () => (void $s.__v(), $s.row.label))
          return {
            nodes: [span],
            cleanup: disposeText,
            update: (newRow: any, newI: number) => {
              $s.row = newRow
              $s.__i = newI
              $s.__v.set((c: number) => c + 1)
            },
          }
        })(childScope)
      },
    })

    expect(host.textContent).toBe('AlphaBeta')

    list.set([
      { id: 'a', label: 'Alicia' },
      { id: 'b', label: 'Beta' },
    ])
    expect(host.textContent).toBe('AliciaBeta')

    list.set([
      { id: 'b', label: 'Bertha' },
      { id: 'a', label: 'Alicia' },
    ])
    expect(host.textContent).toBe('BerthaAlicia')

    host.remove()
  })

  it('counter inside a for row reacts via captured signal', () => {
    // Sanity: inside a for body, click handlers can mutate per-row
    // state captured at render time. (Compiler emits per-row consts
    // before the binding wires.)
    const tag = 'sail-e2e-for-row-state'
    defineComponent(tag, { observedAttrs: [], props: {} }, (host) => {
      const cleanups: Array<() => void> = []
      const list = signal<Array<{ id: string }>>([{ id: 'a' }, { id: 'b' }])
      host.innerHTML = `<ul><template data-sf="0"></template></ul>`
      {
        const _slot = host.querySelector('[data-sf="0"]')!
        const _anchor = document.createComment('sf')
        _slot.replaceWith(_anchor)
        cleanups.push(
          bindFor({
            anchor: _anchor,
            items: () => list(),
            key: (it: any) => it.id,
            render: (initialItem: any) => {
              const it = signal(initialItem)
              const local = signal(0)
              const tpl = document.createElement('template')
              tpl.innerHTML = `<li><button data-sa="0">+</button><span data-st="0"></span></li>`
              const _root = tpl.content
              const _nodes = Array.from(_root.childNodes) as Node[]
              const _c: Array<() => void> = []
              const li = _nodes[0] as Element
              {
                const _n = li.querySelector('[data-sa="0"]')
                if (_n) _c.push(on(_n, 'click', () => local.set(local() + 1)))
              }
              {
                const _n = li.querySelector('[data-st="0"]')
                if (_n) _c.push(bindText(_n, () => `${it().id}:${local()}`, [it, local]))
              }
              return {
                nodes: _nodes,
                cleanup: () => _c.forEach((c) => c()),
                update: (nextItem: any) => it.set(nextItem),
              }
            },
          }),
        )
      }
      return () => cleanups.forEach((c) => c())
    })

    const el = document.createElement(tag)
    document.body.appendChild(el)
    const buttons = el.querySelectorAll('button')
    const spans = el.querySelectorAll('span')
    expect(spans[0]!.textContent).toBe('a:0')
    ;(buttons[0] as HTMLButtonElement).click()
    ;(buttons[0] as HTMLButtonElement).click()
    expect(spans[0]!.textContent).toBe('a:2')
    expect(spans[1]!.textContent).toBe('b:0')
    el.remove()
  })
})
