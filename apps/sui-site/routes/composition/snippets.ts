// Code snippets for the Composition page. Kept OUT of the controller so the
// compiler's textual scope scan isn't fooled by the sample strings. The .sui
// samples are compile-checked against the in-repo sui-compiler (slots as a
// component + consumer pair; props; event modifiers).
export const code = {
  slots: `<!-- app-card.sui — declare named slots and a default slot -->
<column sx="p:4 gap:2 radius:lg bg:surface-1 border:border">
  <slot name="title"></slot>
  <slot></slot>
  <row justify="end"><slot name="actions"></slot></row>
</column>`,

  consumer: `<!-- consumer — fill slots by name; loose children fall into the default -->
<app-card>
  <h2 slot="title">A new project</h2>
  <p>Body content goes in the default slot.</p>
  <button slot="actions" primary>Save</button>
</app-card>`,

  props: `<!-- pass props down; the child declares them with <prop> and reads
     them as signals -->
<chip tone="primary" count={unread()}></chip>`,

  events: `<!-- on:<event> takes modifiers: |prevent, |stop, |self, |debounce:<ms> -->
<form on:submit|prevent={save}>
  <input bind:value={draft} />
  <input on:input|debounce:200={onType} />
  <button on:click|stop={commit}>Save</button>
</form>`,
}
