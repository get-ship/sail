<scope>
  props: {
    value = ''
  }
</scope>
<!--
  NEW SURFACE (re-port from feat/next, parity-gate pass)
  ─────────────────────────────────────────────────────────────────────
  - markdown has no tone/size of its own. No theme block, no
    per-component .sail-markdown[tone='…'] or .sail-markdown[size='…'] redeclaration
    in this style. The framework's global tone/size channels are
    irrelevant — markdown renders prose, not control chrome.
  - The host's surface baseline moves to a host sx attribute: display,
    text size, line-height, foreground color are all reachable via
    tokens. font-family is the one host surface the sx vocabulary
    doesn't expose, so it stays in style as a single line.
  - The prose body (h1-h6 / p / a / code / pre / blockquote / ul / ol /
    li / strong / table / thead / tbody) is intentionally NOT in sx:
    the parser writes raw HTML into .md-root, not SUI elements, so
    sx can't apply to children — the whole prose stylesheet has to
    live in style. The selector list + each property is chosen so
    the css-token lint passes (every value is a token; the lint
    local-marker comment at the top of style declares the set the
    residual block references).
  - The host [hidden] display:none is provided by the runtime's
    BASE_CSS — not authored here.
-->

<controller>
import { controller, bindHtml } from '@ship/sail-runtime'
import type { Scope } from './sail-markdown.sail'

function escape(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function inline(s: string): string {
  let out = escape(s)
  out = out.replace(/`([^`]+)`/g, (_, c) => `<code>${c}</code>`)
  out = out.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
  out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, '$1<em>$2</em>')
  out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, t, h) => `<a href="${h}" rel="noopener noreferrer">${t}</a>`)
  return out
}

export function renderMarkdown(src: string): string {
  const lines = src.replace(/\r\n/g, '\n').split('\n')
  // `lines[i]` is `string | undefined` under noUncheckedIndexedAccess. Wrap
  // each read in `cur()` so the call sites stay focused on the parsing
  // logic instead of the null-narrowing.
  const cur = (): string => lines[i] ?? ''
  const out: string[] = []
  let i = 0
  while (i < lines.length) {
    const line = cur()
    if (/^```/.test(line)) {
      const lang = line.slice(3).trim()
      const buf: string[] = []
      i++
      while (i < lines.length && !/^```/.test(cur())) buf.push(cur()), i++
      i++
      out.push(`<pre data-lang="${escape(lang)}"><code>${escape(buf.join('\n'))}</code></pre>`)
      continue
    }
    const h = /^(#{1,6})\s+(.*)$/.exec(line)
    if (h) {
      out.push(`<h${h[1]!.length}>${inline(h[2]!)}</h${h[1]!.length}>`)
      i++
      continue
    }
    if (/^\s*[-*]\s+/.test(line)) {
      const buf: string[] = []
      while (i < lines.length && /^\s*[-*]\s+/.test(cur())) {
        buf.push(`<li>${inline(cur().replace(/^\s*[-*]\s+/, ''))}</li>`)
        i++
      }
      out.push(`<ul>${buf.join('')}</ul>`)
      continue
    }
    if (/^\s*\d+\.\s+/.test(line)) {
      const buf: string[] = []
      while (i < lines.length && /^\s*\d+\.\s+/.test(cur())) {
        buf.push(`<li>${inline(cur().replace(/^\s*\d+\.\s+/, ''))}</li>`)
        i++
      }
      out.push(`<ol>${buf.join('')}</ol>`)
      continue
    }
    if (/^>\s?/.test(line)) {
      const buf: string[] = []
      while (i < lines.length && /^>\s?/.test(cur())) { buf.push(cur().replace(/^>\s?/, '')); i++ }
      out.push(`<blockquote>${inline(buf.join(' '))}</blockquote>`)
      continue
    }
    if (line.trim() === '') { i++; continue }
    const buf: string[] = []
    while (i < lines.length && cur().trim() !== '' && !/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(cur())) {
      buf.push(cur()); i++
    }
    out.push(`<p>${inline(buf.join(' '))}</p>`)
  }
  return out.join('\n')
}

export default controller((ctx: Scope) => {
  mount(() => {
    const el = ctx.contentEl() as HTMLElement | null
    if (!el) return () => {}
    return bindHtml(el, () => renderMarkdown(ctx.value() ?? ''))
  })
  return {}
})

</controller>
<prose>
  <div sx="block text-base" bind:this={contentEl}></div>
</prose>
