// Code snippets for getting-started. SUI samples are compiled by the build
// (crates/sui/compiler/tests/site_snippets_compile.rs).
export const code = {
  toolchain: `# Prerequisites: Bun and Rust. SUI's compiler and deck's server are
# Rust binaries; the runtime and tokens are JS packages. Prebuilt
# binaries are planned; today you build the toolchain from the repo:
git clone <the sui repo> && cd sui
bun install                              # JS workspace
cargo run -p deck-cli -- build-assets    # bake the framework bundles
cargo build                              # deck CLI + compiler`,

  app: `my-app/
└─ routes/
   └─ page.sui        # the / route

$ deck dev --project my-app
  ▸ serving http://127.0.0.1:7878`,

  firstPage: `<!-- routes/page.sui — a page is markup. The components are
     already in the language; there is nothing to import. -->
<column gap="4" sx="p-6 max-w-panel-max-w-lg">
  <h1>Team</h1>
  <row gap="2">
    <input placeholder="Filter people" />
    <button variant="solid" tone="primary">Invite</button>
  </row>
  <card sx="p-4">
    <row gap="3" align="center">
      <avatar></avatar>
      <strong>Ada Lovelace</strong>
      <badge tone="success">Active</badge>
    </row>
  </card>
</column>`,

  feedback: `# Write something off the system and the build says no — precisely.
$ deck dev
Error: routes/page.sui: line 4 col 13: [sui-sx]
  sx \`bg:blurple\` — \`blurple\` is not a token.
  allowed: accent, background, border, muted, primary, … (the token list)
  suggest: sx is surface-only: spacing 0-8 (p/m…), colors (bg/fg),
  radius, text. Raw px/hex are not expressible.`,

  state: `<!-- routes/page.sui — state lives in a <controller> block in the
     same file. The template binds ONLY what scope() returns. -->
<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const count = signal(0)
    return { count }
  },
})
</controller>

<button on:click={() => count.set(count() + 1)}>
  Clicked {count()} times
</button>`,

  routesTree: `routes/
├─ page.sui               → /
├─ settings/
│  └─ page.sui            → /settings
└─ users/
   └─ [id]/
      └─ page.sui         → /users/:id   (read it: route.params.id())`,

  build: `$ deck build --project my-app
# static output: SSR'd HTML per route + hydrating bundles.
# Deploy the directory anywhere that serves files.`,

  install: `# one package. The 48 elements are in the LANGUAGE —
# there is no component library to add on top.
npm install @ship/sui-runtime`,

  component: `<!-- greeting.sui — a component is a <component> block + markup -->
<component>{
  props: { name: '' },
}</component>

<p>Hello, {name()}.</p>`,

  compile: `# lower the .sui to a standard custom-element module
sui-compiler build greeting/greeting.sui`,

  use: `<!-- importing the compiled module registers the element -->
<script type="module" src="/greeting.js"></script>
<my-greeting name="Ada"></my-greeting>`,

  interactive: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const open = signal(false)
    return { open }
  },
})
</controller>

<button on:click={() => open.set(!open())}>Toggle</button>
{#if open()}
  <card motion:enter={{ opacity: 'fade' }} sx="p-4">Hello.</card>
{/if}`,
}
