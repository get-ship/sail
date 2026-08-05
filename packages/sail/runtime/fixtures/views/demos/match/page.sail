<controller>
import { controller, signal } from '@ship/sail-runtime'

type Tone = 'info' | 'success' | 'warning' | 'destructive'

export default controller(() => ({ tone: signal<Tone>('info') }))
</controller>

<div data-demo="true">
  <p><a href="/demos">← demos</a></p>
  <h1>match</h1>
  <p>
    Block <code>match expr / case "x" / default</code>. Like a switch
    statement, but only the active branch is mounted.
  </p>

  <div role="tablist">
    <button role="tab" data-test="tone-info"        aria-selected={tone() === 'info'}        on:click={() => tone.set('info')}>info</button>
    <button role="tab" data-test="tone-success"     aria-selected={tone() === 'success'}     on:click={() => tone.set('success')}>success</button>
    <button role="tab" data-test="tone-warning"     aria-selected={tone() === 'warning'}     on:click={() => tone.set('warning')}>warning</button>
    <button role="tab" data-test="tone-destructive" aria-selected={tone() === 'destructive'} on:click={() => tone.set('destructive')}>destructive</button>
  </div>

  <section data-test="alert" data-tone={tone()}>
    <match {tone()}>
      <case {'info'}>
        <strong>Heads up.</strong> Just a friendly note.
      </case><case {'success'}>
        <strong>All good.</strong> The thing finished without errors.
      </case><case {'warning'}>
        <strong>Careful.</strong> This action may have side-effects.
      </case><case {'destructive'}>
        <strong>Stop.</strong> This will permanently delete data.
      </case><default>
        <em>Pick a tone.</em>
    </default></match>
  </section>
</div>
