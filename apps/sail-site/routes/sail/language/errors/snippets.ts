// Snippets for the Errors chapter. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by patterns inside these strings.
// Compile-checked by crates/sail/compiler/tests/site_snippets_compile.rs;
// the demo snippet is the source of the live demo beside it.
export const code = {
  demo: `<controller>
export default controller(() => {
  const armed = signal(false)
  return {
    armed,
    arm: () => armed.set(true),
    disarm: () => armed.set(false),
    // called from the template — throws while the branch renders
    boom: (): string => { throw new Error('the demo component threw') },
  }
})
</controller>

<error-boundary on:sail-error-reset={disarm}>
  <if {armed()}><p>{boom()}</p></if>
  <if {!armed()}>
    <p>This subtree is protected.</p>
    <button on:click={arm} tone="destructive" variant="outline">
      Throw during render
    </button>
  </if>
</error-boundary>`,

  fallback: `<!-- message= sets the built-in fallback's text; slot="fallback"
     replaces the whole fallback card with your own markup. -->
<error-boundary message="Couldn't draw the chart.">
  <column>…the risky subtree…</column>
  <column slot="fallback" gap="2">
    <p>Couldn't draw the chart. The rest of the page is fine.</p>
  </column>
</error-boundary>`,

  async: `<!-- async failure is a STATE, not a throw — render it -->
<while-error {users} as="err">
  <alert tone="destructive">{err.message}</alert>
</while-error>`,
}
