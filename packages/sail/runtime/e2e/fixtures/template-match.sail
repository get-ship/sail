<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => ({
  tone: signal('info'),
}))
</controller>
<div data-testid="match-page">
  <span role="button" tabindex="0" data-testid="to-info" on:click={() => tone.set('info')}>info</span>
  <span role="button" tabindex="0" data-testid="to-success" on:click={() => tone.set('success')}>success</span>
  <span role="button" tabindex="0" data-testid="to-other" on:click={() => tone.set('other')}>other</span>
  <section data-testid="alert">
    <match {tone()}>
      {'info' =>
        <p data-testid="arm-info">Heads up.</p>
      }
      {'success' =>
        <p data-testid="arm-success">All good.</p>
      }
      {else =>
        <p data-testid="arm-else">Pick a tone.</p>
      }
    </match>
  </section>
</div>
