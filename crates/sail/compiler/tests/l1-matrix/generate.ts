#!/usr/bin/env bun
/**
 * Write fixtures/l1-matrix/routes/<host>/page.sail from the catalog.
 * Do not hand-author those pages. Re-run after TAGS / THEME_META change.
 *
 *   bun crates/sail/compiler/tests/l1-matrix/generate.ts
 */
import { mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'
import { COMPONENTS, type L1Tag } from '../../../../../packages/sail/l1/src/index.ts'
import { cells, isBoolAxis, tagOf } from './cells.ts'

const REPO = join(import.meta.dir, '../../../../..')
const FIXTURE = join(import.meta.dir, '../fixtures/l1-matrix')
const ROUTES = join(FIXTURE, 'routes')
const DUMP = join(FIXTURE, 'layout-diags.json')

const SKIP_PROPS = new Set([
  'onClose',
  'onDismiss',
  'onToggle',
  'onSelect',
  'onChange',
  'onPageChange',
  'onTabChange',
  'onToggleChange',
  'onThemeChange',
  'onErrorReset',
  'onErrorCaught',
  'onRemove',
])

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/"/g, '&quot;')
}

function attrString(tag: L1Tag, props: Record<string, string>): string {
  const parts: string[] = []
  for (const [name, value] of Object.entries(props)) {
    if (SKIP_PROPS.has(name)) continue
    if (value === undefined || value === '') continue
    if (isBoolAxis(tag, name) || tag.props.find((p) => p.name === name)?.type === 'bool') {
      if (value === 'true') parts.push(name)
      continue
    }
    const def = tag.props.find((p) => p.name === name)?.default
    if (def !== undefined && value === def) continue
    parts.push(`${name}="${esc(value)}"`)
  }
  return parts.join(' ')
}

function openTag(name: string, attrs: string): string {
  return attrs ? `<${name} ${attrs}>` : `<${name}>`
}

function idOf(specimen: string): string {
  return specimen.replace(/[^a-z0-9]+/gi, '-').replace(/^-|-$/g, '').toLowerCase()
}

/** Unique accessible name on the painted host (or trigger). */
function named(tag: string, specimen: string, props: Record<string, string>): string {
  const t = tagOf(tag)
  // Anatomy already stamps a product name — do not fight it with a fixture label.
  const anatomyNamed = new Set([
    'card',
    'code-block',
    'navbar',
    'page',
    'pane',
    'scroll-area',
    'skeleton',
    'spinner',
    'toolbar',
  ])
  const attrs = attrString(t, anatomyNamed.has(tag) ? props : { ...props, label: specimen })
  return attrs
}

function textHost(tag: string, specimen: string, props: Record<string, string>, body: string): string {
  return `${openTag(tag, named(tag, specimen, props))}${body}</${tag}>`
}

function voidHost(tag: string, specimen: string, props: Record<string, string>): string {
  return `${openTag(tag, named(tag, specimen, props))}</${tag}>`
}

function renderHost(tag: string, specimen: string, props: Record<string, string>): string {
  const t = tagOf(tag)
  const a = named(tag, specimen, props)
  const sid = idOf(specimen)

  switch (tag) {
    case 'a':
      return `${openTag('a', `${a} href="#"`) }${specimen}</a>`
    case 'accordion':
      return `${openTag('accordion', a)}
      <span slot="trigger">${specimen}</span>
      Accordion body.
    </accordion>`
    case 'alert':
      return `${openTag('alert', a)}
      <span slot="title">${specimen}</span>
      Alert body.
    </alert>`
    case 'avatar':
      return `${openTag('avatar', `${a} initials="AL" name="Ada Lovelace"`)}</avatar>`
    case 'badge':
      return textHost(tag, specimen, props, specimen)
    case 'breadcrumb':
      return `${openTag('breadcrumb', attrString(t, props))}
      <a href="#">Home</a>
      <a href="#">Rest</a>
      <small>${specimen}</small>
    </breadcrumb>`
    case 'button': {
      const icon = props.size === 'icon' || props.size === 'iconSm' || props.size === 'iconLg'
      if (icon) return `${openTag('button', a)}★</button>`
      return textHost(tag, specimen, props, specimen)
    }
    case 'card':
      return `${openTag('card', a)}${specimen}</card>`
    case 'checkbox':
      return voidHost(tag, specimen, props)
    case 'chip':
      return textHost(tag, specimen, props, specimen)
    case 'code-block':
      return `${openTag('code-block', `${a} code="const x = 1" lang="ts"`)}</code-block>`
    case 'color':
      return voidHost(tag, specimen, props)
    case 'combobox':
      return `${openTag('combobox', `${attrString(t, props)} placeholder="${esc(specimen)}"`)}
      <option value="apple">Apple</option>
      <option value="banana">Banana</option>
    </combobox>`
    case 'context-menu':
      return `<button id="${sid}-target">${specimen} target</button>
    ${openTag('context-menu', `${a} for="${sid}-target"`)}
      <menu-item value="cut">Cut</menu-item>
      <menu-item value="copy">Copy</menu-item>
    </context-menu>`
    case 'date':
      return voidHost(tag, specimen, props)
    case 'dialog':
      return `<button command="show-modal" commandfor="${sid}">Open ${specimen}</button>
    ${openTag('dialog', `${a} id="${sid}"`)}
      <header>${specimen}</header>
      <section>Dialog body.</section>
      <footer>
        <button command="close" commandfor="${sid}">Close ${specimen}</button>
      </footer>
    </dialog>`
    case 'drawer':
      return `<button command="show" commandfor="${sid}">Open ${specimen}</button>
    ${openTag('drawer', `${a} id="${sid}"`)}
      <header>${specimen}</header>
      <section>Drawer body.</section>
      <footer>
        <button command="close" commandfor="${sid}">Close ${specimen}</button>
      </footer>
    </drawer>`
    case 'dropdown':
      return `<button popovertarget="${sid}">Open ${specimen}</button>
    ${openTag('dropdown', `${a} id="${sid}"`)}
      <menu-item value="edit">Edit</menu-item>
      <menu-item value="dup">Duplicate</menu-item>
    </dropdown>`
    case 'empty-state':
      return `${openTag('empty-state', a)}
      <span slot="title">${specimen}</span>
      Nothing here.
    </empty-state>`
    case 'error-boundary':
      return `${openTag('error-boundary', a)}
      <p>${specimen}</p>
    </error-boundary>`
    case 'error-state':
      return `${openTag('error-state', a)}
      <span slot="title">${specimen}</span>
      Try again.
    </error-state>`
    case 'field':
      return `${openTag('field', a)}
      <label slot="label">${specimen}</label>
      <input placeholder="token" />
    </field>`
    case 'fieldset':
      return `${openTag('fieldset', a)}
      <legend>${specimen}</legend>
      <field>
        <label slot="label">Email</label>
        <input placeholder="you@ship.dev" />
      </field>
    </fieldset>`
    case 'file':
      return `${openTag('file', `${a} accept="image/*"`)}</file>`
    case 'form':
      return `${openTag('form', a)}
      <label for="${sid}">${specimen}</label>
      <input id="${sid}">
      <button type="submit">Send</button>
    </form>`
    case 'list-row':
      return textHost(tag, specimen, props, specimen)
    case 'hover-card':
      return `<button popovertarget="${sid}">Open ${specimen}</button>
    ${openTag('hover-card', `${a} id="${sid}"`)}
      <strong>${specimen}</strong>
    </hover-card>`
    case 'icon':
      return `${openTag('icon', `${a} name="check"`)}</icon>`
    case 'input':
      return openTag('input', `${a} placeholder="${esc(specimen)}"`)
    case 'kbd':
      return textHost(tag, specimen, props, specimen)
    case 'label': {
      const fid = `${idOf(specimen)}-field`
      return `${openTag('label', `${attrString(t, props)} for="${fid}"`)}${specimen}</label>
    <input id="${fid}" />`
    }
    case 'menu-item':
      return textHost(tag, specimen, props, specimen)
    case 'meter':
      return `${openTag('meter', `${a} value="0.4"`)}</meter>`
    case 'navbar':
      return `${openTag('navbar', a)}
      <span slot="brand">${specimen}</span>
    </navbar>`
    case 'page':
      return `${openTag('page', a)}
      <h1>${specimen}</h1>
    </page>`
    case 'pagination':
      return `${openTag('pagination', `${attrString(t, props)} value="1"`)}
      <page-item value="1">1</page-item>
      <page-item value="2">2</page-item>
    </pagination>`
    case 'popover':
      return `<button popovertarget="${sid}">Open ${specimen}</button>
    ${openTag('popover', `${a} id="${sid}"`)}
      ${specimen} body.
    </popover>`
    case 'progress':
      return `${openTag('progress', `${a} value="40"`)}</progress>`
    case 'prose':
      return `${openTag('prose', a)}
      <p>${specimen} with <strong>emphasis</strong>.</p>
    </prose>`
    case 'radio-group':
      return `${openTag('radio-group', `${a} name="${sid}" value="a"`)}
      <radio value="a">A</radio>
      <radio value="b">B</radio>
    </radio-group>`
    case 'resizable':
      return `${openTag('resizable', attrString(t, props))}</resizable>`
    case 'scroll-area':
      return `${openTag('scroll-area', a)}
      <p>${specimen} line one</p>
      <p>Line two</p>
    </scroll-area>`
    case 'select':
      return `${openTag('select', a)}
      <option>One</option>
      <option>Two</option>
    </select>`
    case 'separator':
      return voidHost(tag, specimen, props)
    case 'skeleton':
      return `${openTag('skeleton', attrString(t, props))}</skeleton>`
    case 'slider':
      return `${openTag('slider', `${a} value="40"`)}</slider>`
    case 'spinner':
      return `${openTag('spinner', attrString(t, props))}</spinner>`
    case 'switch':
      return `<label>${specimen} ${voidHost(tag, specimen, props)}</label>`
    case 'table': {
      const search = props.searchable === 'false' ? '' : 'searchable'
      return `${openTag('table', [attrString(t, props), search].filter(Boolean).join(' '))}
      <thead>
        <tr>
          <th>Name</th>
          <th>Team</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>Ada Lovelace</td>
          <td>Engines</td>
        </tr>
        <tr>
          <td>Katherine Johnson</td>
          <td>NASA</td>
        </tr>
      </tbody>
    </table>`
    }
    case 'tabs':
      return `${openTag('tabs', `${attrString(t, props)} active="one"`)}
      <tab name="one" title="${esc(specimen)}">Panel one</tab>
      <tab name="two" title="Two">Panel two</tab>
    </tabs>`
    case 'textarea':
      return `${openTag('textarea', `${a} placeholder="${esc(specimen)}"`)}</textarea>`
    case 'theme-picker':
      return `${openTag('theme-picker', attrString(t, props))}</theme-picker>`
    case 'ticker':
      return textHost(tag, specimen, props, specimen)
    case 'toast':
      return `<toast-region>
      ${openTag('toast', a)}
        <strong>${specimen}</strong> Notice body.
      </toast>
    </toast-region>`
    case 'toggle-group':
      return `${openTag('toggle-group', `${attrString(t, props)} value="day"`)}
      <toggle value="day">Day</toggle>
      <toggle value="week">Week</toggle>
    </toggle-group>`
    case 'tooltip':
      return `${openTag('tooltip', `${attrString(t, props)} content="${esc(specimen)}"`)}
      <button>${specimen}</button>
    </tooltip>`
    default:
      if (t.acceptsChildren) return textHost(tag, specimen, props, specimen)
      return voidHost(tag, specimen, props)
  }
}

function renderPart(tag: string, specimen: string, props: Record<string, string>): string {
  const t = tagOf(tag)
  const a = named(tag, specimen, props)
  switch (tag) {
    case 'caption':
      return `<table>
      ${openTag('caption', a)}${specimen}</caption>
      <tbody><tr><td>Ada</td></tr></tbody>
    </table>`
    case 'td':
      return `<table>
      <tbody><tr>${openTag('td', a)}${specimen}</td></tr></tbody>
    </table>`
    case 'th':
      return `<table>
      <thead><tr>${openTag('th', a)}${specimen}</th></tr></thead>
      <tbody><tr><td>—</td></tr></tbody>
    </table>`
    case 'legend':
      return `<fieldset>
      ${openTag('legend', a)}${specimen}</legend>
    </fieldset>`
    case 'page-item':
      return `<pagination value="1">
      ${openTag('page-item', `${a} value="1"`)}${specimen}</page-item>
      <page-item value="2">2</page-item>
    </pagination>`
    case 'radio':
      return `<radio-group name="${idOf(specimen)}" value="a">
      ${openTag('radio', `${a} value="a"`)}${specimen}</radio>
      <radio value="b">B</radio>
    </radio-group>`
    case 'tab':
      return `<tabs active="one">
      ${openTag('tab', `${attrString(t, props)} name="one" title="${esc(specimen)}"`)}Panel</tab>
      <tab name="two" title="Two">Two</tab>
    </tabs>`
    case 'tab-list':
      return `<tabs>
      ${openTag('tab-list', a)}
        <tab title="${esc(specimen)}"></tab>
        <tab title="Two"></tab>
      </tab-list>
      <tab-panel>${specimen} panel</tab-panel>
      <tab-panel>Two panel</tab-panel>
    </tabs>`
    case 'tab-panel':
      return `<tabs>
      <tab-list>
        <tab title="${esc(specimen)}"></tab>
      </tab-list>
      ${openTag('tab-panel', a)}${specimen} panel</tab-panel>
    </tabs>`
    case 'toggle':
      return `<toggle-group value="a">
      ${openTag('toggle', `${a} value="a"`)}${specimen}</toggle>
      <toggle value="b">B</toggle>
    </toggle-group>`
    case 'toast-region':
      return `${openTag('toast-region', attrString(t, props))}
      <toast><strong>${specimen}</strong> Notice.</toast>
    </toast-region>`
    default:
      return renderHost(tag, specimen, props)
  }
}

function renderSpecimen(tag: string, specimen: string, props: Record<string, string>): string {
  const t = tagOf(tag)
  const body = t.role === 'part' ? renderPart(tag, specimen, props) : renderHost(tag, specimen, props)
  return `  <column gap="2">
    <h2>${specimen}</h2>
    ${body}
  </column>`
}

function uniqueSpecs(
  component: string,
  all: ReturnType<typeof cells>,
): { tag: string; specimen: string; props: Record<string, string> }[] {
  const out: { tag: string; specimen: string; props: Record<string, string> }[] = []
  const seen = new Set<string>()
  for (const c of all) {
    if (c.component !== component) continue
    const key = `${c.tag}::${c.specimen}`
    if (seen.has(key)) continue
    seen.add(key)
    out.push({ tag: c.tag, specimen: c.specimen, props: c.props })
  }
  return out
}

function pageSource(component: string, all: ReturnType<typeof cells>): string {
  const specs = uniqueSpecs(component, all)
  const sections = specs.map((s) => renderSpecimen(s.tag, s.specimen, s.props)).join('\n\n')
  return `<!--
  GENERATED by crates/sail/compiler/tests/l1-matrix/generate.ts — do not hand-author.
  Probe locates each <h2> specimen via getByContract.
-->
<scope>
  style shell = { pad(6).bg(background) }
  style band = { size({ w: 100%, max-w: breakpoint-desktop }) }
</scope>

<column gap="8" style(shell)>
  <column gap="2" style(band)>
    <h1>${component} matrix</h1>
    <p>Generated specimens. Not the docs site.</p>
  </column>

${sections}
</column>
`
}

export function writeRoutes(): string[] {
  const written: string[] = []
  const all = cells()
  for (const comp of COMPONENTS) {
    const dir = join(ROUTES, comp.host)
    mkdirSync(dir, { recursive: true })
    const dest = join(dir, 'page.sail')
    writeFileSync(dest, pageSource(comp.host, all))
    written.push(dest)
  }
  return written
}

const LAYOUT_RULES = new Set(['sail-layout', 'sail-clipped-text', 'sail-unintended-overflow'])

export function dumpLayoutDiags(pages: string[]): void {
  const sail = join(REPO, 'target/debug/sail')
  const files: unknown[] = []
  for (const page of pages) {
    const rel = page.slice(FIXTURE.length + 1)
    const source = readFileSync(page, 'utf8')
    const proc = spawnSync(sail, ['op', 'SailAuthoring/LayoutCheck', '--json', JSON.stringify({ source })], {
      encoding: 'utf8',
      cwd: REPO,
      maxBuffer: 16 * 1024 * 1024,
    })
    if (proc.error) {
      throw new Error(`layout dump: sail missing at ${sail} — cargo build -p deck-cli --bin sail`)
    }
    if (proc.status !== 0 && !proc.stdout) {
      throw new Error(`layout dump failed for ${rel}: ${proc.stderr || proc.status}`)
    }
    let json: {
      diagnostics?: { kind?: string; rule?: string; message?: string; severity?: string; line?: number }[]
    }
    try {
      json = JSON.parse(proc.stdout)
    } catch {
      throw new Error(`layout dump: non-JSON from LayoutCheck for ${rel}: ${proc.stderr || proc.stdout}`)
    }
    const diagnostics = (json.diagnostics ?? []).filter((d) => d.rule && LAYOUT_RULES.has(d.rule))
    files.push({ path: rel, diagnostics })
  }
  writeFileSync(
    DUMP,
    JSON.stringify({ engine: 'taffy', generated: new Date().toISOString(), files }, null, 2) + '\n',
  )
}

function pruneStale(keep: Set<string>): void {
  for (const name of readdirSync(ROUTES, { withFileTypes: true })) {
    if (!name.isDirectory()) continue
    if (name.name === 'node_modules') continue
    const dest = join(ROUTES, name.name, 'page.sail')
    if (keep.has(dest)) continue
    // Leave the human rest-strip; only drop generated host dirs not in COMPONENTS.
    if (name.name) {
      rmSync(join(ROUTES, name.name), { recursive: true, force: true })
    }
  }
}

if (import.meta.main) {
  const pages = writeRoutes()
  pruneStale(new Set(pages))
  dumpLayoutDiags(pages)
  console.log(`wrote ${pages.length} routes + ${DUMP.slice(REPO.length + 1)}`)
}
