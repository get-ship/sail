<controller>
// Authored controller for the `badge` component proof — a REAL
// `controller(setup)` export. Confirms a COMPONENT (`.sail` +
// controller) composes through the same `setup(ctx)` path as a route: the
// synthetic controller runs `resolveController` and recovers the contributed
// scope type from the `ControllerExport`. Component props remain a Phase-2
// concern (the standalone synthetic controller can't model mount-time props
// yet); this proves a state-only, controller-bearing component.
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => {
    const count = signal(0)
    const bump = (): void => count.set(count() + 1)
    return { count, bump }
  })
</controller>

<row gap="1" align="center" justify="center">
  style({ box: pad(1), paint: radius(full).bg(surface-2).fg(foreground) })
  <span>
    style({ box: display(contents), text: fontSize(sm).fontFamily(mono) })
    <button on:click={() => bump()}>{count()}</button>
  </span>
</row>
