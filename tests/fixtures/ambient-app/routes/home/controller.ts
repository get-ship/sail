/**
 * Authored route controller — uses only compiler-provided ambients
 * (`controller`, `signal`, `style`) and runtime *types* via import.
 * No hand ambient shims in this project.
 */
import type { ControllerScope } from '@ship/sail-runtime'

/** Template/host surface the setup receives (route-level Scope shape). */
type HomeScope = {
  // Intentionally empty host inputs — surface is fully owned by the controller.
}

export default controller((_ctx: ControllerScope<HomeScope>) => {
  const count = signal(0)
  const inc = () => count.set(count() + 1)
  // Style builder under tsc: setters + token consts + string hyphenated tokens.
  const pill = style
    .bg(primary)
    .fg('muted-foreground')
    .padX(2)
    .radius(sm)
    .when({ hover: style.bg(muted) })
  return { count, inc, pill }
})
