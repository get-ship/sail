<!--
  <components-inspect-surface> — Inspect mode: wide IR tree + instance detail.
  Composition: { session, inspect }. Library is the sibling route surface.
-->
<scope>
  const eyebrow = style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground)
  const pathMeta = style.fontSize(xs).fontFamily(mono).fg(muted-foreground)
  const muted = style.fontSize(sm).fg(muted-foreground)
</scope>

<controller>
import { createComponentsSession } from '../components-session/components-session'
import { createComponentsInspect } from '../components-inspect/components-inspect'
export default controller(() => {
  const session = createComponentsSession({ mode: 'inspect' })
  const inspect = createComponentsInspect(session)
  return { session, inspect }
})
</controller>

<column
  data-surface="components"
  data-components-surface
  data-components-inspect
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden).bg(background)}
>
  <row
    data-components-workbench
    grow
    gap="0"
    nowrap
    align="stretch"
    sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden)}
  >
    <column
      data-inspect-tree-pane
      grow
      gap="0"
      sx={style.minH(0).h(full).minW(panel-min-w-lg).basis(0).borderRWidth(1).borderR(border-subtle).bg(surface-0).overflow(hidden)}
    >
      <column data-inspect-tree-head shrink="0" gap="2" sx={style.padX(4).padT(4).padB(3).borderBWidth(1).borderB(border-subtle)}>
        <row align="center" justify="between" nowrap gap="2">
          <p sx={eyebrow}>Components</p>
        </row>
        <row data-mode-toggle gap="1" align="center" nowrap on:click={session.onModeToggle}>
          <button type="button" size="sm" data-mode="library" variant="ghost" tone="muted">Library</button>
          <button type="button" size="sm" data-mode="inspect" variant="solid" tone="primary" aria-pressed="true">Inspect</button>
        </row>
        <p sx={pathMeta}>Surface picker · open files & routes</p>
        <scroll-area data-surface-picker data-surface-list sx={style.maxH(28)} on:click={inspect.onSurfacePick}>
          <column gap="1">
            <for {inspect.surfaces()} as="s" key={s.key}>
              <studio-list-row
                data-surface-path={s.value}
                title={s.label}
                selected={s.selected}
              ></studio-list-row>
            </for>
          </column>
        </scroll-area>
        <input
          type="search"
          size="sm"
          placeholder="Filter tree…"
          data-tree-filter
          value={inspect.treeFilter()}
          on:input={inspect.onTreeFilterInput}
        />
      </column>

      <if {inspect.isLoading()}>
        <row align="center" gap="2" shrink="0" sx={style.padX(4).padY(2)}>
          <spinner size="sm"></spinner>
          <p sx={muted}>{inspect.loadingLabel()}</p>
        </row>
      </if>

      <column grow gap="0" sx={style.minH(0).basis(0).pad(1).overflow(hidden)}>
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
              data-inspect-tree-host
              tree={inspect.tree}
              label="Instance tree"
              selectable="single"
              size="sm"
              show-guides
              on:sail-select={inspect.onTreeSelect}
              sx={style.w(full).h(full).minH(0)}
            ></sail-tree>
          </if>
        </if>
      </column>
    </column>

    <column
      data-inspect-detail
      data-components-detail
      grow
      gap="0"
      sx={style.minH(0).h(full).minW(panel-min-w-md).basis(0).bg(background).overflow(hidden)}
    >
      <column shrink="0" gap="3" sx={style.padX(5).padY(4).borderBWidth(1).borderB(border-subtle).bg(surface-0)}>
        <if {!inspect.hasNode()}>
          <p sx={style.fontSize(lg).fontWeight(semibold)}>Instance</p>
          <p sx={muted}>Select a node in the tree.</p>
        </if>
        <if {inspect.hasNode()}>
          <column gap="1">
            <p data-node-tag sx={style.fontSize(lg).fontWeight(semibold).fontFamily(mono)}>{inspect.nodeTag()}</p>
            <p sx={pathMeta}>{inspect.nodePath()} · {inspect.nodeSpanLabel()}</p>
            <row gap="2" nowrap>
              <if {inspect.nodeLayer()}>
                <badge tone="muted" size="sm" variant="outline" pill>{inspect.nodeLayer()}</badge>
              </if>
            </row>
          </column>
          <row gap="2" nowrap>
            <button type="button" size="sm" variant="solid" tone="primary" data-open-edit-node on:click={inspect.onOpenNodeInEdit}>
              Open in Edit
            </button>
            <button
              type="button"
              size="sm"
              variant="ghost"
              data-show-in-library
              disabled={!inspect.canShowInLibrary()}
              on:click={inspect.onShowInLibrary}
            >
              Show in library
            </button>
          </row>
        </if>
      </column>

      <scroll-area grow sx={style.minH(0).basis(0).h(full)}>
        <column gap="4" sx={style.pad(5)}>
          <if {inspect.panelsNote()}>
            <p sx={muted}>{inspect.panelsNote()}</p>
          </if>

          <column data-inspect-issues gap="2">
            <p sx={eyebrow}>Issues</p>
            <if {!inspect.hasIssues()}>
              <p sx={muted}>No diagnostics on this surface.</p>
            </if>
            <for {inspect.issueRows()} as="iss" key={iss.key}>
              <row gap="2" align="start" nowrap>
                <badge tone={iss.tone} size="sm" variant="soft" pill shrink="0">{iss.severity}</badge>
                <column grow min-w="0" gap="0">
                  <p sx={style.fontSize(sm)}>{iss.message}</p>
                  <p sx={pathMeta}>{iss.spanLabel}</p>
                </column>
              </row>
            </for>
          </column>

          <column data-inspect-layout gap="2">
            <p sx={eyebrow}>Layout</p>
            <if {!inspect.hasLayout()}>
              <p sx={muted}>No layout boxes for this surface yet.</p>
            </if>
            <if {inspect.hasLayout()}>
              <p sx={muted}>Boxes from the compiler model (not browser computed style).</p>
              <for {inspect.layoutRows()} as="box" key={box.key}>
                <row gap="2" align="center" nowrap sx={style.padY(1)}>
                  <p shrink="0" sx={style.fontSize(sm).fontFamily(mono)}>{box.tag}</p>
                  <p grow min-w="0" sx={pathMeta}>{box.dims}</p>
                  <if {box.flags}>
                    <badge tone="warning" size="sm" variant="soft" pill shrink="0">{box.flags}</badge>
                  </if>
                </row>
              </for>
            </if>
          </column>

          <column data-inspect-state gap="2">
            <p sx={eyebrow}>State</p>
            <if {!inspect.hasState()}>
              <p sx={muted}>No state bindings on this surface.</p>
            </if>
            <for {inspect.stateRows()} as="st" key={st.key}>
              <row gap="2" align="center" nowrap>
                <p grow min-w="0" sx={style.fontSize(sm).fontFamily(mono).truncate()}>{st.name}</p>
                <badge tone="muted" size="sm" variant="outline" pill>{st.kind}</badge>
                <badge tone={st.tierTone} size="sm" variant="soft" pill>{st.tierLabel}</badge>
              </row>
            </for>
          </column>
        </column>
      </scroll-area>
    </column>
  </row>

  <toast-region position="bottom-end" data-components-toast-region>
    <for {session.toastItems()} as="t" key={t.key}>
      <toast tone={t.tone} data-components-toast on:sail-toast-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
