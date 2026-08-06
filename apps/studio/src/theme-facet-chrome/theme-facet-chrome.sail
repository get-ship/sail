<!--
  <theme-facet-chrome> — shared Themes facet chrome band.
  Receives session via identity bind: session={session}
  Path crumb, sun/moon mode switch, Save/Add.
  No overflow menu — delete / remix live on Themes home.
  Dirty leave: Save / Discard / Cancel dialog.
-->
<controller>
import { mountAmbient } from '@ship/sail-runtime'
import type { ThemeSession } from '../theme-session/theme-session'

export default controller(() => {
  const sessionSig = signal<ThemeSession | null>(null)
  /**
   * Real signal for bindOpen → showModal() (top layer + backdrop).
   * Static `open` is non-modal and nests in the chrome column — density UI
   * bled through and Discard was clipped. Mirror session leaveOpen here.
   */
  const leaveOpen = signal(false)

  // Ambient lifecycle mount (do not import { mount } — that is the host view-mounter).
  mount((host) => {
    const pre = (host as unknown as { session?: ThemeSession | null }).session
    if (pre != null) sessionSig.set(pre)
    Object.defineProperty(host, 'session', {
      configurable: true,
      get: () => sessionSig(),
      set: (s: ThemeSession | null) => {
        sessionSig.set(s)
      },
    })
  })

  mountAmbient(() =>
    effect(() => {
      leaveOpen.set(sessionSig()?.leaveOpen() ?? false)
    }),
  )

  const s = () => sessionSig()
  const nameEdit = () => s()?.nameEdit() ?? ''
  const canEdit = () => s()?.canEdit() ?? false
  const isCatalog = () => s()?.isCatalog() ?? false
  const isDirty = () => s()?.isDirty() ?? false
  const canUndo = () => s()?.canUndo() ?? false
  const canRedo = () => s()?.canRedo() ?? false
  const hasDiffRows = () => s()?.hasDiffRows() ?? false
  const diffRows = () => s()?.diffRows() ?? []
  const diffMenuId = () => `theme-${s()?.chromeMarker ?? 'facet'}-diff-menu`
  const canToggleMode = () => s()?.canToggleMode() ?? false
  const showAddMode = () => s()?.showAddMode() ?? false
  const addModeLabel = () => s()?.addModeLabel() ?? 'Add mode'
  const modeDarkOn = () => s()?.modeDarkOn() ?? false
  const stateChipLabel = () => s()?.stateChipLabel() ?? ''
  const stateChipTone = () => s()?.stateChipTone() ?? 'muted'
  const showStateChip = () => s()?.showStateChip() ?? false
  const pathCrumb = () => s()?.pathCrumb() ?? 'Overview'
  const pathMenuId = () => s()?.pathMenuId ?? 'theme-path-menu'
  const pathNav = () => s()?.pathNav() ?? []
  const saveButtonLabel = () => s()?.saveButtonLabel() ?? 'Save'

  const goBack = () => s()?.goBack()
  const onNameInput = (e: Event) => s()?.onNameInput(e)
  const onNameChange = (e: Event) => s()?.onNameChange(e)
  const onModeSwitch = (e: Event) => s()?.onModeSwitch(e)
  const onAddMode = () => s()?.onAddMode()
  const onSave = () => s()?.onSave()
  const onUndo = () => s()?.onUndo?.()
  const onRedo = () => s()?.onRedo?.()
  const onAddToProject = () => s()?.onAddToProject()
  const onPathSelect = (e: Event) => s()?.onPathSelect(e)
  const onLeaveSave = () => s()?.onLeaveSave()
  const onLeaveDiscard = () => s()?.onLeaveDiscard()
  const onLeaveCancel = () => s()?.onLeaveCancel()

  return {
    nameEdit,
    canEdit,
    isCatalog,
    isDirty,
    canUndo,
    canRedo,
    hasDiffRows,
    diffRows,
    diffMenuId,
    canToggleMode,
    showAddMode,
    addModeLabel,
    modeDarkOn,
    stateChipLabel,
    stateChipTone,
    showStateChip,
    pathCrumb,
    pathMenuId,
    pathNav,
    saveButtonLabel,
    leaveOpen,
    goBack,
    onNameInput,
    onNameChange,
    onModeSwitch,
    onAddMode,
    onSave,
    onUndo,
    onRedo,
    onAddToProject,
    onPathSelect,
    onLeaveSave,
    onLeaveDiscard,
    onLeaveCancel,
  }
})
</controller>

<column theme-facet-chrome-root gap="0">
  style({  box: shrink(0).minW(0)  })

  <row theme-facet-chrome align="center" justify="between" gap="3" nowrap>
    style({  box: shrink(0).padX(4).h(10).minH(10).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(background)  })

    <row align="center" gap="2" nowrap>
      style({  box: grow(1).minW(0)  })

      <button type="button" size="sm" variant="ghost" tone="muted" facet-back on:click={goBack} label="Back to Themes">
        Themes
      </button>
      <p hidden="true">
        style({ paint: fg(muted-foreground), text: fontSize(sm) })
/</p>
      <input
        type="text"
        theme-name
        size="sm"
        use:autosize
        value={nameEdit()}
        label="Theme name"
        spellcheck="false"
        disabled={!canEdit()}
        on:input={onNameInput}
        on:change={onNameChange}

>
        style({ box: maxW(48), paint: borderWidth(0), text: fontWeight(semibold) })
      </input>
      <p hidden="true">
        style({ paint: fg(muted-foreground), text: fontSize(sm) })
/</p>
      <button
        type="button"
        size="sm"
        variant="ghost"
        tone="muted"
        popovertarget={pathMenuId()}
        path-trigger
        label="Section"
      >
        {pathCrumb()}
        <icon name="chevron-down" size="sm"></icon>
      </button>
      <dropdown id={pathMenuId()} placement="bottom-start" on:sail-menu-select={onPathSelect}>
        <for {pathNav()} as="item" key={item.key}>
          <menu-item value={item.value}>
            <icon name={item.icon} size="sm">
              style({ box: shrink(0) })
</icon>
            {item.label}
          </menu-item>
        </for>
      </dropdown>
      <if {showStateChip()}>
        <badge state-chip tone={stateChipTone()} size="sm" variant="outline" pill>
          style({ box: shrink(0) })

          {stateChipLabel()}
        </badge>
      </if>
    </row>

    <!-- Edit cluster (save / unsaved / undo) — separate from mode switch on the right. -->
    <row align="center" gap="3" nowrap>
      style({ box: shrink(0) })

      <row edit-cluster align="center" gap="1" nowrap>
        style({ box: shrink(0) })

        <button type="button" size="sm" variant="ghost" tone="muted" facet-undo label="Undo" disabled={!canUndo()} on:click={onUndo}>
          style({ box: shrink(0) })

          <icon name="undo-2" size="sm"></icon>
        </button>
        <button type="button" size="sm" variant="ghost" tone="muted" facet-redo label="Redo" disabled={!canRedo()} on:click={onRedo}>
          style({ box: shrink(0) })

          <icon name="redo-2" size="sm"></icon>
        </button>
        <!-- Dropdown always mounted (not inside isDirty) so use:anchor can bind to the trigger. -->
        <if {isDirty()}>
          <button type="button" size="sm" variant="soft" tone="primary" unsaved-trigger popovertarget={diffMenuId()} label="Unsaved changes">
            style({ box: shrink(0) })

            Unsaved
            <icon name="chevron-down" size="sm"></icon>
          </button>
        </if>
        <dropdown id={diffMenuId()} placement="bottom-end">
          <column gap="2">
            style({ box: pad(3).minW(panel-min-w-md).maxW(panel-max-w-md) })

            <p>
              style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold) })
Unsaved changes</p>
            <if {hasDiffRows()}>
              <scroll-area>
                style({ box: maxH(48) })

                <column gap="2">
                  <for {diffRows()} as="row" key={row.key}>
                    <column gap="0" diff-row>
                      <p>
                        style({ paint: fg(foreground), text: fontSize(xs).fontFamily(mono).truncate() })
{row.path}</p>
                      <p>
                        style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono).truncate() })

                        {row.from} → {row.to}
                      </p>
                    </column>
                  </for>
                </column>
              </scroll-area>
            </if>
            <if {!hasDiffRows()}>
              <p>
                style({ paint: fg(muted-foreground), text: fontSize(xs) })
Edits pending — save to write theme JSON.</p>
            </if>
            <button type="button" size="sm" variant="solid" tone="primary" diff-save on:click={onSave}>
              Save
            </button>
          </column>
        </dropdown>
        <if {canEdit()}>
          <button type="button" size="sm" variant="solid" tone="primary" facet-save on:click={onSave}>
            style({ box: shrink(0) })

            {saveButtonLabel()}
          </button>
        </if>
        <if {isCatalog()}>
          <button type="button" size="sm" variant="solid" tone="primary" facet-add-project on:click={onAddToProject}>
            style({ box: shrink(0) })

            Add to project
          </button>
        </if>
      </row>

      <stack hidden="true">
        style({ box: w(half).h(5).minW(half), paint: bg(border-subtle) })
</stack>

      <row mode-cluster align="center" gap="2" nowrap>
        style({ box: shrink(0) })

        <if {canToggleMode()}>
          <row dual-mode gap="2" align="center" nowrap label="Theme mode">
            style({ box: shrink(0) })

            <icon name="sun" size="sm" label="Light">
              style({ box: shrink(0), paint: fg(muted-foreground) })
</icon>
            <switch
              size="sm"
              tone="muted"
              mode-switch
              checked={modeDarkOn()}
              label={modeDarkOn() ? 'Switch to light mode' : 'Switch to dark mode'}
              on:change={onModeSwitch}
            ></switch>
            <icon name="moon" size="sm" label="Dark">
              style({ box: shrink(0), paint: fg(muted-foreground) })
</icon>
          </row>
        </if>
        <if {showAddMode()}>
          <button type="button" size="sm" variant="ghost" tone="muted" add-mode on:click={onAddMode}>
            style({ box: shrink(0) })

            {addModeLabel()}
          </button>
        </if>
      </row>
    </row>
  </row>

  <!--
    Dirty leave — keep <dialog> mounted (not inside <if>). bindIf renders the
    branch into a DocumentFragment and only then inserts; bindOpen's showModal
    requires a connected element, so if+open=true never entered the top layer
    (density bled through a closed/zero-size dialog). Always-mounted +
    open={leaveOpen()} → bindOpen showModal when dirty leave fires.
    ESC / backdrop → sail-close → cancel leave (pending navigation dropped).
  -->
  <dialog
    theme-leave-dialog
    open={leaveOpen()}
    size="md"
    on:sail-close={onLeaveCancel}
  >
    <header id="theme-leave-title">Unsaved theme changes</header>
    <section>
      Save writes the theme to the project. Discard drops in-memory edits.
    </section>
    <footer>
      <button type="button" size="sm" variant="ghost" tone="muted" leave-cancel on:click={onLeaveCancel}>
        Cancel
      </button>
      <button type="button" size="sm" variant="outline" tone="destructive" leave-discard on:click={onLeaveDiscard}>
        Discard
      </button>
      <button type="button" size="sm" variant="solid" tone="primary" leave-save on:click={onLeaveSave}>
        Save
      </button>
    </footer>
  </dialog>
</column>
