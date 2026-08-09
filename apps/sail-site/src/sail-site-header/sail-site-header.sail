<!--
  <sail-site-header> — site chrome: brand, Docs/Components/Playground nav,
  theme picker, GitHub.

  Playground is a full-bleed workbench: no site navbar (theme + exit live in
  the playground toolbar). Landing uses autohide. Docs/components keep a
  persistent bar.
-->

<controller>
import { siteTokenCss } from '../compositions/site-token-css'
import { applyTheme, readStoredTheme } from '@ship/tokens/themes'
void siteTokenCss

function sectionFor(p: string): string {
  if (p === '/') return 'home'
  if (p === '/playground' || p.startsWith('/playground/')) return 'playground'
  if (p === '/components' || p.startsWith('/components/')) return 'components'
  if (p === '/docs' || p.startsWith('/docs/')) return 'docs'
  if (p === '/sail' || p.startsWith('/sail/')) return 'docs'
  return 'docs'
}

function productAttr(section: string): string {
  switch (section) {
    case 'home':
      return 'ship'
    case 'components':
      return 'components'
    case 'playground':
      return 'sail'
    default:
      return 'sail'
  }
}

function livePath(ctx: { route?: { pathname?: () => string } }): string {
  const r = ctx.route
  if (r?.pathname) void r.pathname()
  if (typeof location !== 'undefined' && location.pathname) return location.pathname
  return r?.pathname?.() ?? '/'
}

export default controller((ctx) => {
    const root = document.documentElement
    const path = derived(() => livePath(ctx as { route?: { pathname?: () => string } }))
    const section = derived(() => sectionFor(path()))
    // Workbench owns its chrome — hide the site bar so editor gets the pixels.
    const workbench = derived(() => section() === 'playground')
    const autohide = derived(() => section() === 'home')
    const landing = derived(() => path() === '/')
    effect(() => {
      root.setAttribute('data-product', productAttr(section()))
    })
    effect(() => {
      applyTheme(readStoredTheme())
    })

    return { path, section, workbench, autohide, landing }
})
</controller>

<scope>
  style brand = { text: fontSize(lg).tracking(tight).whitespace(nowrap) }
  style brandMark = { paint: fg(primary) }
  style productSwitch = { box: pad(half).maxW(full).minW(0).overflowX(auto), paint: bg(muted).borderWidth(1).border(solid).border(border).radius(full) }
  style productLink = { box: display(inline-flex).padX(1_5).h(control-h-sm), paint: radius(full).fg(muted-foreground), text: fontSize(xs).fontWeight(medium).leading(none).underline(none).whitespace(nowrap), when: { hover: { paint: fg(foreground).bg(hover-bg) }, current: { paint: bg(primary).fg(primary-foreground) }, tablet: { box: padX(2_5), text: fontSize(sm) } } }
  style brandWord = { box: display(none), text: whitespace(nowrap), when: { tablet: { box: display(inline) } } }
  style shell = { box: minH(screen).h(screen) }
  style landingMain = { box: basis(0).minH(0).minW(0).maxW(full).overflowX(hidden).overflowY(auto) }
  style appMain = { box: display(flex).flexDir(column).basis(0).minH(0).minW(0).overflow(hidden) }
  style githubLink = { box: display(inline-flex).pad(1), paint: radius(md).fg(muted-foreground), when: { hover: { paint: fg(foreground).bg(hover-bg) } } }
</scope>

<column gap="0" style(shell, { box: w(full) })>
  <if {!workbench()}>
    <navbar sticky bordered autohide={autohide()} use:chromeOffset>
      <a slot="brand" href="/" label="SHIP UI home" style({ box: shrink(0) })>
        <row gap="2" align="center" nowrap>
          <svg width="22" height="22" viewBox="0 0 26 26" fill="none" hidden="true">
            <circle cx="6" cy="6" r="3.4" fill="var(--primary)" />
            <circle cx="20" cy="6" r="3.4" fill="var(--primary)" opacity="0.45" />
            <circle cx="6" cy="20" r="3.4" fill="var(--primary)" opacity="0.45" />
            <circle cx="20" cy="20" r="3.4" fill="var(--primary)" opacity="0.7" />
            <path d="M6 6 L20 20 M6 6 L20 6 M6 6 L6 20" stroke="var(--primary)" stroke-width="1.4" opacity="0.5" stroke-linecap="round" />
          </svg>
          <strong style(brandWord)><b style(brandMark)>SHIP</b> UI</strong>
        </row>
      </a>

      <row slot="center" gap="1" align="center" nowrap testid="site-nav" label="Site" style(productSwitch, { box: minW(0) })>
        <a href="/docs" current={section() === 'docs' ? 'page' : null} style(productLink, { box: shrink(0) })>Docs</a>
        <a href="/components" current={section() === 'components' ? 'page' : null} style(productLink, { box: shrink(0) })>Components</a>
        <a href="/playground" current={section() === 'playground' ? 'page' : null} style(productLink, { box: shrink(0) })>Playground</a>
      </row>

      <row slot="actions" gap="2" align="center" nowrap>
        <a
          href="https://github.com/get-ship/sail"
          target="_blank"
          rel="noopener noreferrer"
          label="Sail on GitHub"
          style(githubLink)
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" hidden="true">
            <path d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.17 6.839 9.49.5.092.682-.217.682-.482 0-.237-.008-.866-.013-1.7-2.782.603-3.369-1.34-3.369-1.34-.454-1.156-1.11-1.464-1.11-1.464-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.269 2.75 1.025A9.578 9.578 0 0112 6.836c.85.004 1.705.114 2.504.336 1.909-1.294 2.747-1.025 2.747-1.025.546 1.377.202 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.167 22 16.418 22 12c0-5.523-4.477-10-10-10z" />
          </svg>
        </a>
        <theme-picker></theme-picker>
      </row>
    </navbar>
  </if>

  <if {landing()}>
    <main style(landingMain, { box: grow(1).minW(0).w(full) })>
      <slot name="route"></slot>
    </main>
  {else => <main style(appMain, { box: grow(1).minW(0).w(full) })>
      <slot name="route"></slot>
    </main>}</if>
</column>
