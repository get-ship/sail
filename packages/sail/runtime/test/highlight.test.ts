import { describe, expect, it } from 'vitest'
import { highlight } from '../src/actions/highlight'
// The real docs snippets — the fixtures the owner saw the defects on.
import { code as sailFileSnippets } from '../../../../apps/sail-site/routes/sail/language/sail-file/snippets'

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

describe('highlight — markup text is prose, never keywords', () => {
  it('does not keyword-match words inside element text ("Signals in the Rain")', () => {
    const out = highlight('<strong>Signals in the Rain</strong>')
    expect(tokens(out, 'keyword')).toEqual([])
    // The prose survives verbatim, unwrapped.
    expect(out).toContain('Signals in the Rain')
  })

  it('does not keyword-match text in the real template snippet', () => {
    const out = highlight(sailFileSnippets.template)
    // `for`/`of`/`in` appear only inside {…} directives — never as keywords.
    expect(tokens(out, 'keyword')).toEqual([])
    // Prose like the badge labels stays plain text.
    expect(out).toContain('>Boarding<')
    expect(out).toContain('>On time<')
  })

  it('does not treat words inside attribute strings as keywords', () => {
    const out = highlight('<badge tone="in for of default">x</badge>')
    expect(tokens(out, 'keyword')).toEqual([])
    expect(tokens(out, 'string')).toEqual(['"in for of default"'])
  })
})

describe('highlight — directive headers and bindings', () => {
  it('tokenizes element control-flow attrs as bindings, never comments', () => {
    const out = highlight(sailFileSnippets.template)
    expect(tokens(out, 'comment')).toEqual([])
    // Element tags are markup; brace attrs on <for>/<match>/<case> are bindings.
    expect(tokens(out, 'bind')).toEqual(expect.arrayContaining([
      '{flights()}',
      '{f.id}',
      '{f.status}',
      "{'boarding'}",
    ]))
    expect(out).toContain('<for as="f"')
    expect(out).toContain('<match on=')
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
  const out = highlight(sailFileSnippets.controller)

  it('never emits bind spans inside code (nested arrow bodies stay intact)', () => {
    expect(tokens(out, 'bind')).toEqual([])
  })

  it('never emits comment spans (nothing in the snippet is a comment)', () => {
    expect(tokens(out, 'comment')).toEqual([])
  })

  it('keywords, verbs, strings and numbers get their classes', () => {
    const kws = tokens(out, 'keyword')
    for (const kw of ['import', 'from', 'export', 'default', 'const', 'if', 'return']) {
      expect(kws).toContain(kw)
    }
    const fns = tokens(out, 'fn')
    for (const v of ['controller', 'signal', 'effect']) expect(fns).toContain(v)
    expect(tokens(out, 'string')).toContain("'@ship/sail-runtime'")
    expect(tokens(out, 'number')).toContain('2400')
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
    // Prelude is code…
    expect(tokens(out, 'keyword')).toContain('const')
    expect(tokens(out, 'fn')).toContain('signal')
    // …and the template half still gets tag/bind coloring.
    expect(tokens(out, 'tag')).toEqual(expect.arrayContaining(['<column', '<badge', '</badge']))
    expect(tokens(out, 'bind')).toEqual(["{playing() ? 'playing' : 'paused'}"])
    // Generics never trigger the handoff.
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
  it('round-trips every real sail-file snippet to its source', () => {
    for (const src of Object.values(sailFileSnippets)) {
      expect(plainText(highlight(src))).toBe(src)
    }
  })

  it('escapes HTML so output cannot inject', () => {
    const out = highlight('<p>a < b && c > d</p>')
    expect(out).not.toMatch(/<(?!\/?span|span)[^>]*onerror/i)
    expect(plainText(out)).toBe('<p>a < b && c > d</p>')
  })
})
