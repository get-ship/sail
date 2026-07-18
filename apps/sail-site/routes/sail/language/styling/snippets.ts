// Snippets for /sail/language/styling. SAIL samples are compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs); the two diagnostic
// samples are real compiler output, captured verbatim.
export const code = {
  sx: `<card sx="p-5 bg-surface-1 border border-border rounded-lg shadow-sm">
  <p sx="text-muted-foreground text-sm">On the system, or it does not compile.</p>
</card>`,

  sxError: `error: [sail-sx] sx \`rounded-huge\` — \`huge\` is not a token.
  \`rounded-\` accepts: full, lg, md, none, sm, xl; did you mean \`rounded-full\`?`,

  states: `<button sx="bg-surface-1 hover:bg-muted focus:ring-ring">Save</button>`,

  current: `<row gap="1" align="center" sx="p-1 bg-surface-2 border border-border rounded-full">
  <a href="/docs" sx="px-3 py-1 rounded-full text-sm current:bg-primary current:text-primary-foreground" aria-current={onDocs() ? 'page' : null}>Docs</a>
  <a href="/blog" sx="px-3 py-1 rounded-full text-sm current:bg-primary current:text-primary-foreground" aria-current={onDocs() ? null : 'page'}>Blog</a>
</row>`,

  conditions: `<column gap="2" sx="p-4 tablet:p-6 dark:bg-surface-2">
  <p sx="text-sm">Padding grows at 768px; the panel darkens with the OS theme.</p>
</column>`,

  prefixError: `error: [sail-sx] sx unknown prefix \`bogus:\` — the closed set:
  hover:/focus:/focus-within:/active:/disabled:/current:/selected:/expanded:/
  busy:/checked:/pressed:/open:/sortable:, a condition (mobile:/tablet:/desktop:/
  motion-reduce:/motion-safe:/forced-colors:/dark:/light:), \`state-<value>:\`
  for a component state, or group-<state>: to react to an ancestor`,

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
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => {
  const x = signal(0)
  return { x }
})
</controller>

<card sx="w-9 h-9 translate-x-(--x)"></card>`,

  layout: `<row gap="3" align="center">...</row>
<column gap="2">...</column>
<grid cols="3" gap="4">...</grid>`,
}
