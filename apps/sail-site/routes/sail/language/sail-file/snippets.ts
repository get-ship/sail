// Snippets for /language/sail-file — SLICES of a real component
// (apps/sail-site/src/demo-player/demo-player.sail, the intro's player),
// walked part by part. Never the whole file at once (ruling: no code dumps).
export const code = {
  shape: `<!-- the contract: props, variants, types -->
<scope>
  props: {
    playing = true
  }
</scope>

<!-- behavior: TypeScript, signals -->
<controller>
import { controller } from '@ship/sail-runtime'
export default controller(() => ({}))
</controller>

<!-- the template: markup -->
<card>Signals in the Rain</card>`,

  template: `<card sx="p-3 w-full">
  <row gap="3" align="center" nowrap>
    <column gap="1" sx="grow-1 min-w-0">
      <strong sx="text-sm truncate">Signals in the Rain</strong>
      <progress value={position()} max="100"></progress>
    </column>
    {#if playing()}
      …equalizer bars…
    {:else}
      …idle bars…
    {/if}
  </row>
</card>`,

  scope: `<scope>
  props: {
    playing = true
  }
</scope>`,

  controller: `<controller>
import { controller, signal, effect } from '@ship/sail-runtime'
import type { Scope } from './demo-player.sail'

export default controller((ctx: Scope) => {
    const position = signal(23)
    effect(() => {
      if (!ctx.playing()) return
      const t = setInterval(() => position.set((p) => (p + 1) % 100), 150)
      return () => clearInterval(t)
    })
    return { position }
})
</controller>`,

  usage: `<demo-player playing={playing()}></demo-player>`,

  slot: `<!-- a component renders its children where it places <slot> -->
<card><slot></slot></card>

<!-- …so users write: -->
<my-panel>Anything in here lands at the slot.</my-panel>`,
}
