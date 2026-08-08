/** Minimal markdown → HTML for sail-markdown (headings, lists, code, links). */

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
  out = out.replace(
    /\[([^\]]+)\]\(([^)\s]+)\)/g,
    (_, t, h) => `<a href="${h}" rel="noopener noreferrer">${t}</a>`,
  )
  return out
}

export function renderMarkdown(src: string): string {
  const lines = src.replace(/\r\n/g, '\n').split('\n')
  // noUncheckedIndexedAccess: wrap reads so parse sites stay focused.
  const cur = (): string => lines[i] ?? ''
  const out: string[] = []
  let i = 0
  while (i < lines.length) {
    const line = cur()
    if (/^```/.test(line)) {
      const lang = line.slice(3).trim()
      const buf: string[] = []
      i++
      while (i < lines.length && !/^```/.test(cur())) {
        buf.push(cur())
        i++
      }
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
      while (i < lines.length && /^>\s?/.test(cur())) {
        buf.push(cur().replace(/^>\s?/, ''))
        i++
      }
      out.push(`<blockquote>${inline(buf.join(' '))}</blockquote>`)
      continue
    }
    if (line.trim() === '') {
      i++
      continue
    }
    const buf: string[] = []
    while (
      i < lines.length &&
      cur().trim() !== '' &&
      !/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(cur())
    ) {
      buf.push(cur())
      i++
    }
    out.push(`<p>${inline(buf.join(' '))}</p>`)
  }
  return out.join('\n')
}
