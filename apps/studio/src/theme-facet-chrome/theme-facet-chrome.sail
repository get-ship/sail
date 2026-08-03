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

<column data-theme-facet-chrome-root shrink="0" gap="0" sx={style.minW(0)}>
  <row
    data-theme-facet-chrome
    align="center"
    justify="between"
    gap="3"
    shrink="0"
    nowrap
    sx={style.padX(4).h(10).minH(10).borderBWidth(1).borderB(border-subtle).bg(background).minW(0)}
  >
    <row align="center" gap="2" grow min-w="0" nowrap sx={style.minW(0)}>
      <button type="button" size="sm" variant="ghost" tone="muted" data-facet-back on:click={goBack} aria-label="Back to Themes">
        Themes
      </button>
      <p sx={style.fg(muted-foreground).fontSize(sm)} aria-hidden="true">/</p>
      <input
        type="text"
        data-theme-name
        size="sm"
        use:autosize
        value={nameEdit()}
        aria-label="Theme name"
        spellcheck="false"
        disabled={!canEdit()}
        on:input={onNameInput}
        on:change={onNameChange}
        sx={style.maxW(48).fontWeight(semibold).borderWidth(0)}
      />
      <p sx={style.fg(muted-foreground).fontSize(sm)} aria-hidden="true">/</p>
      <button
        type="button"
        size="sm"
        variant="ghost"
        tone="muted"
        popovertarget={pathMenuId()}
        data-path-trigger
        aria-haspopup="listbox"
        aria-label="Section"
      >
        {pathCrumb()}
        <icon name="chevron-down" size="sm"></icon>
      </button>
      <dropdown id={pathMenuId()} placement="bottom-start" on:sail-menu-select={onPathSelect}>
        <for {pathNav()} as="item" key={item.key}>
          <menu-item value={item.value}>
            <icon name={item.icon} size="sm" shrink="0"></icon>
            {item.label}
          </menu-item>
        </for>
      </dropdown>
      <if {showStateChip()}>
        <badge data-state-chip tone={stateChipTone()} size="sm" variant="outline" pill shrink="0">
          {stateChipLabel()}
        </badge>
      </if>
    </row>

    <!-- Edit cluster (save / unsaved / undo) — separate from mode switch on the right. -->
    <row align="center" gap="3" shrink="0" nowrap>
      <row data-edit-cluster align="center" gap="1" shrink="0" nowrap>
        <button
          type="button"
          size="sm"
          variant="ghost"
          tone="muted"
          data-facet-undo
          aria-label="Undo"
          disabled={!canUndo()}
          on:click={onUndo}
          shrink="0"
        >
          <icon name="undo-2" size="sm"></icon>
        </button>
        <button
          type="button"
          size="sm"
          variant="ghost"
          tone="muted"
          data-facet-redo
          aria-label="Redo"
          disabled={!canRedo()}
          on:click={onRedo}
          shrink="0"
        >
          <icon name="redo-2" size="sm"></icon>
        </button>
        <!-- Dropdown always mounted (not inside isDirty) so use:anchor can bind to the trigger. -->
        <if {isDirty()}>
          <button
            type="button"
            size="sm"
            variant="soft"
            tone="primary"
            shrink="0"
            data-unsaved-trigger
            popovertarget={diffMenuId()}
            aria-haspopup="menu"
            aria-label="Unsaved changes"
          >
            Unsaved
            <icon name="chevron-down" size="sm"></icon>
          </button>
        </if>
        <dropdown id={diffMenuId()} placement="bottom-end">
          <column gap="2" sx={style.pad(3).minW(panel-min-w-md).maxW(panel-max-w-md)}>
            <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>Unsaved changes</p>
            <if {hasDiffRows()}>
              <scroll-area sx={style.maxH(48)}>
                <column gap="2">
                  <for {diffRows()} as="row" key={row.key}>
                    <column gap="0" data-diff-row>
                      <p sx={style.fontSize(xs).fontFamily(mono).fg(foreground).truncate()}>{row.path}</p>
                      <p sx={style.fontSize(xs).fontFamily(mono).fg(muted-foreground).truncate()}>
                        {row.from} → {row.to}
                      </p>
                    </column>
                  </for>
                </column>
              </scroll-area>
            </if>
            <if {!hasDiffRows()}>
              <p sx={style.fontSize(xs).fg(muted-foreground)}>Edits pending — save to write theme JSON.</p>
            </if>
            <button type="button" size="sm" variant="solid" tone="primary" data-diff-save on:click={onSave}>
              Save
            </button>
          </column>
        </dropdown>
        <if {canEdit()}>
          <button type="button" size="sm" variant="solid" tone="primary" data-facet-save on:click={onSave} shrink="0">
            {saveButtonLabel()}
          </button>
        </if>
        <if {isCatalog()}>
          <button type="button" size="sm" variant="solid" tone="primary" data-facet-add-project on:click={onAddToProject} shrink="0">
            Add to project
          </button>
        </if>
      </row>

      <stack aria-hidden="true" sx={style.w(half).h(5).minW(half).bg(border-subtle)}></stack>

      <row data-mode-cluster align="center" gap="2" shrink="0" nowrap>
        <if {canToggleMode()}>
          <row data-dual-mode gap="2" align="center" nowrap shrink="0" aria-label="Theme mode">
            <icon name="sun" size="sm" label="Light" shrink="0" sx={style.fg(muted-foreground)}></icon>
            <switch
              size="sm"
              tone="muted"
              data-mode-switch
              checked={modeDarkOn()}
              aria-label={modeDarkOn() ? 'Switch to light mode' : 'Switch to dark mode'}
              on:change={onModeSwitch}
            ></switch>
            <icon name="moon" size="sm" label="Dark" shrink="0" sx={style.fg(muted-foreground)}></icon>
          </row>
        </if>
        <if {showAddMode()}>
          <button type="button" size="sm" variant="ghost" tone="muted" data-add-mode on:click={onAddMode} shrink="0">
            {addModeLabel()}
          </button>
        </if>
      </row>
    </row>
  </row>

  <!--
    Dirty leave — mount only when open (closed dialog + overflow:hidden is a
    sail-layout trap against content height). open={leaveOpen()} → bindOpen →
    showModal(): top layer + backdrop so Overview density (etc.) cannot bleed
    through; actions use dialog footer anatomy so they never clip.
    ESC / backdrop → sail-close → cancel leave (pending navigation dropped).
  -->
  <if {leaveOpen()}>
    <dialog
      data-theme-leave-dialog
      open={leaveOpen()}
      size="md"
      aria-labelledby="theme-leave-title"
      on:sail-close={onLeaveCancel}
    >
      <header id="theme-leave-title">Unsaved theme changes</header>
      <section>
        Save writes the theme to the project. Discard drops in-memory edits.
      </section>
      <footer>
        <button type="button" size="sm" variant="ghost" tone="muted" data-leave-cancel on:click={onLeaveCancel}>
          Cancel
        </button>
        <button type="button" size="sm" variant="outline" tone="destructive" data-leave-discard on:click={onLeaveDiscard}>
          Discard
        </button>
        <button type="button" size="sm" variant="solid" tone="primary" data-leave-save on:click={onLeaveSave}>
          Save
        </button>
      </footer>
    </dialog>
  </if>
</column>
