<!--
  /docs/guide/forms — the write side of the data story. query() is how a Sail
  app reads; a form is how it writes. The live demo is the curriculum: every
  snippet on this page is the demo’s code.
-->

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style monoLabel = { box: mb(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style demoPanel = { box: pad(6), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style help = { box: mt(2), paint: fg(muted-foreground), text: fontSize(sm) }
  style callout = { box: pad(3).padX(4), paint: borderWidth(1).border(border).borderLWidth(3).borderL(primary).radius(md).fg(muted-foreground), text: fontSize(sm) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { profileApi, type Profile } from '../../../../src/compositions/profile-api'
import { code } from '../../../../src/compositions/sail-guide-forms-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  const sabotage = signal(false)
  const tried = signal(false)
  const saved = signal<Profile | null>(null)

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
    code, profile, tried, saved,
    onSubmit: (e: Event) => { tried.set(true); void profile.submit(e) },
    onReset: () => { tried.set(false); saved.set(null); profile.reset() },
    breakIt: () => sabotage.set(true),
  }
})
</controller>

<column gap="5" style(page)>
  <p style(eyebrow)>Guide</p>
  <h1>Forms</h1>
  <p style(lead)>
    Forms are how an app writes. <code>query()</code> turns reading async data
    into a signal with explicit loading and error states; a form is the same
    idea pointed the other way — user input, validated, sent somewhere, with
    the in-flight and failure states handled instead of forgotten. One
    function, <code>form()</code>, carries all of it.
  </p>

  <h2>A real form, live</h2>
  <grid cols="2" gap="4">
    <column>
      <p style(monoLabel)>Live — saves to a 900 ms fake API</p>
      <column gap="3" style(demoPanel)>
        <form on:submit={onSubmit} testid="forms-demo">
          <column gap="3">
            <field invalid={tried() && !!profile.errors().name}>
              <label slot="label" for="profile-name">Display name</label>
              <input id="profile-name" bind:value={profile.fields.name} testid="forms-name" />
              <span slot="hint">Public — shows on your profile.</span>
              <span slot="error">{profile.errors().name}</span>
            </field>

            <field invalid={tried() && !!profile.errors().email}>
              <label slot="label" for="profile-email">Email</label>
              <input id="profile-email" bind:value={profile.fields.email} testid="forms-email" />
              <span slot="hint">We never send anything to it.</span>
              <span slot="error">{profile.errors().email}</span>
            </field>

            <row gap="2">
              <button type="submit" disabled={profile.pending()} testid="forms-submit">
                {profile.pending() ? 'Saving…' : 'Save'}
              </button>
              <button type="button" variant="ghost" on:click={onReset}>Reset</button>
              <button type="button" variant="outline" tone="destructive" on:click={breakIt}>Fail the next save</button>
            </row>

            <if {profile.pending()}>
              <row gap="2" align="center"><spinner></spinner> <p style(mutedSm)>saving…</p></row>
            </if>
            <if {profile.lastError()}>
              <alert tone="destructive" testid="forms-error">{profile.lastError().message}</alert>
            </if>
            <if {saved()}>
              <alert tone="success" testid="forms-success">Saved — {saved().name} ({saved().email}).</alert>
            </if>
          </column>
        </form>
      </column>
      <p style(help)>
        Try it: save while empty to see the errors, fill it in and save to
        watch the pending state, and use "Fail the next save" to see the
        failure path.
      </p>
    </column>
    <column>
      <p style(monoLabel)>Source</p>
      <code-block code={code.demo}></code-block>
    </column>
  </grid>

  <h2>One call in the controller</h2>
  <p>
    <code>form()</code> takes three things: the <code>initial</code> values
    (one key per field), a <code>validate</code> function that returns an
    error message — or <code>null</code> — per field, and a
    <code>submit</code> function called only when every field validates. It
    hands back everything the template needs: a signal per field under
    <code>fields</code>, plus <code>errors()</code>, <code>valid()</code>,
    <code>dirty()</code>, <code>pending()</code>, <code>lastError()</code>,
    <code>submit()</code> and <code>reset()</code>.
  </p>
  <code-block code={code.call}></code-block>
  <p>
    Each field is its own signal — typing in one input re-renders only what
    reads that field, not the whole form. That is why <code>fields</code> is
    a bag of signals rather than one signal holding an object.
  </p>

  <h2>Bind inputs to fields</h2>
  <p>
    <code>bind:value</code> connects an input to a field signal in both
    directions: typing updates the signal, and setting the signal (say, from
    <code>reset()</code>) updates the input. Checkboxes use
    <code>bind:checked</code>. No <code>on:input</code> handler, no manual
    reads.
  </p>
  <p>
    The <code>&lt;field&gt;</code> element wraps a control with a label, a
    hint, and an error slot. It does more than lay them out: it points the
    control’s field anatomy association at whichever message is showing
    — the hint normally, the error when <code>invalid</code> is set — keeps
    semantic prop <code>invalid</code> in sync, and announces the error region
    politely to screen readers. You get all of that by filling in the slots.
  </p>
  <code-block code={code.bind}></code-block>

  <h2>Decide when errors appear</h2>
  <p>
    <code>errors()</code> recomputes from the very first keystroke — an empty
    form is, strictly, an invalid one. Showing those errors before the reader
    has done anything feels like being scolded on arrival, so the demo keeps
    a <code>tried</code> signal that flips on the first submit attempt and
    gates the <code>invalid</code> flag with it. Click save on the empty
    form: the errors appear, and from then on they clear live as you fix
    each field. When to surface errors is a product decision, so it stays in
    your hands — <code>form()</code> tells you what is wrong, you decide
    when to say it.
  </p>

  <h2>Submit: pending, failure, success</h2>
  <p>
    Put <code>profile.submit</code> (or a wrapper, as here) on the form’s
    <code>on:submit</code>. It calls <code>preventDefault()</code> itself —
    forgetting that is a full page reload, so the composition owns it. Then
    it runs the lifecycle: <code>pending()</code> is <code>true</code> while
    the async submit is in flight, and if the submit throws,
    <code>lastError()</code> holds the error until the next attempt.
  </p>
  <p>
    Success is yours to define — a toast, a redirect, a message. The demo
    keeps a <code>saved</code> signal that the submit function sets when the
    fake API resolves.
  </p>
  <code-block code={code.submit}></code-block>

  <h2>Reset and dirty</h2>
  <p>
    <code>reset()</code> reverts every field to the initial values and clears
    the last error. If <code>initial</code> was a function, it is re-read —
    so a form seeded from server state resets to <em>current</em> server
    state, not a stale snapshot. <code>dirty()</code> is <code>true</code>
    whenever any field differs from that baseline; a successful submit makes
    the submitted values the new baseline, because saved means clean.
  </p>
  <code-block code={code.reset}></code-block>

  <h2>The declarative shortcut</h2>
  <p>
    In a deck app with a typed <code>ship</code> client, a plain form post
    doesn’t need a controller at all: put
    <code>mutation="&lt;client&gt;.&lt;method&gt;"</code> on the
    <code>&lt;form&gt;</code> and the runtime intercepts submit, calls that
    client method with the form’s <code>FormData</code>, and injects a
    <code>$mutation</code> signal that walks <code>loading → ok | error</code>
    — the same three-state shape <code>query()</code> uses for reads.
    Side effects use typed callback props
    <code>on-success</code> / <code>on-failure</code> — not a custom-event
    bus. Reach for <code>form()</code> when you need validation or
    field-level state; reach for <code>mutation=</code> when the form is a
    straight post.
  </p>
  <code-block code={code.mutation}></code-block>

  <p style(callout)>
    A Sail page renders its markup before JavaScript arrives, so a form is
    visible and focusable immediately — but submitting needs the runtime
    attached. The server-rendered <code>action</code> a deck app injects
    marks the seam; it is not a working no-JS submit target.
  </p>

  <row justify="between" gap="2" style(footNav)>
    <a variant="default" href="/docs/guide/building-a-component">← Building a component</a>
    <a variant="solid" tone="primary" href="/docs/guide/making-a-theme">Making a theme →</a>
  </row>
</column>
