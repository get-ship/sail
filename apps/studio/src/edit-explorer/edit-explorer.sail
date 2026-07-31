<!--
  <edit-explorer> — collapsible dir groups + keyboard file tree.
  Panel grammar: header · toolbar · tree · outline. Ban-clean.
-->
<controller>
import { createEditExplorer } from './edit-explorer-vm'
export default controller(() => {
  const vm = createEditExplorer()
  return {
    treeGroups: vm.treeGroups,
    hasFiles: vm.hasFiles,
    filterText: vm.filterText,
    activeDescendant: vm.activeDescendant,
    onFilterInput: (e: Event) => vm.onFilterInput(e),
    onFileClick: (e: Event) => vm.onFileClick(e),
    onTreeKeydown: (e: Event) => vm.onTreeKeydown(e),
  }
})
</controller>

<column
  data-edit-explorer
  shrink="0"
  gap="0"
  sx={style.w(60).minH(0).h(full).borderRWidth(1).borderR(border).bg(surface-1).overflow(hidden)}
>
  <row align="center" shrink="0" sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle)}>
    <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
      Explorer
    </p>
  </row>
  <column shrink="0" gap="1" sx={style.padX(2).padY(1).borderBWidth(1).borderB(border-subtle)}>
    <input
      type="search"
      data-edit-filter
      placeholder="Filter files"
      value={filterText()}
      on:input={onFilterInput}
      aria-label="Filter files"
      size="sm"
      sx={style.w(full)}
    />
    <p sx={style.fontSize(xs).fg(muted-foreground)}>Click a file to open it in the editor.</p>
  </column>
  <scroll-area
    data-file-tree
    role="tree"
    tabindex="0"
    aria-label="Project files"
    aria-activedescendant={activeDescendant()}
    grow
    sx={style.minH(0).basis(0)}
    on:click={onFileClick}
    on:keydown={onTreeKeydown}
  >
    <column gap="0" sx={style.padY(1)}>
      <if {!hasFiles()}>
        <empty-state variant="placeholder" size="sm" w="full" data-files-empty sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
          <strong slot="title">No files</strong>
          Open a project to list Sail sources.
        </empty-state>
      </if>
      <for {treeGroups()} as="g" key={g.key}>
        <collapsible open={true} data-tree-group={g.id}>
          <row slot="trigger" gap="2" align="center" sx={style.h(6).padX(2)}>
            <icon name="folder" size="sm"></icon>
            <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
              {g.label}
            </p>
          </row>
          <column gap="0" role="group" aria-label={g.label}>
            <for {g.files} as="f" key={f.key}>
              <button
                type="button"
                id={f.domId}
                role="treeitem"
                tabindex={f.tabIndex}
                data-file-path={f.path}
                data-selected={f.selected ? 'true' : 'false'}
                data-focused={f.focused ? 'true' : 'false'}
                aria-selected={f.selected ? 'true' : 'false'}
                size="sm"
                variant={f.selected || f.focused ? 'soft' : 'ghost'}
                tone={f.selected ? 'primary' : 'muted'}
                sx={style.w(full).h(6).padX(3).radius(none).when({ hover: style.bg(hover-bg) })}
              >
                <icon name="file-code-2" size="sm"></icon>
                <p grow min-w="0" sx={style.truncate().fontSize(sm).textAlign(start)}>{f.name}</p>
                <p data-file-meta sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}>{f.metaLabel}</p>
                <if {f.hasChecks}>
                  <p sx={style.fontSize(xs).fg(destructive)}>{f.checksLabel}</p>
                </if>
              </button>
            </for>
          </column>
        </collapsible>
      </for>
    </column>
  </scroll-area>

  <edit-element-tree grow sx={style.minH(0).borderTWidth(1).borderT(border)} />
</column>
