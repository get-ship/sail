// Snippets for /sail/language/styling. Sail samples are compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs); the two diagnostic
// samples are real compiler output, captured verbatim.
export const code = {
  sx: `<card sx={style.pad(5).bg(surface-1).borderWidth(1).border(border).radius(lg).shadow(sm)}>
  <p sx={style.fg(muted-foreground).fontSize(sm)}>On the system, or it does not compile.</p>
</card>`,

  sxError: `error: [sail-sx] unknown Style token on setter \`radius\`: huge
  \`radius\` accepts scale tokens (none, sm, md, lg, xl, full, …); did you mean \`full\`?`,

  current: `<row gap="1" align="center" sx={style.pad(1).bg(surface-2).borderWidth(1).border(border).radius(full)}>
  <a href="/docs" sx={style.padX(3).padY(1).radius(full).fontSize(sm).when({ current: style.bg(primary).fg(primary-foreground) })} aria-current={onDocs() ? 'page' : null}>Docs</a>
  <a href="/blog" sx={style.padX(3).padY(1).radius(full).fontSize(sm).when({ current: style.bg(primary).fg(primary-foreground) })} aria-current={onDocs() ? null : 'page'}>Blog</a>
</row>`,

  conditions: `<column gap="2" sx={style.pad(3).bg(surface-1).borderWidth(1).border(border).radius(lg).when({ tablet: style.pad(6) })}>
  <p sx={style.fontSize(sm)}>pad(3) on a phone; when({ tablet: style.pad(6) }) from 768px up.</p>
</column>`,

  states: `<button sx={when({ hover: style.bg(muted) })}>hover → bg(muted)</button>
<button sx={when({ hover: style.fg(primary) })}>hover → fg(primary)</button>
<button sx={when({ active: style.bg(primary).fg(primary-foreground) })}>active:…</button>`,

  tones: `<alert tone="destructive">
  <p>Could not save.</p>
</alert>`,

  variants: `<scope>
  props: {
    label = ''
    emphasis: 'solid' | 'soft' | 'outline' = 'solid'
  }

  const base = style.padX(3).padY(1).radius(md)
  const solid = style.bg(primary).fg(primary-foreground)
  const soft = style.bg(muted).fg(foreground)
  const outline = style.borderWidth(1).border(border)
</scope>

<button sx={[
  base,
  match(emphasis, {
    solid: solid,
    soft: soft,
    outline: outline,
  }),
]}>{label()}</button>`,

  variantUse: `<my-button label="Save" emphasis="outline"></my-button>`,

  dynamic: `<controller>
import { controller, signal, effect } from "@ship/sail-runtime"
export default controller(() => {
  const shift = signal("0")
  const px = signal("0px")
  effect(() => px.set(shift() + "px"))
  return { shift, px }
})
</controller>

<input type="range" min="0" max="200" bind:value={shift} />
<card sx={style.w(9).h(9).bg(primary).radius(md).translateX(bind(px))}></card>`,

  layout: `<row gap="3" align="center">...</row>
<column gap="2">...</column>
<grid cols="3" gap="4">...</grid>`,
}
