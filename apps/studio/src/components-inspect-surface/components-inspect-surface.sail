<!--
  <components-inspect-surface> — Inspect mode: wide IR tree + instance detail.
  Composition: { session, inspect }. Library is the sibling route surface.
-->
<scope>
  style eyebrow = { fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style pathMeta = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style surfaceRoot = { grow(1).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }).bg(background) }
  style workbench = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
  style treePane = { grow(1).size({h: full, min-w: panel-min-w-lg, min-h: 0, basis: 0}).overflow({ all: hidden }).border({ edge: r, color: border-subtle, width: 1 }).bg(surface-0) }
  style treeHead = { shrink(0).pad({ x: 4, t: 4, b: 3 }).border({ edge: b, color: border-subtle, width: 1 }) }
  style detailPane = { grow(1).size({h: full, min-w: panel-min-w-md, min-h: 0, basis: 0}).overflow({ all: hidden }).bg(background) }
  style detailHead = { shrink(0).pad({ x: 5, y: 4 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-0) }
  style loadRow = { shrink(0).pad({ x: 4, y: 2 }) }
  style treeBody = { grow(1).size({min-h: 0, basis: 0}).pad(1).overflow({ all: hidden }) }
  style scrollGrow = { grow(1).size({h: full, min-h: 0, basis: 0}) }
  style bodyPad = { pad(5) }
  style titleLg = { type({ size: lg, weight: semibold }) }
  style nodeTag = { type({ family: mono, size: lg, weight: semibold }) }
  style treeHost = { size({ w: full, h: full, min-h: 0 }) }
  style shrink0 = { shrink(0) }
  style growMin = { grow(1).size({ min-w: 0 }) }
  style msgSm = { type({ size: sm }) }
  style monoSm = { shrink(0).type({ family: mono, size: sm }) }
  style stateName = { grow(1).size({ min-w: 0 }).type({ family: mono, size: sm, wrap: truncate }) }
  style pickerMax = { size({ max-h: 28 }) }
</scope>

<controller>
import { createComponentsSession } from '../compositions/components-session/components-session'
import { createComponentsInspect } from '../compositions/components-inspect/components-inspect'
export default controller(() => {
  const session = createComponentsSession({ mode: 'inspect' })
  const inspect = createComponentsInspect(session)
  return { session, inspect }
})
</controller>

<column surface="components" components-surface components-inspect gap="0" style(surfaceRoot)>
  <row components-workbench gap="0" nowrap align="stretch" style(workbench)>
    <column inspect-tree-pane gap="0" style(treePane)>
      <column inspect-tree-head gap="2" style(treeHead)>
        <row align="center" justify="between" nowrap gap="2">
          <p style(eyebrow)>Components</p>
        </row>
        <row mode-toggle gap="1" align="center" nowrap>
          <button type="button" size="sm" mode="library" variant="ghost" tone="muted" on:click={() => session.onModeToggle('library')}>Library</button>
          <button type="button" size="sm" mode="inspect" variant="solid" tone="primary" pressed="true" on:click={() => session.onModeToggle('inspect')}>Inspect</button>
        </row>
        <p style(pathMeta)>Surface picker · open files & routes</p>
        <scroll-area surface-picker surface-list style(pickerMax)>
          <column gap="1">
            <for {inspect.surfaces() as s} key={s.key}>
              <studio-list-row
                surface-path={s.value}
                title={s.label}
                selected={s.selected}
                on:click={() => inspect.onSurfacePick(s.value)}
              ></studio-list-row>
            </for>
          </column>
        </scroll-area>
        <input
          type="search"
          size="sm"
          placeholder="Filter tree…"
          tree-filter
          value={inspect.treeFilter()}
          on:input={inspect.onTreeFilterInput}
        />
      </column>

      <if {inspect.isLoading()}>
        <row align="center" gap="2" style(loadRow)>
          <spinner size="sm"></spinner>
          <p style(muted)>{inspect.loadingLabel()}</p>
        </row>
      </if>

      <column gap="0" style(treeBody)>
        <if {!inspect.hasSurface()}>
          <studio-empty title="Select a surface to inspect" size="sm">
            Pick an open file or route surface. Trees come from source IR — not a live DOM clone.
          </studio-empty>
        </if>
        <if {inspect.hasSurface()}>
          <if {!inspect.hasTree()}>
            <studio-empty title="No tree yet" size="sm">
              Open this surface in Edit to load the model, then return — or wait for inspect ops.
            </studio-empty>
          </if>
          <if {inspect.hasTree()}>
            <sail-tree
              inspect-tree-host
              tree={inspect.tree}
              label="Instance tree"
              selectable="single"
              size="sm"
              show-guides
              on-select={inspect.onTreeSelect} style(treeHost)></sail-tree>
          </if>
        </if>
      </column>
    </column>

    <column inspect-detail components-detail gap="0" style(detailPane)>
      <column gap="3" style(detailHead)>
        <if {!inspect.hasNode()}>
          <p style(titleLg)>Instance</p>
          <p style(muted)>Select a node in the tree.</p>
        </if>
        <if {inspect.hasNode()}>
          <column gap="1">
            <p node-tag style(nodeTag)>{inspect.nodeTag()}</p>
            <p style(pathMeta)>{inspect.nodePath()} · {inspect.nodeSpanLabel()}</p>
            <row gap="2" nowrap>
              <if {inspect.nodeLayer()}>
                <badge tone="muted" size="sm" variant="outline" pill>{inspect.nodeLayer()}</badge>
              </if>
            </row>
          </column>
          <row gap="2" nowrap>
            <button type="button" size="sm" variant="solid" tone="primary" open-edit-node on:click={inspect.onOpenNodeInEdit}>
              Open in Edit
            </button>
            <button
              type="button"
              size="sm"
              variant="ghost"
              show-in-library
              disabled={!inspect.canShowInLibrary()}
              on:click={inspect.onShowInLibrary}
            >
              Show in library
            </button>
          </row>
        </if>
      </column>

      <scroll-area style(scrollGrow)>
        <column gap="4" style(bodyPad)>
          <if {inspect.panelsNote()}>
            <p style(muted)>{inspect.panelsNote()}</p>
          </if>

          <column inspect-issues gap="2">
            <p style(eyebrow)>Issues</p>
            <if {!inspect.hasIssues()}>
              <p style(muted)>No diagnostics on this surface.</p>
            </if>
            <for {inspect.issueRows() as iss} key={iss.key}>
              <row gap="2" align="start" nowrap>
                <badge tone={iss.tone} size="sm" variant="soft" pill style(shrink0)>{iss.severity}</badge>
                <column gap="0" style(growMin)>
                  <p style(msgSm)>{iss.message}</p>
                  <p style(pathMeta)>{iss.spanLabel}</p>
                </column>
              </row>
            </for>
          </column>

          <column inspect-layout gap="2">
            <p style(eyebrow)>Layout</p>
            <if {!inspect.hasLayout()}>
              <p style(muted)>No layout boxes for this surface yet.</p>
            </if>
            <if {inspect.hasLayout()}>
              <p style(muted)>Boxes from the compiler model (not browser computed style).</p>
              <for {inspect.layoutRows() as box} key={box.key}>
                <row gap="2" align="center" nowrap style({ pad({ y: 1 }) })>
                  <p style(monoSm)>{box.tag}</p>
                  <p style(pathMeta, { grow(1).size({ min-w: 0 }) })>{box.dims}</p>
                  <if {box.flags}>
                    <badge tone="warning" size="sm" variant="soft" pill style(shrink0)>{box.flags}</badge>
                  </if>
                </row>
              </for>
            </if>
          </column>

          <column inspect-state gap="2">
            <p style(eyebrow)>State</p>
            <if {!inspect.hasState()}>
              <p style(muted)>No state bindings on this surface.</p>
            </if>
            <for {inspect.stateRows() as st} key={st.key}>
              <row gap="2" align="center" nowrap>
                <p style(stateName)>{st.name}</p>
                <badge tone="muted" size="sm" variant="outline" pill>{st.kind}</badge>
                <badge tone={st.tierTone} size="sm" variant="soft" pill>{st.tierLabel}</badge>
              </row>
            </for>
          </column>
        </column>
      </scroll-area>
    </column>
  </row>

  <toast-region position="bottom-end" components-toast-region>
    <for {session.toastItems() as t} key={t.key}>
      <toast tone={t.tone} components-toast on-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
