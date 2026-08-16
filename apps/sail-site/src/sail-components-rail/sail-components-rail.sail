<!--
  /components/* layout shell — docs-rail geometry: fixed side list + scrolling main.
  List is text-only (no previews). Catalog tiles stay on /components hub.
-->

<scope>
  style shell = { size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
  // Same rail geometry as sail-docs-rail (14rem / space-56).
  style rail = { display(none).dir(column).size({ w: 56, h: full, min-h: 0 }).pad({ x: 3, y: 4 }).overflow({ x: hidden, y: auto }).border({ color: border, width: 1, edge: e }).bg(background).fg(foreground), when: { tablet: display(flex) } }
  style railLabel = { margin({ t: 4, b: 1 }).fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style railLink = { display(block).pad({ x: 2, y: 1 }).radius(md).fg(muted-foreground).type({ family: mono, size: sm }), when: { hover: fg(foreground).bg(hover-bg), current: fg(primary).bg(muted).type({ weight: semibold }) } }
  style railHub = { display(block).pad({ x: 2, y: 1 }).margin({ b: 2 }).radius(md).fg(muted-foreground).type({ size: sm, weight: medium }), when: { hover: fg(foreground).bg(hover-bg), current: fg(primary).bg(muted).type({ weight: semibold }) } }
  style mainPane = { pad(4).size({min-h: 0, basis: 0}).overflow({ x: hidden, y: auto }), when: { tablet: pad(6) } }
  style mobileBar = { display(flex).pad({ x: 3, y: 2 }).border({ color: border, width: 1, edge: b }).bg(background), when: { tablet: display(none) } }
  style contentCol = { size({ min-w: 0, min-h: 0 }) }
  style mobileLabel = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style drawerTitle = { type({ size: sm }) }
  style fullW = { size({ w: full }) }

  type ComponentsNavProps = {
    path: any
    currentSlug: any
    sections: any
    onNav: any
  }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-components-rail.sail'
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

export default controller((ctx: ControllerScope<Scope>) => {
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
      >{item.tag === 'a' ? 'anchor' : item.tag}</a>
    </for>
  </for>
</partial>

<row nowrap align="stretch" gap="0" style(shell, { size({ w: full, min-w: 0 }).grow(1) })>
  <nav label="Components" testid="components-rail" use:scrollRestore={'components-rail'} style(rail, { shrink(0) })>
    <components-nav path={path} currentSlug={currentSlug} sections={sections} onNav={noopNav}></components-nav>
  </nav>

  <column gap="0" style(contentCol, { grow(1).size({ min-w: 0 }) })>
    <div style(mobileBar)>
      <row gap="2" align="center">
        <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
        <small style(mobileLabel)>Components</small>
      </row>
    </div>
    <main style(mainPane, { grow(1).size({ min-w: 0 }) })>
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
