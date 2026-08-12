<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => ({ enabled: signal(false) }))
</controller>

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style panel = {
    box: pad(4),
    paint: borderWidth(1).border(border).radius(md).bg(surface-1),
  }
</scope>

<column gap="4" style(page)>
  <p><a href="/demos">← demos</a></p>
  <column gap="1">
    <h1>conditional render</h1>
    <p style(muted)>
      Block <code>if … else</code>. The branch swap is reactive — flip the
      checkbox and the panel re-renders.
    </p>
  </column>

  <label>
    <checkbox data-test="toggle" bind:checked={enabled} />
    Show secret panel
  </label>

  <if {enabled()}>
    <section data-test="panel-on" style(panel)>
      <h2>Panel is visible.</h2>
      <p style(muted)>Toggling the checkbox unmounts this branch and mounts the other.</p>
    </section>
  {else =>
    <section data-test="panel-off" style(panel)>
      <p style(muted)>Hidden — flip the box to reveal.</p>
    </section>
  }</if>
</column>
