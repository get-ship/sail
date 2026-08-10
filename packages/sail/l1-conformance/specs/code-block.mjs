// L1 conformance: <code-block> surface (host + pre + .tok-* skins).
// Visual matrix stamps lowered classes (action use:syntaxHighlight is runtime).
// Probe: pre paints component.code-block tokens; tok spans resolve --syntax-* ink.
const PRE =
  's-m0 s-padY4 s-padX5 s-overflowXauto s-bgcodeblock s-fgcodeblockforeground s-borderWidth1 s-borderborder s-radiuslg s-fontFamilymono s-fontSizesm s-leadingcode s-tab2 s-whitespacepre'
const block = (inner) =>
  `<div class="sail-code-block s-displayblock" data-sail-host="code-block"><pre class="${PRE}">${inner}</pre></div>`

export default {
  tag: 'code-block',
  width: 640,
  rows() {
    return [
      block(
        `<span class="tok-keyword">const</span> n = <span class="tok-fn">signal</span>(<span class="tok-number">0</span>)`,
      ),
      block(
        `<span class="tok-tag">&lt;button</span> <span class="tok-attr">variant</span>=<span class="tok-string">"solid"</span><span class="tok-tag">&gt;</span>Save<span class="tok-tag">&lt;/button&gt;</span>`,
      ),
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-code-block pre').forEach((el, i) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      if (clear(cs.backgroundColor)) bad.push({ i, why: 'no-token-bg', bg: cs.backgroundColor })
      if (clear(cs.color)) bad.push({ i, why: 'no-token-fg', fg: cs.color })
    })
    const tok = document.querySelector('.sail-code-block .tok-keyword')
    if (tok) {
      const c = getComputedStyle(tok).color
      if (clear(c)) bad.push({ why: 'tok-keyword-clear', color: c })
    } else {
      bad.push({ why: 'missing-tok-keyword' })
    }
    return {
      count: document.querySelectorAll('.sail-code-block').length,
      bad: bad.slice(0, 8),
    }
  },
}
