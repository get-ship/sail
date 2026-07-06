// Code snippets for the Composition page. Kept OUT of the controller so the
// compiler's textual scope scan isn't fooled by the sample strings. The .sui
// samples are compile-checked against the in-repo sui-compiler (slots as a
// component + consumer pair; props; event modifiers).
export const code = {
  slots: `<!-- card.sui — declare named slots and a default slot -->
<article class="card">
  <header><slot name="title"></slot></header>
  <div class="body"><slot></slot></div>
  <footer><slot name="actions"></slot></footer>
</article>`,

  consumer: `<!-- consumer — fill slots by name; loose children fall into the default -->
<sui-card>
  <h2 slot="title">A new project</h2>
  <p>Body content goes in the default slot.</p>
  <button slot="actions" primary>Save</button>
</sui-card>`,

  props: `<!-- pass props down; the child declares them with <prop> and reads
     them as signals -->
<sui-badge tone="primary" count={unread()}></sui-badge>`,

  events: `<!-- on:<event> takes modifiers: |prevent, |stop, |self, |debounce:<ms> -->
<form on:submit|prevent={save}>
  <input bind:value={draft} />
  <input on:input|debounce:200={onType} />
  <button on:click|stop={commit}>Save</button>
</form>`,
}
