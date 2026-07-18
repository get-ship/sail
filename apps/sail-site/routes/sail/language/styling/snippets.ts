// Snippets for /sail/language/styling. Sail samples are compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs); the two diagnostic
// samples are real compiler output, captured verbatim.
export const code = {
  sx: `<card sx="p-5 bg-surface-1 border border-border rounded-lg shadow-sm">
  <p sx="text-muted-foreground text-sm">On the system, or it does not compile.</p>
</card>`,

  sxError: `error: [sail-sx] sx \`rounded-huge\` — \`huge\` is not a token.
  \`rounded-\` accepts: full, lg, md, none, sm, xl; did you mean \`rounded-full\`?`,

  current: `<row gap="1" align="center" sx="p-1 bg-surface-2 border border-border rounded-full">
  <a href="/docs" sx="px-3 py-1 rounded-full text-sm current:bg-primary current:text-primary-foreground" aria-current={onDocs() ? 'page' : null}>Docs</a>
  <a href="/blog" sx="px-3 py-1 rounded-full text-sm current:bg-primary current:text-primary-foreground" aria-current={onDocs() ? null : 'page'}>Blog</a>
</row>`,

  conditions: `<column gap="2" sx="p-3 tablet:p-6 bg-surface-1 border border-border rounded-lg">
  <p sx="text-sm">p-3 on a phone, tablet:p-6 from 768px up.</p>
</column>`,

  states: `<button sx="hover:bg-muted">hover:bg-muted</button>
<button sx="hover:text-primary">hover:text-primary</button>
<button sx="active:bg-primary active:text-primary-foreground">active:…</button>`,

  tones: `<alert tone="destructive">
  <p sx="text-tone-fg">Could not save.</p>
</alert>`,

  variants: `<scope>
  props: {
    label = ''
  }
  variants: {
    emphasis: {
      solid: 'bg-primary text-primary-foreground',
      soft: 'bg-muted text-foreground',
      outline: 'border border-border'
    }
  }
</scope>

<button sx="px-3 py-1 rounded-md">{label()}</button>`,

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
<card sx="w-9 h-9 bg-primary rounded-md translate-x-(--px)"></card>`,

  layout: `<row gap="3" align="center">...</row>
<column gap="2">...</column>
<grid cols="3" gap="4">...</grid>`,
}
