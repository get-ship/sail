// Code snippets for the styling page. Every SUI sample is compiled by the build
// (crates/sui/compiler/tests/site_snippets_compile.rs).
export const code = {
  sx: `<!-- sx is a closed vocabulary of design-system atoms.
     A value that is not on the system is a BUILD ERROR, not a review note. -->
<card sx="p:5 bg:surface-1 border:border radius:lg shadow:sm">
  <p sx="fg:muted-foreground text:sm">On the system, or it does not compile.</p>
</card>`,

  states: `<!-- states are a prefix, not a stylesheet -->
<button sx="bg:surface-1 hover:bg:muted focus-visible:ring:ring">Save</button>`,

  tones: `<!-- a TONE is a contract. tone="destructive" resolves --tone-bg,
     --tone-fg, --tone-border and --tone-ring for the whole subtree — so a
     component reads the tokens instead of choosing a red. -->
<alert tone="destructive">
  <p sx="fg:tone-fg">Could not save.</p>
</alert>`,

  layout: `<!-- structure is NOT sx. The axis IS the name. -->
<row gap="3" align="center">…</row>
<column gap="2">…</column>
<grid cols="3" gap="4">…</grid>`,

  sides: `<!-- a border on one side: a topbar's rule, a rail's edge -->
<navbar sx="border-b:border">…</navbar>`,
}
