<!--
  <sail-site-header> — site chrome: brand, Docs/Components/Playground nav,
  theme picker, GitHub.

  Playground is a full-bleed workbench: no site navbar (theme + exit live in
  the playground toolbar). Landing uses autohide. Docs/components keep a
  persistent bar.
-->

<scope>
  style brand = { text: type({ size: lg, tracking: tight, wrap: nowrap }) }
  style brandMark = { paint: ink({ color: primary }) }
  style productSwitch = { box: space({ pad: half }).size({ min-w: 0, max-w: full }).overflow({ x: auto }), paint: fill({ color: muted }).border({ color: border, width: 1, style: solid }).radius(full) }
  style productLink = { box: flow({ display: inline-flex }).space({ pad-x: 1_5 }).size({ h: control-h-sm }), paint: radius(full).ink({ color: muted-foreground }), text: type({size: xs, weight: medium, leading: none, wrap: nowrap, decoration: none}), when: { hover: { paint: ink({ color: foreground }).fill({ color: hover-bg }) }, current: { paint: fill({ color: primary }).ink({ color: primary-foreground }) }, tablet: { box: space({ pad-x: 2_5 }), text: type({ size: sm }) } } }
  style brandWord = { box: flow({ display: none }), text: type({ wrap: nowrap }), when: { tablet: { box: flow({ display: inline }) } } }
  style shell = { box: size({ h: screen, min-h: screen }) }
  style landingMain = { box: size({basis: 0, min-w: 0, min-h: 0, max-w: full}).overflow({ x: hidden, y: auto }) }
  style appMain = { box: flow({ display: flex, dir: column }).size({min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
  style githubLink = { box: flow({ display: inline-flex }).space({ pad: 1 }), paint: radius(md).ink({ color: muted-foreground }), when: { hover: { paint: ink({ color: foreground }).fill({ color: hover-bg }) } } }
</scope>

<controller>
import { siteTokenCss } from '../compositions/site-token-css'
void siteTokenCss

function sectionFor(p: string): string {
  if (p === '/') return 'home'
  if (p === '/playground' || p.startsWith('/playground/')) return 'playground'
  if (p === '/components' || p.startsWith('/components/')) return 'components'
  if (p === '/docs' || p.startsWith('/docs/')) return 'docs'
  if (p === '/sail' || p.startsWith('/sail/')) return 'docs'
  return 'docs'
}

function livePath(ctx: { route?: { pathname?: () => string } }): string {
  // Empty — not '/' — when route is missing. Defaulting to '/' used to turn
  // on landing autohide (position:fixed navbar) on every page after a bake
  // that did not inject route into this layout CE.
  const fn = ctx.route?.pathname
  if (typeof fn !== 'function') return ''
  return fn() || ''
}

export default controller((ctx) => {
    const path = derived(() => livePath(ctx as { route?: { pathname?: () => string } }))
    const section = derived(() => (path() ? sectionFor(path()) : 'docs'))
    // Workbench owns its chrome — hide the site bar so editor gets the pixels.
    const workbench = derived(() => section() === 'playground')
    // Only the real landing page tucks the bar. Unknown path stays persistent.
    const autohide = derived(() => path() === '/')
    const landing = derived(() => path() === '/')
    effect(() => {
      applyTheme(readStoredTheme())
    })

    return { path, section, workbench, autohide, landing }
})
</controller>

<column gap="0" style(shell, { box: size({ w: full }) })>
  <if {!workbench()}>
    <navbar sticky bordered autohide={autohide() ? true : null} use:chromeOffset>
      <a slot="brand" href="/" label="SHIP UI home" style({ box: flow({ shrink: 0 }) })>
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

      <row slot="center" gap="1" align="center" nowrap testid="site-nav" label="Site" style(productSwitch, { box: size({ min-w: 0 }) })>
        <a href="/docs" current={section() === 'docs' ? 'page' : null} style(productLink, { box: flow({ shrink: 0 }) })>Docs</a>
        <a href="/components" current={section() === 'components' ? 'page' : null} style(productLink, { box: flow({ shrink: 0 }) })>Components</a>
        <a href="/playground" current={section() === 'playground' ? 'page' : null} style(productLink, { box: flow({ shrink: 0 }) })>Playground</a>
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
    <main style(landingMain, { box: flow({ grow: 1 }).size({ w: full, min-w: 0 }) })>
      <slot name="route"></slot>
    </main>
  {else => <main style(appMain, { box: flow({ grow: 1 }).size({ w: full, min-w: 0 }) })>
      <slot name="route"></slot>
    </main>}</if>
</column>
