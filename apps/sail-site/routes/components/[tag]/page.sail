<!--
  /components/[tag] — detail route. Live stage + prop rail via sail-component-viewer.
  L1 from component-details; pack tags from component-pack-meta.
  Full Mantine-style configurator corpus (4.8k lines) is staged separately — the
  prior megapage is parse/type-gate pathological in this worktree (compiler hang
  after policy resolve). This page keeps the product surface: specimens, props,
  usage, pack metadata — gate-green under sail-compiler.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).pad(4) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageColMd = { box: w(panel-max-w-md).maxW(full) }
  style fullW = { box: w(full) }
  style emptyEnter = { animate: enter({ opacity: 'fade', translate-y: 'spring:snappy' }) }
  style pressBtn = { animate: press({ scale: 'press' }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { DETAILS, type DetailEntry } from '../../../src/compositions/component-details'
import {
  packBlurbFor,
  packLabelFor,
  packPropsFor,
} from '../../../src/compositions/component-pack-meta'

export default controller((ctx: ControllerScope<Scope>) => {
  const tag = derived(() => ctx.route.params.tag?.() ?? '')
  const entry = derived(() => {
    const t = tag()
    const list = Object.values(DETAILS) as Array<
      DetailEntry & { key: string }
    >
    return list.find((e) => e.key === t) ?? null
  })
  const known = derived(() => entry() !== null)
  // Prefer packLabelFor over PACK_TAGS.has — region harness open-any stubs
  // erase Set methods; packLabelFor is a plain function that still works.
  const packLabel = derived(() => packLabelFor(tag()))
  const knownPack = derived(() => packLabel().length > 0)
  const showViewer = derived(() => known() || knownPack())
  const heading = derived(() => entry()?.heading ?? tag())
  const blurb = derived(() => entry()?.blurb ?? '')
  const props = derived(() => entry()?.props ?? [])
  const usageCode = derived(() => entry()?.usage ?? '')
  const packBlurb = derived(() => packBlurbFor(tag()))
  const packProps = derived(() => packPropsFor(tag()))
  const variants = derived(() => entry()?.variants ?? [])
  const variantAttr = derived(() => entry()?.variantAttr ?? '')
  const variant = signal('')
  const onVariantChange = (d: { variant?: string }) => {
    if (d.variant != null) variant.set(d.variant)
  }

  // Specimen state for live stages (zero-arg template reads).
  const presses = signal(0)
  const press = () => presses.set(presses() + 1)
  const name = signal('Ada Lovelace')
  const bio = signal('')
  const team = signal('Design')
  const notify = signal(true)
  const agreed = signal(false)
  const load = signal(62)
  const pageNo = signal(2)
  const goPage = (n: number) => pageNo.set(n)

  const stage = {
    button: () => tag() === 'button',
    a: () => tag() === 'a',
    input: () => tag() === 'input',
    textarea: () => tag() === 'textarea',
    select: () => tag() === 'select',
    switchToggle: () => tag() === 'switch',
    checkbox: () => tag() === 'checkbox',
    badge: () => tag() === 'badge',
    alert: () => tag() === 'alert',
    spinner: () => tag() === 'spinner',
    progressBar: () => tag() === 'progress',
    meter: () => tag() === 'meter',
    skeleton: () => tag() === 'skeleton',
    emptyState: () => tag() === 'empty-state',
    errorState: () => tag() === 'error-state',
    card: () => tag() === 'card',
    icon: () => tag() === 'icon',
    kbd: () => tag() === 'kbd',
    prose: () => tag() === 'prose',
    codeBlock: () => tag() === 'code-block',
    tabs: () => tag() === 'tabs',
    breadcrumb: () => tag() === 'breadcrumb',
    pagination: () => tag() === 'pagination',
    separator: () => tag() === 'separator',
    field: () => tag() === 'field',
    label: () => tag() === 'label',
    fieldset: () => tag() === 'fieldset',
    slider: () => tag() === 'slider',
    radio: () => tag() === 'radio',
    generic: () =>
      known() &&
      ![
        'button',
        'a',
        'input',
        'textarea',
        'select',
        'switch',
        'checkbox',
        'badge',
        'alert',
        'spinner',
        'progress',
        'meter',
        'skeleton',
        'empty-state',
        'error-state',
        'card',
        'icon',
        'kbd',
        'prose',
        'code-block',
        'tabs',
        'breadcrumb',
        'pagination',
        'separator',
        'field',
        'label',
        'fieldset',
        'slider',
        'radio',
      ].includes(tag()),
  }

  return {
    tag,
    entry,
    known,
    knownPack,
    showViewer,
    heading,
    blurb,
    props,
    usageCode,
    packBlurb,
    packLabel,
    packProps,
    variants,
    variantAttr,
    variant,
    onVariantChange,
    presses,
    press,
    name,
    bio,
    team,
    notify,
    agreed,
    load,
    pageNo,
    goPage,
    stage,
  }
})
</controller>

<column gap="4" style(page)>
  <if {showViewer()}>
    <sail-component-viewer
      name={heading()}
      blurb={knownPack() ? packBlurb() : blurb()}
      pack={knownPack() ? packLabel() : 'Component primitives'}
      variant-attr={variantAttr()}
      usage={usageCode()}
      variants={variants()}
      rows={knownPack() ? packProps() : props()}
      on-variant-change={onVariantChange}
    >
      <column slot="stage" gap="4" align="stretch" style(fullW)>
        <if {stage.button()}>
          <column gap="3" align="center" style(stageCol)>
            <button variant={variant() || 'solid'} tone="primary" style(pressBtn) on:click={press}>
              Save
            </button>
            <p style(monoXs)>pressed {presses()} · variant {variant() || 'solid'}</p>
          </column>
        </if>
        <if {stage.a()}>
          <row gap="3" align="center" style(stageCol)>
            <a href="#">Text link</a>
            <a href="#" variant="solid" tone="primary" size="sm">Button link</a>
          </row>
        </if>
        <if {stage.input()}>
          <column gap="2" style(stageCol)>
            <label for="demo-name" required>Display name</label>
            <input id="demo-name" placeholder="Ada Lovelace" bind:value={name} style(fullW) />
            <p style(monoXs)>value: {name() || '∅'}</p>
          </column>
        </if>
        <if {stage.textarea()}>
          <column gap="2" style(stageCol)>
            <textarea placeholder="Tell us about yourself" bind:value={bio} rows="3" style(fullW)></textarea>
            <p style(monoXs)>{bio().length} characters</p>
          </column>
        </if>
        <if {stage.select()}>
          <column gap="2" style(stageCol)>
            <select bind:value={team}>
              <option>Engineering</option>
              <option>Design</option>
              <option>Research</option>
            </select>
            <p style(monoXs)>team: {team()}</p>
          </column>
        </if>
        <if {stage.switchToggle()}>
          <row gap="3" align="center">
            <switch bind:checked={notify}></switch>
            <p style(mutedSm)>{notify() ? 'Notifications on' : 'Muted'}</p>
          </row>
        </if>
        <if {stage.checkbox()}>
          <label>
            <input type="checkbox" bind:checked={agreed} />
            I agree to the terms
          </label>
        </if>
        <if {stage.badge()}>
          <row gap="2" align="center">
            <badge tone="primary">Primary</badge>
            <badge tone="success" pill>Success</badge>
            <badge tone="warning">Warning</badge>
            <badge tone="destructive">Destructive</badge>
          </row>
        </if>
        <if {stage.alert()}>
          <alert tone="info" style(stageColMd)>
            <strong slot="title">Heads up</strong>
            A new version is available. Refresh when you get a chance.
          </alert>
        </if>
        <if {stage.spinner()}>
          <row gap="3" align="center">
            <spinner size="sm"></spinner>
            <spinner tone="primary"></spinner>
            <spinner size="lg" centered tone="primary"></spinner>
          </row>
        </if>
        <if {stage.progressBar()}>
          <column gap="2" style(stageCol)>
            <progress value={load()} max="100"></progress>
            <p style(monoXs)>{load()}% of 100</p>
          </column>
        </if>
        <if {stage.meter()}>
          <column gap="2" style(stageCol)>
            <meter value="72" min="0" max="100" low="33" high="66" optimum="100"></meter>
            <p style(monoXs)>Disk free — 72%</p>
          </column>
        </if>
        <if {stage.skeleton()}>
          <column gap="2" style(stageCol)>
            <skeleton shape="text" size="md"></skeleton>
            <skeleton shape="text" size="sm"></skeleton>
            <row gap="3" align="center">
              <skeleton shape="circle" size="md"></skeleton>
              <column gap="2" style({ box: grow(1) })>
                <skeleton shape="text" size="sm"></skeleton>
                <skeleton shape="text" size="sm"></skeleton>
              </column>
            </row>
          </column>
        </if>
        <if {stage.emptyState()}>
          <empty-state style(stageCol)>
            <icon slot="icon" name="inbox"></icon>
            <strong slot="title">Empty inbox</strong>
            Messages you receive land here.
            <button slot="action" variant="outline" size="sm">Compose</button>
          </empty-state>
        </if>
        <if {stage.errorState()}>
          <error-state code="ERR_THEME" size="sm" style(stageCol)>
            <strong slot="title">Couldn't load preview</strong>
            Rest chrome — override component.error-state leaves.
            <button slot="action" type="button" variant="outline" size="sm">Retry</button>
          </error-state>
        </if>
        <if {stage.card()}>
          <card style(stageCol)>
            <strong slot="title">Project card</strong>
            <p style(mutedSm)>Body copy sits in the card surface.</p>
          </card>
        </if>
        <if {stage.icon()}>
          <row gap="3" align="center">
            <icon name="check" size="sm"></icon>
            <icon name="sparkles"></icon>
            <icon name="refresh-cw" spin label="Refreshing"></icon>
          </row>
        </if>
        <if {stage.kbd()}>
          <row gap="2" align="center">
            <kbd>⌘</kbd>
            <kbd>K</kbd>
            <span style(mutedSm)>Open command palette</span>
          </row>
        </if>
        <if {stage.prose()}>
          <prose style(stageColMd)>
            <h2>Typography scope</h2>
            <p>
              Body copy with <strong>strong</strong>, <em>emphasis</em>,
              <code>inline()</code>, and <mark>mark</mark>.
            </p>
          </prose>
        </if>
        <if {stage.codeBlock()}>
          <code-block
            code={'const n = signal(0)\nconst double = derived(() => n() * 2)'}
            lang="ts"
          ></code-block>
        </if>
        <if {stage.tabs()}>
          <tabs style(fullW)>
            <tab-list>
              <tab>Overview</tab>
              <tab>Activity</tab>
              <tab>Settings</tab>
            </tab-list>
            <tab-panel>Overview panel</tab-panel>
            <tab-panel>Activity panel</tab-panel>
            <tab-panel>Settings panel</tab-panel>
          </tabs>
        </if>
        <if {stage.breadcrumb()}>
          <breadcrumb>
            <a href="/components">components</a>
            <small>{tag()}</small>
          </breadcrumb>
        </if>
        <if {stage.pagination()}>
          <pagination page={pageNo()} count="5" on-change={goPage}></pagination>
        </if>
        <if {stage.separator()}>
          <column gap="3" style(stageCol)>
            <p style(mutedSm)>Above</p>
            <separator></separator>
            <p style(mutedSm)>Below</p>
          </column>
        </if>
        <if {stage.field()}>
          <field style(stageCol)>
            <label slot="label" for="f-email">Email</label>
            <input id="f-email" type="email" placeholder="ada@lovelace.dev" />
          </field>
        </if>
        <if {stage.label()}>
          <column gap="1" style(stageCol)>
            <label for="lbl-demo" required>Email</label>
            <input id="lbl-demo" placeholder="ada@lovelace.dev" />
          </column>
        </if>
        <if {stage.fieldset()}>
          <fieldset style(stageCol)>
            <legend>Account details</legend>
            <column gap="2">
              <input placeholder="Name" />
              <input placeholder="Email" />
            </column>
          </fieldset>
        </if>
        <if {stage.slider()}>
          <column gap="2" style(stageCol)>
            <row justify="between"><span>Volume</span><span style(monoXs)>{load()}%</span></row>
            <slider bind:value={load} min="0" max="100"></slider>
          </column>
        </if>
        <if {stage.radio()}>
          <radio-group label="Plan" name="plan" value="monthly">
            <radio value="monthly">Monthly</radio>
            <radio value="yearly">Yearly</radio>
          </radio-group>
        </if>
        <if {stage.generic()}>
          <column gap="2" style(stageCol)>
            <p style(mutedSm)>
              {heading()} — live specimen. Props, variants, and usage are in the rail.
            </p>
            <p style(monoXs)>tag: {tag()}</p>
          </column>
        </if>
        <if {knownPack()}>
          <column gap="2" style(stageCol)>
            <p style(mutedSm)>{packLabel()} pack · {heading()}</p>
            <p style(monoXs)>{packBlurb()}</p>
          </column>
        </if>
      </column>
    </sail-component-viewer>
    <row gap="2">
      <a variant="default" href="/components">← All components</a>
      <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
    </row>
  {else =>
    <column gap="4">
      <breadcrumb>
        <a href="/components">components</a>
        <small>{tag()}</small>
      </breadcrumb>
      <empty-state style(emptyEnter)>
        <strong slot="title">Nothing named “{tag()}” here</strong>
        <p style(mutedSm)>Every element on the L1 surface has a detail page.</p>
      </empty-state>
      <row gap="2">
        <a variant="solid" tone="primary" href="/components">← All components</a>
      </row>
    </column>
  }</if>
</column>
