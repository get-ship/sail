<!--
  /components/* layout shell — docs-rail geometry: fixed side list + scrolling main.
  List is text-only (no previews). Catalog tiles stay on /components hub.
-->

<scope>
  style shell = { box: size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
  // Same rail geometry as sail-docs-rail (14rem / space-56).
  style rail = { box: flow({ display: none, dir: column }).size({ w: 56, h: full, min-h: 0 }).space({ pad-x: 3, pad-y: 4 }).overflow({ x: hidden, y: auto }), paint: border({ color: border, width: 1, edge: e }).fill({ color: background }).ink({ color: foreground }), when: { tablet: { box: flow({ display: flex }) } } }
  style railLabel = { box: space({ mt: 4, mb: 1 }), paint: ink({ color: muted-foreground }), text: type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style railLink = { box: flow({ display: block }).space({ pad-x: 2, pad-y: 1 }), paint: radius(md).ink({ color: muted-foreground }), text: type({ family: mono, size: sm }), when: { hover: { paint: ink({ color: foreground }).fill({ color: hover-bg }) }, current: { paint: ink({ color: primary }).fill({ color: muted }), text: type({ weight: semibold }) } } }
  style railHub = { box: flow({ display: block }).space({ pad-x: 2, pad-y: 1, mb: 2 }), paint: radius(md).ink({ color: muted-foreground }), text: type({ size: sm, weight: medium }), when: { hover: { paint: ink({ color: foreground }).fill({ color: hover-bg }) }, current: { paint: ink({ color: primary }).fill({ color: muted }), text: type({ weight: semibold }) } } }
  style mainPane = { box: space({ pad: 4 }).basis(0).size({ min-h: 0 }).overflow({ x: hidden, y: auto }), when: { tablet: { box: space({ pad: 6 }) } } }
  style mobileBar = { box: flow({ display: flex }).gap(2).space({ pad-x: 3, pad-y: 2 }), paint: border({ color: border, width: 1, edge: b }).fill({ color: background }), when: { tablet: { box: flow({ display: none }) } } }
  style contentCol = { box: size({ min-w: 0, min-h: 0 }) }
  style mobileLabel = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style drawerTitle = { text: type({ size: sm }) }
  style fullW = { box: size({ w: full }) }

  type ComponentsNavProps = {
    path: any
    currentSlug: any
    sections: any
    onNav: any
  }
</scope>

<controller>
import { CATALOG, type CatalogEntry } from '../compositions/component-catalog'

function slugFromPath(p: string): string {
  const m = p.match(/^\/components\/([^/]+)/)
  return m?.[1] ?? ''
}

const CAT_ORDER: CatalogEntry['cat'][] = [
  'inputs',
  'display',
  'feedback',
  'overlay',
  'navigation',
  'data',
  'layout',
  'effects',
]

const CAT_LABEL: Record<CatalogEntry['cat'], string> = {
  inputs: 'Inputs',
  display: 'Display',
  feedback: 'Feedback',
  overlay: 'Overlay',
  navigation: 'Navigation',
  data: 'Data',
  layout: 'Layout',
  effects: 'Effects',
}

export default controller((ctx) => {
  // deck route.pathname is SSOT and updates same-tick on navigate.
  const path = derived(() => ctx.route?.pathname?.() ?? '')
  const navOpen = signal(false)
  const openNav = (): void => navOpen.set(true)
  const closeNav = (): void => navOpen.set(false)
  const noopNav = (): void => {}
  effect(() => {
    path()
    navOpen.set(false)
  })

  const currentSlug = derived(() => slugFromPath(path()))
  // L1 primitives only in the rail (packs stay on the hub catalog).
  const sections = derived(() => {
    const lang = CATALOG.filter((e: CatalogEntry) => e.pack === 'lang')
    return CAT_ORDER.map((cat) => ({
      cat,
      label: CAT_LABEL[cat],
      items: lang.filter((e: CatalogEntry) => e.cat === cat),
    })).filter((s: { items: CatalogEntry[] }) => s.items.length > 0)
  })

  return { path, navOpen, openNav, closeNav, noopNav, currentSlug, sections }
})
</controller>

<partial components-nav props: ComponentsNavProps>
  <a
    href="/components"
    on:click={onNav}
    current={path() === '/components' ? 'page' : null}
    style(railHub)
  >All components</a>
  <for {sections() as sec} key={sec.cat}>
    <p style(railLabel)>{sec.label}</p>
    <for {sec.items as item} key={item.slug}>
      <a
        href={`/components/${item.slug}`}
        on:click={onNav}
        current={currentSlug() === item.slug ? 'page' : null}
        style(railLink)
      >{item.tag}</a>
    </for>
  </for>
</partial>

<row nowrap align="stretch" gap="0" style(shell, { box: size({ w: full, min-w: 0 }).flow({ grow: 1 }) })>
  <nav label="Components" testid="components-rail" style(rail, { box: flow({ shrink: 0 }) })>
    <components-nav path={path} currentSlug={currentSlug} sections={sections} onNav={noopNav}></components-nav>
  </nav>

  <column gap="0" style(contentCol, { box: flow({ grow: 1 }).size({ min-w: 0 }) })>
    <div style(mobileBar)>
      <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
      <small style(mobileLabel)>Components</small>
    </div>
    <main style(mainPane, { box: flow({ grow: 1 }).size({ min-w: 0 }) })>
      <slot name="route"></slot>
    </main>
  </column>
</row>

<!-- Mobile drawer only below tablet; side rail owns nav at tablet+. -->
<when {tablet}>
{else =>
<dialog
  id="sail-components-drawer"
  open={navOpen()}
  side="start"
  size="sm"
  label="Components navigation"
  on-close={closeNav}
>
  <header>
    <row align="center" justify="between" gap="2" style(fullW)>
      <strong style(drawerTitle)>Components</strong>
      <button type="button" variant="ghost" size="sm" on:click={closeNav}>Close</button>
    </row>
  </header>
  <section>
    <components-nav path={path} currentSlug={currentSlug} sections={sections} onNav={closeNav}></components-nav>
  </section>
</dialog>
}</when>
