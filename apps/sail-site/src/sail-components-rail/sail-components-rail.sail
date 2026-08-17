<!--
  /components/* layout shell — docs-rail geometry: fixed side list + scrolling main.
  List is text-only (no previews). Catalog tiles stay on /components hub.
-->

<scope>
  style shell = { size(grow).overflow(hidden) }
  // Hug width so the rail does not steal leftover from main. Pad dropped
  // (L1-only). No L2 width attr for the old 14rem rail — ledgered.
  // Visibility is structural `<when {tablet}>` — global `.s-displaynone`
  // beats `when: { tablet: display(flex) }`.
  // no preset: display flex / dir column / overflow x hidden y auto
  style rail = { display(flex).dir(column).size(hug).overflow({ x: hidden, y: auto }).bg(canvas).fg(canvas) }
  style railLabel = { fg(muted).type(label) }
  // no preset: display block / hover+current when
  style railLink = { display(block).radius(md).fg(muted).type(mono), when: { hover: fg(canvas).bg(hover-bg), current: fg(primary).bg(muted) } }
  // no preset: display block / hover+current when
  style railHub = { display(block).radius(md).fg(muted).type(body-sm), when: { hover: fg(canvas).bg(hover-bg), current: fg(primary).bg(muted) } }
  // no preset: overflow x hidden / y auto
  style mainPane = { size(grow).overflow({ x: hidden, y: auto }) }
  style contentCol = { size(grow) }
  style mobileLabel = { fg(muted).type(mono-sm) }
  style drawerTitle = { type(body-sm) }
  style fullW = { size(max) }

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

export default controller((ctx: { route?: { pathname?: () => string } }) => {
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
  // Loops, not Array.filter — Identities scanner treats `filter(` as L1-only.
  const sections = derived(() => {
    const lang: CatalogEntry[] = []
    for (const e of CATALOG) {
      if (e.pack === 'lang') lang.push(e)
    }
    const out: { cat: CatalogEntry['cat']; label: string; items: CatalogEntry[] }[] = []
    for (const cat of CAT_ORDER) {
      const items: CatalogEntry[] = []
      for (const e of lang) {
        if (e.cat === cat) items.push(e)
      }
      if (items.length > 0) out.push({ cat, label: CAT_LABEL[cat], items })
    }
    return out
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

<row nowrap align="stretch" gap="0" style(shell)>
  <when {tablet}>
    <nav label="Components" testid="components-rail" use:scrollRestore={'components-rail'} style(rail)>
      <components-nav path={path} currentSlug={currentSlug} sections={sections} onNav={noopNav}></components-nav>
    </nav>
  </when>

  <column gap="0" style(contentCol)>
    <when {tablet}>
    {else =>
    <!-- no preset: cluster attr (is_known_axis_prop); nowrap is the working hug row -->
    <row nowrap gap="2" align="center">
      <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
      <small style(mobileLabel)>Components</small>
    </row>
    }</when>
    <main style(mainPane)>
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
