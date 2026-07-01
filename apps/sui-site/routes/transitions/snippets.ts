// Code snippets for the Transitions page. Kept OUT of the controller so the
// compiler's textual scope scan isn't fooled by the `controller({ ... })` and
// `transition:` patterns inside these strings.
export const code = {
  // transition: applies to both enter and leave. The runtime runs the enter
  // phase after inserting the nodes and the leave phase before removing them.
  basic: `{#if open()}
  <div transition:fade={{ duration: 150 }}>Now you see me</div>
{/if}`,

  // in:/out: split the two phases so enter and leave can differ.
  asymmetric: `{#if toast()}
  <div in:slide={{ axis: 'y' }} out:fade={{ duration: 90 }}>Saved</div>
{/if}`,

  // Built-ins use the Web Animations API — no JS interpolation cost. Config is
  // duration / easing / delay plus per-transition extras (slide.axis, scale.from).
  builtins: `{#if panel()}
  <aside transition:scale={{ from: 0.9, easing: 'ease-out' }}>…</aside>
{/if}`,

  // A custom transition is any object matching the Transition type, returned
  // from the controller's scope() so the template can name it.
  custom: `import { controller, signal } from '@ship/sui-runtime'
import type { Transition } from '@ship/sui-runtime'

// in/out each return an Animation (or undefined for a no-op side).
const grow: Transition = {
  in: (node, o) =>
    node.animate(
      [{ height: '0', opacity: 0 }, { height: 'auto', opacity: 1 }],
      { duration: (o.duration as number) ?? 200, fill: 'both' },
    ),
}

export default controller({
  scope: () => ({ open: signal(false), grow }),
})`,

  customTemplate: `{#if open()}
  <section transition:grow>…</section>
{/if}`,
}
