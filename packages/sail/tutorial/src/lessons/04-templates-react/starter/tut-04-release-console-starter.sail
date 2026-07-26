<!--
  Lesson 04 starter — the console renders, but nothing reacts yet.

  Fill in the three handlers in the <controller> block, then finish the
  template: the {#match} needs its building/live branches, and the list
  needs key= so promote() MOVES rows instead of rebuilding them.
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
      // TODO 1: deploy() — set phase to 'building', then settle it with a
      // setTimeout: 'live' most of the time, 'failed' sometimes. The
      // handler WRITES the signal; the {#match} in the template reads it.
      deploy: (): void => {},
      reset: (): void => phase.set('idle'),
      search: (e: Event): void => query.set((e.target as HTMLInputElement).value),
      // TODO 2: clear() — empty the query. The template fires it via
      // on:keydown|key:Escape.
      clear: (): void => {},
      // TODO 3: promote(id) — move that env to the FRONT of envs. The
      // keyed {#for} will MOVE the row, not rebuild it.
      promote: (_id: string): void => {},
    }
  })
</controller>

<scope>
  const envRow = style.padX(3).padY(2).bg(surface-1).radius(md)
  const envName = style.fontSize(sm)
  const envRegion = style.fg(muted-foreground)
</scope>

<column gap="4">
  <row align="center" justify="between">
    <h3>Release console</h3>
    {#match phase()}
      {:case 'idle'} <badge tone="muted">idle</badge>
      <!-- TODO 4: {:case 'building'} and {:case 'live'} branches, each
           with a style.enter so the swapped-in badge announces itself. -->
      {:default} <badge tone="destructive">failed</badge>
    {/match}
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
    <!-- TODO 5: this {#for} has no key. Add key={env.id} and compare what
         promote() does to the DOM with and without it. -->
    {#for env of filtered() key={env.id}}
      <row gap="3" align="center" sx={envRow}>
        <badge tone={env.tier === 'prod' ? 'primary' : 'muted'} variant="outline" size="sm">{env.tier}</badge>
        <strong grow sx={envName}>{env.name}</strong>
        <small sx={envRegion}>{env.region}</small>
        <button size="sm" variant="ghost" on:click={() => promote(env.id)}>Promote</button>
      </row>
    {/for}
  </column>
</column>
