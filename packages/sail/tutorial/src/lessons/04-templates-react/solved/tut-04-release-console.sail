<!--
  Lesson 04 solved — the template is a read of state.

  {expr} and attribute bindings hold zero-arg reads only; handlers write.
  {#match} swaps the status badge (each branch enters with its own
  motion), the keyed {#for} moves rows on promote() instead of
  rebuilding them, and the event modifiers replace handler boilerplate.
-->

<controller>
type Phase = 'idle' | 'building' | 'live' | 'failed'

interface Env {
  id: string
  name: string
  tier: string
  region: string
}

const SEED: Env[] = [
  { id: 'prod', name: 'production', tier: 'prod', region: 'eu-central' },
  { id: 'stage', name: 'staging', tier: 'preview', region: 'eu-central' },
  { id: 'dev', name: 'dev-sandbox', tier: 'preview', region: 'us-east' },
]

export default controller(() => {
    const phase = signal<Phase>('idle')
    const envs = signal<Env[]>(SEED)
    const query = signal('')

    const filtered = derived(() =>
      envs().filter((e) => e.name.includes(query().toLowerCase())),
    )

    return {
      phase,
      filtered,
      // A tiny async state machine: the handler WRITES phase, the
      // {#match} in the template reads it. The build "runs" for a
      // moment, then lands on live (mostly) or failed (sometimes) —
      // so every branch of the match is reachable.
      deploy: (): void => {
        if (phase() === 'building') return
        phase.set('building')
        setTimeout(() => phase.set(Math.random() < 0.75 ? 'live' : 'failed'), 900)
      },
      reset: (): void => phase.set('idle'),
      search: (e: Event): void => query.set((e.target as HTMLInputElement).value),
      clear: (): void => query.set(''),
      // Move the env to the front. The keyed {#for} sees the same ids in a
      // new order, so the reconciler MOVES the existing rows — DOM identity
      // (focus, animation state) survives the reorder.
      promote: (id: string): void =>
        envs.set([...envs().filter((e) => e.id === id), ...envs().filter((e) => e.id !== id)]),
    }
  })
</controller>

<scope>
  const envRow = style({ box: padX(3).padY(2), paint: bg(surface-1).radius(md) })
  const envName = style({ text: fontSize(sm) })
  const envRegion = style({ paint: fg(muted-foreground) })
</scope>

<column gap="4">
  <row align="center" justify="between">
    <h3>Release console</h3>
    <match {phase()}>
      <case {'idle'}> <badge tone="muted">
        style({ animate: enter({ opacity: 'fade' }) })
idle</badge>
      </case><case {'building'}> <badge tone="warning">
        style({ animate: enter({ translate-x: 'spring:snappy' }) })
building</badge>
      </case><case {'live'}> <badge tone="success">
        style({ animate: enter({ scale: 'spring:bouncy' }) })
live</badge>
      </case><default> <badge tone="destructive">
        style({ animate: enter({ opacity: 'fade' }) })
failed</badge>
    </default></match>
  </row>

  <row gap="2">
    <button variant="solid" tone="primary" disabled={phase() === 'building'} on:click={() => deploy()}>
      {phase() === 'failed' ? 'Retry deploy' : 'Deploy'}
    </button>
    <button variant="ghost" on:click={() => reset()}>Reset</button>
  </row>

  <separator></separator>

  <input placeholder="Filter environments — Esc clears"
         on:input|debounce:300={search}
         on:keydown|key:Escape={clear} />

  <column gap="1">
    <for {filtered()} as="env" key={env.id}>
      <row gap="3" align="center">
        style(envRow)

        <badge tone={env.tier === 'prod' ? 'primary' : 'muted'} variant="outline" size="sm">{env.tier}</badge>
        <strong grow>
          style(envName)
{env.name}</strong>
        <small>
          style(envRegion)
{env.region}</small>
        <button size="sm" variant="ghost" on:click={() => promote(env.id)}>Promote</button>
      </row>
    </for>
  </column>
</column>
