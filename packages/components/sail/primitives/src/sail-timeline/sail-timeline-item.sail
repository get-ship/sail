<!--
  <sail-timeline-item> — one timeline entry: gutter connector + tone dot,
  head row (title slot + timestamp), body content. `interactive` makes the
  row a button (role/tabindex/sail-select from the controller; the pressed
  surface from the compound variant). The connector is two real segments
  around the dot — the first item hides the top run, the last hides the
  bottom run (visibility, so the gutter geometry never shifts).

  Residual (LANG-GAPS.md): forced-colors focus outline (system colors are
  not sx values); first/last detection is mount-time (static lists).
-->

<scope>
  props: {
    tone = 'default'
    at = ''
    interactive = false
  }
  compound: [
    { when: { interactive: true }, sx: 'cursor-pointer rounded-sm hover:bg-muted focus:ring-ring' },
  ]
</scope>

<controller>
import { controller, effect, signal } from '@ship/sail-runtime'

export default controller((ctx) => {
    const first = signal(false)
    const last = signal(false)

    mount((host) => {
      if (!host.hasAttribute('role')) host.setAttribute('role', 'listitem')
      first.set(host.matches(':first-of-type'))
      last.set(host.matches(':last-of-type'))

      const stopRole = effect(() => {
        if (ctx.interactive()) {
          host.setAttribute('role', 'button')
          if (!host.hasAttribute('tabindex')) host.setAttribute('tabindex', '0')
        } else {
          host.setAttribute('role', 'listitem')
          host.removeAttribute('tabindex')
        }
      })

      const emit = () => {
        host.dispatchEvent(
          new CustomEvent('sail-select', { bubbles: true, composed: true, detail: {} }),
        )
      }

      const onClick = () => { if (ctx.interactive()) emit() }

      const onKeydown = (e: KeyboardEvent) => {
        if (!ctx.interactive()) return
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          emit()
        }
      }

      host.addEventListener('click', onClick)
      host.addEventListener('keydown', onKeydown)

      return () => {
        stopRole()
        host.removeEventListener('click', onClick)
        host.removeEventListener('keydown', onKeydown)
      }
    })

    return { first, last }
  })

</controller>
<row gap="2" align="stretch" sx="relative">
  <column align="center" gap="0" sx="w-5">
    {#if first()}
      <span sx="w-half h-3 invisible"></span>
    {:else}
      <span sx="w-half h-3 bg-border-subtle"></span>
    {/if}
    <span sx="w-2_5 h-2_5 rounded-full bg-tone-fg"></span>
    {#if last()}
      <span sx="w-half grow-1 invisible"></span>
    {:else}
      <span sx="w-half grow-1 bg-border-subtle"></span>
    {/if}
  </column>
  <column gap="0" sx="grow-1 min-w-0 py-2">
    <row align="baseline" justify="between" gap="2" sx="text-sm font-medium text-foreground font-body">
      <slot name="title"></slot>
      {#if at()}
        <time sx="text-muted-foreground text-xs tracking-wide font-mono">{at()}</time>
      {/if}
    </row>
    <div sx="text-sm text-muted-foreground leading-normal mt-1"><slot></slot></div>
  </column>
</row>
