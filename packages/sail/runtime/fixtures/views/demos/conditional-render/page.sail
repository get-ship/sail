<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => ({ enabled: signal(false) }))
</controller>

<div data-demo="true">
  <p><a href="/demos">← demos</a></p>
  <h1>conditional render</h1>
  <p>
    Block <code>if … else</code>. The branch swap is reactive — flip the
    checkbox and the panel re-renders.
  </p>

  <label>
    <input type="checkbox" data-test="toggle" bind:checked={enabled} />
    Show secret panel
  </label>

  <if {enabled()}>
    <section data-test="panel-on">
      <h2>Panel is visible.</h2>
      <p>Toggling the checkbox unmounts this branch and mounts the other.</p>
    </section>
  <else>
    <section data-test="panel-off">
      <p>Hidden — flip the box to reveal.</p>
    </section>
  </else></if>
</div>
