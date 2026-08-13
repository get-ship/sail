// Snippets for /docs/concepts/styling. Sail samples are compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs); the two diagnostic
// samples are real compiler output, captured verbatim.
import { signal, derived } from '@ship/sail-runtime'
export const code = {
  sx: `<card style({ box: space({ pad: 5 }), paint: fill({ color: surface-1 }).border({ color: border, width: 1 }).radius(lg).elevation(sm) })>
  <p style({ paint: ink({ color: muted-foreground }), text: type({ size: sm }) })>
    On the system, or it does not compile.</p>
</card>`,

  sxError: `error: [sail-style] Style error: style \`radius(huge)\` is retired — use \`radius(md) or radius({ all: md })\``,

  current: `<row gap="1" align="center" style({ box: space({ pad: 1 }), paint: fill({ color: surface-2 }).border({ color: border, width: 1 }).radius(full) })>
  <a href="/docs" current={onDocs() ? 'page' : null} style({ box: space({ pad-x: 3, pad-y: 1 }), paint: radius(full), text: type({ size: sm }), when: { current: { paint: fill({ color: primary }).ink({ color: primary-foreground }) } } })>
    Docs</a>
  <a href="/blog" current={onDocs() ? null : 'page'} style({ box: space({ pad-x: 3, pad-y: 1 }), paint: radius(full), text: type({ size: sm }), when: { current: { paint: fill({ color: primary }).ink({ color: primary-foreground }) } } })>
    Blog</a>
</row>`,

  conditions: `<column gap="2" style({ box: space({ pad: 3 }), paint: fill({ color: surface-1 }).border({ color: border, width: 1 }).radius(lg), when: { tablet: { box: space({ pad: 6 }) } } })>
  <p style({ text: type({ size: sm }) })>
    space({ pad: 3 }) on a phone; when: { tablet: { box: space({ pad: 6 }) } } from 768px up.</p>
</column>`,

  states: `<button style({ when: { hover: { paint: fill({ color: muted }) } } })>
    hover → fill({ color: muted })</button>
<button style({ when: { hover: { paint: ink({ color: primary }) } } })>
  hover → ink({ color: primary })</button>
<button style({ when: { active: { paint: fill({ color: primary }).ink({ color: primary-foreground }) } } })>
  active:…</button>`,

  variants: `<scope>
  props: {
    label = ''
    emphasis: 'solid' | 'soft' | 'outline' = 'solid'
  }

  style base = { box: space({ pad-x: 3, pad-y: 1 }), paint: radius(md) }
  style solid = { paint: fill({ color: primary }).ink({ color: primary-foreground }) }
  style soft = { paint: fill({ color: muted }).ink({ color: foreground }) }
  style outline = { paint: border({ color: border, width: 1 }) }
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
<card style({ box: size(9), paint: fill({ color: primary }).radius(md).motion({ x: bind(shiftPx) }) })>
</card>`,

  layout: `<row gap="3" align="center">...</row>
<column gap="2">...</column>
<grid cols="3" gap="4">...</grid>`,
}
