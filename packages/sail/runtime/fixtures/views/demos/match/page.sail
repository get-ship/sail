<controller>
import { controller, signal } from '@ship/sail-runtime'

type Tone = 'info' | 'success' | 'warning' | 'destructive'

export default controller(() => ({ tone: signal<Tone>('info') }))
</controller>

<scope>
  style page = { pad(6).margin({ x: auto }).size({ max-w: breakpoint-tablet }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style alert = { pad(4).border({ color: border, width: 1 }).radius(md).bg(surface-1) }
</scope>

<column gap="4" style(page)>
  <p><a href="/demos">← demos</a></p>
  <column gap="1">
    <h1>match</h1>
    <p style(muted)>
      Block <code>match expr / case "x" / default</code>. Like a switch
      statement, but only the active branch is mounted.
    </p>
  </column>

  <row gap="2" role="tablist">
    <button
      role="tab"
      data-test="tone-info"
      variant={tone() === 'info' ? 'solid' : 'outline'}
      tone="info"
      aria-selected={tone() === 'info'}
      on:click={() => tone.set('info')}
    >info</button>
    <button
      role="tab"
      data-test="tone-success"
      variant={tone() === 'success' ? 'solid' : 'outline'}
      tone="success"
      aria-selected={tone() === 'success'}
      on:click={() => tone.set('success')}
    >success</button>
    <button
      role="tab"
      data-test="tone-warning"
      variant={tone() === 'warning' ? 'solid' : 'outline'}
      tone="warning"
      aria-selected={tone() === 'warning'}
      on:click={() => tone.set('warning')}
    >warning</button>
    <button
      role="tab"
      data-test="tone-destructive"
      variant={tone() === 'destructive' ? 'solid' : 'outline'}
      tone="destructive"
      aria-selected={tone() === 'destructive'}
      on:click={() => tone.set('destructive')}
    >destructive</button>
  </row>

  <section data-test="alert" style(alert)>
    <match {tone()}>
      {'info' =>
        <span><strong>Heads up.</strong> Just a friendly note.</span>
      }
      {'success' =>
        <span><strong>All good.</strong> The thing finished without errors.</span>
      }
      {'warning' =>
        <span><strong>Careful.</strong> This action may have side-effects.</span>
      }
      {'destructive' =>
        <span><strong>Stop.</strong> This will permanently delete data.</span>
      }
      {else =>
        <em>Pick a tone.</em>
      }
    </match>
  </section>
</column>
