// Code snippets for the Forms guide. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by patterns inside these strings.
// Every snippet here IS the live demo's code (or a focused slice of it);
// the Sail-shaped ones are compile-checked by site_snippets_compile.rs.
import { signal, mutation, form } from '../vendor/runtime'
export const code = {
  demo: `<controller>
import { profileApi } from '../../../../src/compositions/profile-api'

export default controller(() => {
  const sabotage = signal(false)      // "fail the next save" toggle
  const tried = signal(false)         // has the reader tried to submit yet?
  const saved = signal<{ name: string; email: string } | null>(null)

  const profile = form({
    initial: { name: '', email: '' },
    validate: ({ name, email }) => ({
      name: name.trim() === '' ? 'a display name is required' : null,
      email: email.includes('@') ? null : 'must include an @',
    }),
    submit: async (values) => {
      saved.set(null)
      await profileApi.save(values, sabotage())
      sabotage.set(false)
      saved.set(values)
    },
  })

  return {
    profile, tried, saved,
    onSubmit: (e: Event) => { tried.set(true); void profile.submit(e) },
    onReset: () => { tried.set(false); saved.set(null); profile.reset() },
    breakIt: () => sabotage.set(true),
  }
})
</controller>

<form on:submit={onSubmit}>
  <field invalid={tried() && !!profile.errors().name}>
    <label slot="label" for="profile-name">Display name</label>
    <input id="profile-name" bind:value={profile.fields.name} />
    <span slot="hint">Public — shows on your profile.</span>
    <span slot="error">{profile.errors().name}</span>
  </field>

  <field invalid={tried() && !!profile.errors().email}>
    <label slot="label" for="profile-email">Email</label>
    <input id="profile-email" bind:value={profile.fields.email} />
    <span slot="hint">We never send anything to it.</span>
    <span slot="error">{profile.errors().email}</span>
  </field>

  <row gap="2">
    <button type="submit" disabled={profile.pending()}>
      {profile.pending() ? 'Saving…' : 'Save'}
    </button>
    <button type="button" variant="ghost" on:click={onReset}>Reset</button>
  </row>

  <if {profile.lastError()}>
    <alert tone="destructive">{profile.lastError().message}</alert>
  </if>
  <if {saved()}>
    <alert tone="success">Saved — {saved().name} ({saved().email}).</alert>
  </if>
</form>`,

  // form is not ambient; controller is.
  call: `

export default controller(() => {
  const profile = form({
    // One key per field. A function is re-read on reset(), so a form can
    // reset to CURRENT server state instead of a stale snapshot.
    initial: { name: '', email: '' },

    // Return an error message per field, or null for a field that is fine.
    validate: ({ name, email }) => ({
      name: name.trim() === '' ? 'a display name is required' : null,
      email: email.includes('@') ? null : 'must include an @',
    }),

    // Called by profile.submit() — only when every field validates.
    submit: async (values) => {
      await profileApi.save(values)
    },
  })
  return { profile }
})`,

  bind: `<field invalid={tried() && !!profile.errors().email}>
  <label slot="label" for="profile-email">Email</label>
  <input id="profile-email" bind:value={profile.fields.email} />
  <span slot="hint">We never send anything to it.</span>
  <span slot="error">{profile.errors().email}</span>
</field>`,

  submit: `<form on:submit={onSubmit}>
  <!-- fields … -->
  <button type="submit" disabled={profile.pending()}>
    {profile.pending() ? 'Saving…' : 'Save'}
  </button>

  <if {profile.lastError()}>
    <alert tone="destructive">{profile.lastError().message}</alert>
  </if>
  <if {saved()}>
    <alert tone="success">Saved — {saved().name} ({saved().email}).</alert>
  </if>
</form>`,

  reset: `<button type="button" on:click={onReset}>Reset</button>
<!-- profile.dirty() is true whenever any field differs from the baseline -->
<if {profile.dirty()}>
  <p>Unsaved changes.</p>
</if>`,

  mutation: `<!-- In a deck app with a typed ship client in scope: -->
<form mutation="projects.create" on-success={() => close()}>
  <input name="title" />
  <button type="submit">Create</button>

  <if {$mutation().kind === 'loading'}><spinner></spinner></if>
  <if {$mutation().kind === 'error'}>
    <alert tone="destructive">{$mutation().error.message}</alert>
  </if>
</form>`,
}
