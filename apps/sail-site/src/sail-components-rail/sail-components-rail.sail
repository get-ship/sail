<!--
  /components/* layout shell — docs-rail geometry: fixed side list + scrolling main.
  List is text-only (no previews). Catalog tiles stay on /components hub.
-->

<scope>
  style shell = { box: minH(0).h(full).minW(0).overflow(hidden) }
  // Same rail geometry as sail-docs-rail (14rem / space-56).
  style rail = { box: display(none).flexDir(column).w(56).minH(0).h(full).padY(4).padX(3).overflowY(auto).overflowX(hidden), paint: borderEWidth(1).border(border).bg(background).fg(foreground), when: { tablet: { box: display(flex) } } }
  style railLabel = { box: mt(4).mb(1), paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style railLink = { box: display(block).padY(1).padX(2), paint: radius(md).fg(muted-foreground), text: fontSize(sm).fontFamily(mono), when: { hover: { paint: fg(foreground).bg(hover-bg) }, current: { paint: fg(primary).bg(muted), text: fontWeight(semibold) } } }
  style railHub = { box: display(block).padY(1).padX(2).mb(2), paint: radius(md).fg(muted-foreground), text: fontSize(sm).fontWeight(medium), when: { hover: { paint: fg(foreground).bg(hover-bg) }, current: { paint: fg(primary).bg(muted), text: fontWeight(semibold) } } }
  style mainPane = { box: pad(4).basis(0).minH(0).overflowX(hidden).overflowY(auto), when: { tablet: { box: pad(6) } } }
  style mobileBar = { box: display(flex).gap(2).padX(3).padY(2), paint: borderBWidth(1).borderB(border).bg(background), when: { tablet: { box: display(none) } } }
  style contentCol = { box: minH(0).minW(0) }
  style mobileLabel = { paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) }
  style drawerTitle = { text: fontSize(sm) }
  style fullW = { box: w(full) }

  type ComponentsNavProps = {
    path: any
    currentSlug: any
    sections: any
    onNav: any
  }
</scope>

<controller>
import { CATALOG, type CatalogEntry } from '../compositions/component-catalog'
import { currentNav } from '../compositions/current-nav'

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

  return { path, navOpen, openNav, closeNav, noopNav, currentSlug, sections, currentNav }
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

<row nowrap align="stretch" gap="0" style(shell, { box: w(full).grow(1).minW(0) })>
  <nav label="Components" testid="components-rail" use:currentNav style(rail, { box: shrink(0) })>
    <components-nav path={path} currentSlug={currentSlug} sections={sections} onNav={noopNav}></components-nav>
  </nav>

  <column gap="0" style(contentCol, { box: grow(1).minW(0) })>
    <div style(mobileBar)>
      <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
      <small style(mobileLabel)>Components</small>
    </div>
    <main style(mainPane, { box: grow(1).minW(0) })>
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
