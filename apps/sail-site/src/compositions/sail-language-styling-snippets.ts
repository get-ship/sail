// Snippets for /docs/concepts/styling. Sail samples are compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs); the two diagnostic
// samples are real compiler output, captured verbatim.
import { signal, derived } from '@ship/sail-runtime'
export const code = {
  sx: `<card style({ pad(5).bg(surface-1).border({ color: border, width: 1 }).radius(lg).elevation(sm) })>
  <p style({ fg(muted-foreground).type({ size: sm }) })>
    On the system, or it does not compile.</p>
</card>`,

  sxError: `error: [spinnaker] Style error: style \`radius(huge)\` is retired — use \`radius(md) or radius({ all: md })\``,

  current: `<row gap="1" align="center" style({ pad(1).bg(surface-2).border({ color: border, width: 1 }).radius(full) })>
  <a href="/docs" current={onDocs() ? 'page' : null} style({ pad({ x: 3, y: 1 }).radius(full).type({ size: sm }), when: { current: bg(primary).fg(primary-foreground) } })>
    Docs</a>
  <a href="/blog" current={onDocs() ? null : 'page'} style({ pad({ x: 3, y: 1 }).radius(full).type({ size: sm }), when: { current: bg(primary).fg(primary-foreground) } })>
    Blog</a>
</row>`,

  conditions: `<column gap="2" style({ pad(3).bg(surface-1).border({ color: border, width: 1 }).radius(lg), when: { tablet: pad(6) } })>
  <p style({ type({ size: sm }) })>
    pad(3) on a phone; when tablet → pad(6) from 768px up.</p>
</column>`,

  states: `<button style({ when: { hover: bg(muted) } })>
    hover → bg(muted)</button>
<button style({ when: { hover: fg(primary) } })>
  hover → fg(primary)</button>
<button style({ when: { active: bg(primary).fg(primary-foreground) } })>
  active:…</button>`,

  variants: `<scope>
  props: {
    label = ''
    emphasis: 'solid' | 'soft' | 'outline' = 'solid'
  }

  style base = { pad({ x: 3, y: 1 }).radius(md) }
  style solid = { bg(primary).fg(primary-foreground) }
  style soft = { bg(muted).fg(foreground) }
  style outline = { border({ color: border, width: 1 }) }
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
<card style({ size(9).bg(primary).radius(md).translateX(bind(shiftPx)) })>
</card>`,

  layout: `<row gap="3" align="center">...</row>
<column gap="2">...</column>
<grid cols="3" gap="4">...</grid>`,
}
