// Snippets for /docs/concepts/styling. Sail samples are compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs); the two diagnostic
// samples are real compiler output, captured verbatim.
import { signal, derived } from '@ship/sail-runtime'
export const code = {
  sx: `<card style({ box: pad(5), paint: bg(surface-1).borderWidth(1).border(border).radius(lg).shadow(sm) })>
  <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>
    On the system, or it does not compile.</p>
</card>`,

  sxError: `error: [sail-style] unknown Style token on setter \`radius\`: huge
  \`radius\` accepts scale tokens (none, sm, md, lg, xl, full, …); did you mean \`full\`?`,

  current: `<row gap="1" align="center" style({ box: pad(1), paint: bg(surface-2).borderWidth(1).border(border).radius(full) })>
  <a href="/docs" current={onDocs() ? 'page' : null} style({ box: padX(3).padY(1), paint: radius(full), text: fontSize(sm), when: { current: { paint: bg(primary).fg(primary-foreground) } } })>
    Docs</a>
  <a href="/blog" current={onDocs() ? null : 'page'} style({ box: padX(3).padY(1), paint: radius(full), text: fontSize(sm), when: { current: { paint: bg(primary).fg(primary-foreground) } } })>
    Blog</a>
</row>`,

  conditions: `<column gap="2" style({ box: pad(3), paint: bg(surface-1).borderWidth(1).border(border).radius(lg), when: { tablet: { box: pad(6) } } })>
  <p style({ text: fontSize(sm) })>
    pad(3) on a phone; when: { tablet: { box: pad(6) } } from 768px up.</p>
</column>`,

  states: `<button style({ when: { hover: { paint: bg(muted) } } })>
    hover → bg(muted)</button>
<button style({ when: { hover: { paint: fg(primary) } } })>
  hover → fg(primary)</button>
<button style({ when: { active: { paint: bg(primary).fg(primary-foreground) } } })>
  active:…</button>`,

  variants: `<scope>
  props: {
    label = ''
    emphasis: 'solid' | 'soft' | 'outline' = 'solid'
  }

  style base = { box: padX(3).padY(1), paint: radius(md) }
  style solid = { paint: bg(primary).fg(primary-foreground) }
  style soft = { paint: bg(muted).fg(foreground) }
  style outline = { paint: borderWidth(1).border(border) }
</scope>

<button style(base, {
    match: {
      emphasis: {
        solid: solid,
        soft: soft,
        outline: outline,
      },
    },
  })>
  {label()}</button>`,

  variantUse: `<my-button label="Save" emphasis="outline"></my-button>`,

  dynamic: `<controller>
export default controller(() => {
  const shift = signal("0")
  const shiftPx = derived((): number => Number(shift()) || 0)
  return { shift, shiftPx }
})
</controller>

<slider min="0" max="200" bind:value={shift} />
<card style({ box: w(9).h(9), paint: bg(primary).radius(md).translateX(bind(shiftPx)) })>
</card>`,

  layout: `<row gap="3" align="center">...</row>
<column gap="2">...</column>
<grid cols="3" gap="4">...</grid>`,
}
