<!--
  <theme-facet-chrome> — shared Themes facet chrome band.
  Identity bind: session={session}. Path crumb, mode switch, Save/Add, leave dialog.
-->
<scope>
  style chromeRoot = { box: flow({ shrink: 0 }).size({ min-w: 0 }) }
  style chromeBar = { box: flow({ shrink: 0 }).space({ pad-x: 4 }).size({ h: 10, min-w: 0, min-h: 10 }), paint: border({ edge: b, color: border-subtle, width: 1 }).fill({ color: background }) }
  style growMin = { box: flow({ grow: 1 }).size({ min-w: 0 }) }
  style shrink0 = { box: flow({ shrink: 0 }) }
  style tapFloor = { box: flow({ shrink: 0 }).size({ min-w: control-h-sm, min-h: control-h-sm }) }
  style iconBtn = { box: flow({ shrink: 0 }).size({ w: control-h-sm, h: control-h-sm, min-w: control-h-sm, min-h: control-h-sm }) }
  style slashMuted = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style nameInput = { box: size({ max-w: 48 }), paint: border({ width: 0 }), text: type({ weight: semibold }) }
  style diffMenu = { box: space({ pad: 3 }).size({ min-w: panel-min-w-md, max-w: panel-max-w-md }) }
  style diffTitle = { paint: ink({ color: muted-foreground }), text: type({ size: xs, weight: semibold }) }
  style diffPath = { paint: ink({ color: foreground }), text: type({ family: mono, size: xs, wrap: truncate }) }
  style diffDelta = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs, wrap: truncate }) }
  style diffEmpty = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style modeIcon = { box: flow({ shrink: 0 }), paint: ink({ color: muted-foreground }) }
  style divider = { box: size({ w: half, h: 5, min-w: half }), paint: fill({ color: border-subtle }) }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'
import type { ThemeSession } from '../compositions/theme-session/theme-session'

export default controller(() => {
  const sessionSig = signal<ThemeSession | null>(null)
  // Real signal for bindOpen → showModal (top layer). Mirror session.leaveOpen.
  const leaveOpen = signal(false)

  mount((host) => {
    const pre = (host as { session?: ThemeSession | null }).session
    if (pre != null) sessionSig.set(pre)
    Object.defineProperty(host, 'session', {
      configurable: true,
      get: () => sessionSig(),
      set: (s: ThemeSession | null) => {
        sessionSig.set(s)
      },
    })
  })

  mount(() =>
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
  const onNameInput = (e: UiEvent) => s()?.onNameInput(e)
  const onNameChange = (e: UiEvent) => s()?.onNameChange(e)
  const onModeSwitch = (e: UiEvent) => s()?.onModeSwitch(e)
  const onAddMode = () => s()?.onAddMode()
  const onSave = () => s()?.onSave()
  const onUndo = () => s()?.onUndo?.()
  const onRedo = () => s()?.onRedo?.()
  const onAddToProject = () => s()?.onAddToProject()
  const onPathSelect = (detail: { value?: string }) => s()?.onPathSelect(detail)
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

<column theme-facet-chrome-root gap="0" style(chromeRoot)>
  <row theme-facet-chrome align="center" justify="between" gap="3" nowrap style(chromeBar)>
    <row align="center" gap="2" nowrap style(growMin)>
      <button type="button" size="sm" variant="ghost" tone="muted" facet-back on:click={goBack} label="Back to Themes" style(tapFloor)>
        Themes
      </button>
      <p hidden="true" style(slashMuted)>/</p>
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
        on:change={onNameChange} style(nameInput)>
      <p hidden="true" style(slashMuted)>/</p>
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
      <dropdown id={pathMenuId()} placement="bottom-start" on-select={onPathSelect}>
        <for {pathNav() as item} key={item.key}>
          <menu-item value={item.value}>
            <icon name={item.icon} size="sm" style({ box: flow({ shrink: 0 }) })></icon>
            {item.label}
          </menu-item>
        </for>
      </dropdown>
      <if {showStateChip()}>
        <badge state-chip tone={stateChipTone()} size="sm" variant="outline" pill style(shrink0)>
          {stateChipLabel()}
        </badge>
      </if>
    </row>

    <row align="center" gap="3" nowrap style(shrink0)>
      <row edit-cluster align="center" gap="1" nowrap style(shrink0)>
        <button type="button" size="iconSm" variant="ghost" tone="muted" facet-undo label="Undo" disabled={!canUndo()} on:click={onUndo} style(iconBtn)>
          <icon name="undo-2" size="sm"></icon>
        </button>
        <button type="button" size="iconSm" variant="ghost" tone="muted" facet-redo label="Redo" disabled={!canRedo()} on:click={onRedo} style(iconBtn)>
          <icon name="redo-2" size="sm"></icon>
        </button>
        <!-- Diff dropdown always mounted so the trigger can open it when dirty. -->
        <if {isDirty()}>
          <button type="button" size="sm" variant="soft" tone="primary" unsaved-trigger popovertarget={diffMenuId()} label="Unsaved changes" style(shrink0)>
            Unsaved
            <icon name="chevron-down" size="sm"></icon>
          </button>
        </if>
        <dropdown id={diffMenuId()} placement="bottom-end">
          <column gap="2" style(diffMenu)>
            <p style(diffTitle)>Unsaved changes</p>
            <if {hasDiffRows()}>
              <scroll-area style({ box: size({ max-h: 48 }) })>
                <column gap="2">
                  <for {diffRows() as row} key={row.key}>
                    <column gap="0" diff-row>
                      <p style(diffPath)>{row.path}</p>
                      <p style(diffDelta)>
                        {row.from} → {row.to}
                      </p>
                    </column>
                  </for>
                </column>
              </scroll-area>
            </if>
            <if {!hasDiffRows()}>
              <p style(diffEmpty)>Edits pending — save to write theme JSON.</p>
            </if>
            <button type="button" size="sm" variant="solid" tone="primary" diff-save on:click={onSave}>
              Save
            </button>
          </column>
        </dropdown>
        <if {canEdit()}>
          <button type="button" size="sm" variant="solid" tone="primary" facet-save on:click={onSave} style(shrink0)>
            {saveButtonLabel()}
          </button>
        </if>
        <if {isCatalog()}>
          <button type="button" size="sm" variant="solid" tone="primary" facet-add-project on:click={onAddToProject} style(shrink0)>
            Add to project
          </button>
        </if>
      </row>

      <stack hidden="true" style(divider)></stack>

      <row mode-cluster align="center" gap="2" nowrap style(shrink0)>
        <if {canToggleMode()}>
          <row dual-mode gap="2" align="center" nowrap label="Theme mode" style(shrink0)>
            <icon name="sun" size="sm" label="Light" style(modeIcon)></icon>
            <switch
              size="sm"
              tone="muted"
              mode-switch
              checked={modeDarkOn()}
              label={modeDarkOn() ? 'Switch to light mode' : 'Switch to dark mode'}
              on:change={onModeSwitch}
            ></switch>
            <icon name="moon" size="sm" label="Dark" style(modeIcon)></icon>
          </row>
        </if>
        <if {showAddMode()}>
          <button type="button" size="sm" variant="ghost" tone="muted" add-mode on:click={onAddMode} style(shrink0)>
            {addModeLabel()}
          </button>
        </if>
      </row>
    </row>
  </row>

  <!-- Leave dialog always mounted — bindOpen needs a connected host for showModal. -->
  <dialog
    theme-leave-dialog
    open={leaveOpen()}
    size="md"
    on-close={onLeaveCancel}
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
