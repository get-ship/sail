<!--
  <demo-player> — the intro's nested-component demo. A compact music-player
  card driven entirely by one prop: `playing`. The equalizer bars only exist
  (and loop) while playing() holds them in the DOM; the position clock is an
  effect that TRACKS playing(), so pausing re-runs the effect and its cleanup
  stops the timer. Lifted from the showcase player, shrunk to embed.
-->
<scope>
  props: {
    playing = true
  }
</scope>

<controller>
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
</controller>

<card sx="p-3 w-full">
  <row gap="3" align="center" nowrap>
    <column gap="1" sx="grow-1 min-w-0">
      <strong sx="text-sm truncate">Signals in the Rain</strong>
      <p sx="text-xs text-muted-foreground truncate">The Reactive Quartet</p>
      <progress value={position()} max="100"></progress>
    </column>
    {#if playing()}
      <row gap="1" align="end" sx="min-h-6">
        <div sx="w-1 h-4 rounded-sm bg-primary origin-bottom" motion:loop={{ scale-y: 'spring:gentle', stagger: '0.1s' }}></div>
        <div sx="w-1 h-6 rounded-sm bg-primary opacity-75 origin-bottom" motion:loop={{ scale-y: 'spring:gentle', stagger: '0.1s' }}></div>
        <div sx="w-1 h-3 rounded-sm bg-primary origin-bottom" motion:loop={{ scale-y: 'spring:gentle', stagger: '0.1s' }}></div>
        <div sx="w-1 h-5 rounded-sm bg-primary opacity-75 origin-bottom" motion:loop={{ scale-y: 'spring:gentle', stagger: '0.1s' }}></div>
      </row>
    {:else}
      <row gap="1" align="center" sx="min-h-6">
        <div sx="w-1 h-2 rounded-full bg-muted-foreground opacity-50"></div>
        <div sx="w-1 h-2 rounded-full bg-muted-foreground opacity-50"></div>
        <div sx="w-1 h-2 rounded-full bg-muted-foreground opacity-50"></div>
        <div sx="w-1 h-2 rounded-full bg-muted-foreground opacity-50"></div>
      </row>
    {/if}
  </row>
</card>
