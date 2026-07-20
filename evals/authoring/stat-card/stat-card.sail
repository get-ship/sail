<!--
  stat-card — presentational metric card.
  Title (muted), large value, sign-toned delta; header click reveals details.
-->
<scope>
  props: {
    title = 'Metric'
    value = '0'
    delta = '0%'
    details = ''
  }
</scope>

<controller>
import { controller, signal, derived } from '@ship/sail-runtime'

export default controller(({ delta }: { delta: () => string }) => {
  const open = signal(false)

  // 'pos' | 'neg' | 'zero' from the leading signed number in `delta`.
  const deltaTone = derived((): 'pos' | 'neg' | 'zero' => {
    const raw = delta().trim()
    const match = raw.match(/^[+-]?[0-9]*\.?[0-9]+/)
    if (!match) return 'zero'
    const n = Number.parseFloat(match[0]!)
    if (!Number.isFinite(n) || n === 0) return 'zero'
    return n > 0 ? 'pos' : 'neg'
  })

  return {
    open,
    deltaTone,
    toggle: (): void => open.set(!open()),
  }
})
</controller>

<card sx="p-4 bg-surface-1 border border-border rounded-lg shadow-sm">
  <column gap="2">
    <button
      type="button"
      variant="ghost"
      sx="w-full p-0 cursor-pointer"
      on:click={() => toggle()}
    >
      <row align="center" justify="between" gap="3" sx="w-full">
        <column gap="1" sx="grow-1">
          <small sx="text-sm text-muted-foreground tracking-wide">{title()}</small>
          <strong sx="text-3xl tracking-tight text-foreground">{value()}</strong>
        </column>

        {#match deltaTone()}
          {:case 'pos'}
            <span sx="text-sm font-semibold text-success">{delta()}</span>
          {:case 'neg'}
            <span sx="text-sm font-semibold text-destructive">{delta()}</span>
          {:default}
            <span sx="text-sm font-semibold text-muted-foreground">{delta()}</span>
        {/match}
      </row>
    </button>

    {#if open()}
      <p sx="text-sm text-muted-foreground border-t border-border-subtle pt-2 leading-relaxed">
        {details()}
      </p>
    {/if}
  </column>
</card>
