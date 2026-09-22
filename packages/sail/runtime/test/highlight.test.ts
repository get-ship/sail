import { afterEach, describe, expect, it } from 'vitest'
import { highlight, registerSailHighlighter } from '../src/actions/highlight'

afterEach(() => {
  registerSailHighlighter(null)
})

/** Strip the tok-* spans and undo escaping — must round-trip to the source. */
function plainText(html: string): string {
  return html
    .replace(/<span class="tok-[\w-]+">/g, '')
    .replace(/<\/span>/g, '')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&amp;/g, '&')
}

/** All source ranges wrapped in a given tok class. */
function tokens(html: string, cls: string): string[] {
  const re = new RegExp(`<span class="tok-${cls}">([\\s\\S]*?)</span>`, 'g')
  return [...html.matchAll(re)].map((m) => plainText(m[1]!))
}

const TEMPLATE = `<card>
  <for {flights() as f} key={f.id}>
    <match {f.status}>
      {'boarding' => <badge>Boarding</badge>}
      {else => <badge>On time</badge>}
    </match>
  </for>
</card>`

const CONTROLLER = `<controller>
export default controller(() => {
  const flights = signal([])
  if (flights().length === 0) return { flights }
  return { flights }
})
</controller>`

describe('highlight — markup text is prose, never keywords', () => {
  it('does not keyword-match words inside element text ("Signals in the Rain")', () => {
    const out = highlight('<strong>Signals in the Rain</strong>')
    expect(tokens(out, 'keyword')).toEqual([])
    expect(out).toContain('Signals in the Rain')
  })

  it('does not keyword-match text in a template snippet', () => {
    const out = highlight(TEMPLATE)
    expect(tokens(out, 'keyword')).toEqual([])
    expect(plainText(out)).toContain('Boarding')
    expect(plainText(out)).toContain('On time')
  })

  it('does not treat words inside attribute strings as keywords', () => {
    const out = highlight('<badge tone="in for of default">x</badge>')
    expect(tokens(out, 'keyword')).toEqual([])
    expect(tokens(out, 'string')).toEqual(['"in for of default"'])
  })
})

describe('highlight — directive headers and bindings', () => {
  it('tokenizes element control-flow attrs as bindings, never comments', () => {
    const out = highlight(TEMPLATE)
    expect(tokens(out, 'comment')).toEqual([])
    expect(tokens(out, 'bind')).toEqual(
      expect.arrayContaining(['{flights() as f}', '{f.id}', '{f.status}']),
    )
    expect(tokens(out, 'bind').some((t) => t.startsWith("{'boarding'"))).toBe(true)
    expect(tokens(out, 'tag')).toEqual(expect.arrayContaining(['<for', '<match']))
  })

  it('brace-matches bindings with quotes and nested braces', () => {
    const out = highlight("<button on:click={toggle}>{playing() ? 'Pause' : 'Play'}</button>")
    expect(tokens(out, 'bind')).toEqual(['{toggle}', "{playing() ? 'Pause' : 'Play'}"])
    expect(tokens(out, 'attr')).toContain('on:click')
  })

  it('handles double-brace attribute values (motion props)', () => {
    const out = highlight(`<div fx:loop={{ translate-y: 'spring:gentle', stagger: '0.1s' }}></div>`)
    expect(tokens(out, 'bind')).toEqual([`{{ translate-y: 'spring:gentle', stagger: '0.1s' }}`])
  })
})

describe('highlight — controller code stays code', () => {
  const out = highlight(CONTROLLER)

  it('never emits bind spans inside code (nested arrow bodies stay intact)', () => {
    expect(tokens(out, 'bind')).toEqual([])
  })

  it('never emits comment spans (nothing in the snippet is a comment)', () => {
    expect(tokens(out, 'comment')).toEqual([])
  })

  it('keywords, verbs, strings and numbers get their classes', () => {
    const kws = tokens(out, 'keyword')
    for (const kw of ['export', 'default', 'const', 'if', 'return']) {
      expect(kws).toContain(kw)
    }
    const fns = tokens(out, 'fn')
    for (const v of ['controller', 'signal']) expect(fns).toContain(v)
    expect(tokens(out, 'number')).toEqual(['0'])
  })

  it('marks the <controller> wrapper tags as tags', () => {
    const tags = tokens(out, 'tag')
    expect(tags).toContain('<controller')
    expect(tags).toContain('</controller')
  })
})

describe('highlight — pure-code snippets keep keyword highlighting', () => {
  it('highlights plain TS snippets (no leading tag)', () => {
    const out = highlight("import { signal } from '@ship/sail-runtime'\nconst count = signal(0)")
    expect(tokens(out, 'keyword')).toEqual(expect.arrayContaining(['import', 'from', 'const']))
    expect(tokens(out, 'fn')).toContain('signal')
    expect(tokens(out, 'bind')).toEqual([])
  })

  it('hands off to markup when a .sail code prelude is followed by a template', () => {
    const src = `const playing = signal(true)\n\n<column gap="2">\n  <badge>{playing() ? 'playing' : 'paused'}</badge>\n</column>`
    const out = highlight(src)
    expect(tokens(out, 'keyword')).toContain('const')
    expect(tokens(out, 'fn')).toContain('signal')
    expect(tokens(out, 'tag')).toEqual(expect.arrayContaining(['<column', '<badge', '</badge']))
    expect(tokens(out, 'bind')).toEqual(["{playing() ? 'playing' : 'paused'}"])
    const gen = highlight('const flights = signal<Flight[]>([])')
    expect(tokens(gen, 'tag')).toEqual([])
  })

  it('keeps shell comments in shell snippets', () => {
    const out = highlight('npm install @ship/sail-runtime # the runtime')
    expect(tokens(out, 'comment')).toEqual(['# the runtime'])
    expect(tokens(out, 'fn')).toEqual(expect.arrayContaining(['npm', 'install']))
  })
})

describe('highlight — output invariants', () => {
  it('round-trips inline sail snippets to their source', () => {
    for (const src of [TEMPLATE, CONTROLLER, '<p>a < b && c > d</p>']) {
      expect(plainText(highlight(src))).toBe(src)
    }
  })

  it('escapes HTML so output cannot inject', () => {
    const out = highlight('<p>a < b && c > d</p>')
    expect(out).not.toMatch(/<(?!\/?span|span)[^>]*onerror/i)
    expect(plainText(out)).toBe('<p>a < b && c > d</p>')
  })
})

describe('highlight — registered sail painter', () => {
  it('uses the painter for sail-shaped source and leaves TS/shell on regex', () => {
    registerSailHighlighter((src) => `SAIL:${src}`)
    expect(highlight('<div>{n()}</div>')).toBe('SAIL:<div>{n()}</div>')
    const ts = highlight('const x = 1')
    expect(ts).toContain('tok-keyword')
    expect(ts).not.toContain('SAIL:')
    const sh = highlight('npm install x')
    expect(sh).toContain('tok-fn')
    expect(sh).not.toContain('SAIL:')
  })

  it('falls back to regex when the painter throws', () => {
    registerSailHighlighter(() => {
      throw new Error('boom')
    })
    const out = highlight('<strong>ok</strong>')
    expect(tokens(out, 'tag')).toContain('<strong')
    expect(out).toContain('ok')
  })
})
