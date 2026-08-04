// L1 conformance spec: the PROSE surface (mechanism-2, native + skin, zero JS).
//
// The largest untested claim in the system: "write plain <h1>/<p>/<blockquote> and
// get taste, with no sx and no <style>." That claim is what makes the `strictest`
// styling policy livable — text has a home (the tags themselves) instead of needing
// a styling escape hatch. It is also every one of Mantine's nine Typography
// components (Title, Text, List, Blockquote, Code, Mark, Highlight, Table,
// Typography) delivered as zero tags and zero JS.
//
// Nothing here is authored by the user. If the type scale breaks, every page breaks
// quietly — so it gets gated like everything else.

export default {
  tag: 'prose',
  width: 720,
  rows() {
    return [
      `<div class="l1-prose" style="max-width:640px">
        <h1>Heading one</h1>
        <h2>Heading two</h2>
        <h3>Heading three</h3>
        <h4>Heading four</h4>
        <p>Body copy sets the baseline — <strong>strong</strong>, <em>emphasis</em>,
           <a href="#x">a link</a>, <code>inline_code()</code>, <mark>a highlight</mark>,
           <abbr title="Cascading Style Sheets">CSS</abbr>, and <small>small print</small>.</p>
        <blockquote>A quotation carries its own rule.</blockquote>
        <ul><li>Unordered item</li><li>Another item</li></ul>
        <ol><li>Ordered item</li><li>Another item</li></ol>
        <dl><dt>Term</dt><dd>Definition of the term.</dd></dl>
        <pre><code>const x = 1</code></pre>
        <p>Press <kbd>Ctrl</kbd> + <kbd>K</kbd>.</p>
      </div>`,
    ]
  },
  probe() {
    const bad = []
    const scope = "[theme='ship'] .l1-prose "
    const q = (s) => document.querySelector(scope + s)
    const cs = (s, p) => getComputedStyle(q(s))[p]
    const px = (s, p) => parseFloat(cs(s, p))
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    // nothing may be invisible or unstyled-by-accident
    for (const t of ['h1', 'h2', 'h3', 'h4', 'p', 'blockquote', 'ul', 'ol', 'dl', 'pre', 'kbd', 'code', 'mark']) {
      const el = q(t)
      if (!el) {
        bad.push({ why: 'missing', t })
        continue
      }
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ why: 'not-rendered', t })
    }

    // 1. THE TYPE SCALE. Strictly descending — this is the spine of the whole claim.
    const sizes = ['h1', 'h2', 'h3', 'h4', 'p'].map((t) => px(t, 'fontSize'))
    for (let i = 0; i < sizes.length - 1; i++) {
      if (!(sizes[i] > sizes[i + 1]))
        bad.push({ why: 'type-scale-not-descending', at: ['h1', 'h2', 'h3', 'h4', 'p'][i], sizes })
    }
    // headings carry weight, body does not
    for (const h of ['h1', 'h2', 'h3']) {
      if (!(parseInt(cs(h, 'fontWeight'), 10) > parseInt(cs('p', 'fontWeight'), 10)))
        bad.push({ why: 'heading-not-weighted', h })
    }
    // body copy needs a readable measure — leading well above 1.0
    if (!(px('p', 'lineHeight') / px('p', 'fontSize') > 1.35))
      bad.push({ why: 'body-leading-too-tight', ratio: px('p', 'lineHeight') / px('p', 'fontSize') })

    // 2. CODE IS MONOSPACE. Three tags, one job — and they must not be the body font.
    for (const t of ['code', 'pre', 'kbd']) {
      const f = cs(t, 'fontFamily').toLowerCase()
      if (!f.includes('mono')) bad.push({ why: 'code-not-monospace', t, f })
    }
    // and code must be visually set apart from prose, not just re-fonted
    if (clear(cs('code', 'backgroundColor')) && px('code', 'borderTopWidth') === 0)
      bad.push({ why: 'inline-code-not-set-apart' })
    // a <pre> block scrolls rather than blowing out the page
    if (!['auto', 'scroll'].includes(cs('pre', 'overflowX')))
      bad.push({ why: 'pre-does-not-scroll', overflowX: cs('pre', 'overflowX') })

    // 3. THE MARKED-UP INLINES each do their one thing.
    if (clear(cs('mark', 'backgroundColor'))) bad.push({ why: 'mark-not-highlighted' })
    if (parseInt(cs('strong', 'fontWeight'), 10) <= parseInt(cs('p', 'fontWeight'), 10))
      bad.push({ why: 'strong-not-bold' })
    if (cs('em', 'fontStyle') !== 'italic') bad.push({ why: 'em-not-italic' })
    if (cs('abbr', 'textDecorationLine') === 'none') bad.push({ why: 'abbr-not-marked' })
    if (!(px('small', 'fontSize') < px('p', 'fontSize'))) bad.push({ why: 'small-not-smaller' })
    // a link must be distinguishable from body text WITHOUT relying on color alone
    if (cs('a', 'color') === cs('p', 'color') && cs('a', 'textDecorationLine') === 'none')
      bad.push({ why: 'link-indistinguishable' })

    // 4. BLOCKQUOTE carries its own rule (the border IS the quote mark).
    if (px('blockquote', 'borderLeftWidth') === 0 || clear(cs('blockquote', 'borderLeftColor')))
      bad.push({ why: 'blockquote-has-no-rule' })

    // 5. LISTS keep their markers — the classic reset casualty.
    for (const t of ['ul', 'ol']) {
      if (cs(t, 'listStyleType') === 'none') bad.push({ why: 'list-markers-stripped', t })
      // and they must be indented, or the marker sits outside the text column
      if (px(t, 'paddingLeft') === 0 && px(t, 'marginLeft') === 0)
        bad.push({ why: 'list-not-indented', t })
    }
    if (parseInt(cs('dt', 'fontWeight'), 10) <= parseInt(cs('dd', 'fontWeight'), 10))
      bad.push({ why: 'dt-not-distinct-from-dd' })

    // 6. FROZEN-TOKEN GUARD — the prose surface must actually re-theme.
    const at = (t, sel, p) =>
      getComputedStyle(document.querySelector(`[theme='${t}'] .l1-prose ${sel}`))[p]
    if (at('ship', 'h1', 'color') === at('console', 'h1', 'color'))
      bad.push({ why: 'frozen-token', el: 'h1', light: at('ship', 'h1', 'color') })
    if (at('ship', 'code', 'backgroundColor') === at('console', 'code', 'backgroundColor'))
      bad.push({ why: 'frozen-token', el: 'code' })

    return { count: document.querySelectorAll('.l1-prose').length, bad: bad.slice(0, 12) }
  },
}
