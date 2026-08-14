// Code snippets for getting-started. Sail samples are compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs).
import { signal } from '@ship/sail-runtime'
export const code = {
  toolchain: `# Alpha: today the toolchain builds from source, once.
# You need Bun and a Rust toolchain — for now.
git clone https://github.com/get-ship/sail && cd sail
bun install                              # JS workspace
cargo build --release -p deck-cli        # the sail binary (compiler included)`,

  app: `my-app/
└─ routes/
   └─ page.sail        # the / route

$ sail dev --project my-app
  ▸ serving http://127.0.0.1:7878`,

  firstPage: `<!-- routes/page.sail — a page is markup. The components are
     already in the language; there is nothing to import. -->
<column gap="4" style({ pad(6).maxW(panel-max-w-lg) })>
  <h1>Team</h1>
  <row gap="2">
    <input placeholder="Filter people" />
    <button variant="solid" tone="primary">Invite</button>
  </row>
  <card style({ pad(4) })>
    <row gap="3" align="center">
      <avatar></avatar>
      <strong>Ada Lovelace</strong>
      <badge tone="success">Active</badge>
    </row>
  </card>
</column>`,

  feedback: `# Write something off the system and the build says no — precisely.
$ sail dev
Error: routes/page.sail: line 4 col 13: [sail-style]
  sx \`bg:blurple\` — \`blurple\` is not a token.
  allowed: accent, background, border, muted, primary, … (the token list)
  suggest: sx is surface-only: spacing 0-8 (p/m…), colors (bg/fg),
  radius, text. Raw px/hex are not expressible.`,

  state: `<!-- routes/page.sail — state lives in a <controller> block in the
     same file. Ambient idents (controller, signal, …) need no import.
     The template binds ONLY what the controller function returns. -->
<controller>
export default controller((ctx) => {
  const count = signal(0)
  return { count }
})
</controller>

<button on:click={() => count.set(count() + 1)}>
  Clicked {count()} times
</button>`,

  routesTree: `routes/
├─ page.sail               → /
├─ settings/
│  └─ page.sail            → /settings
└─ users/
   └─ [id]/
      └─ page.sail         → /users/:id   (read it: route.params.id())`,

  build: `$ sail build --project my-app
# static output: SSR'd HTML per route + hydrating bundles.
# Deploy the directory anywhere that serves files.`,

  install: `# one package. The 48 elements are in the LANGUAGE —
# there is no component library to add on top.
npm install @ship/sail-runtime`,

  component: `<!-- greeting.sail — a component is a <scope> block + markup -->
<scope>
  props: {
    name = ''
  }
</scope>

<p>Hello, {name()}.</p>`,

  compile: `# lower the .sail to a standard custom-element module
sail-compiler build greeting/greeting.sail`,

  use: `<!-- importing the compiled module registers the element -->
<script type="module" src="/greeting.js"></script>
<my-greeting name="Ada"></my-greeting>`,

  interactive: `<controller>
export default controller((ctx) => {
  const open = signal(false)
  return { open }
})
</controller>

<button on:click={() => open.set(!open())}>Toggle</button>
<if {open()}>
  <card style({ pad(4), animate: enter({ opacity: 'fade' }) })>
    Hello.</card>
</if>`,
}
