<!--
  <edit-explorer> — rail: L1 tabs (Routes / Files / Components) + content hosts.
  Tree views share one sail-tree (data swaps); Components hosts edit-element-tree
  without remount thrash (aria-current + when height). Ban-clean.
-->
<controller>
import { createEditExplorer } from './edit-explorer-vm'
export default controller(() => {
  const vm = createEditExplorer()
  return {
    filterText: vm.filterText,
    hasTree: vm.hasTree,
    railTab: vm.railTab,
    isTreeView: vm.isTreeView,
    isComponents: vm.isComponents,
    treeLabel: vm.treeLabel,
    emptyTitle: vm.emptyTitle,
    emptyBody: vm.emptyBody,
    onRailTabChange: (e: Event) => vm.onRailTabChange(e),
    onFilterInput: (e: Event) => vm.onFilterInput(e),
    onTreeSelect: (e: Event) => vm.onTreeSelect(e),
  }
})
</controller>

<studio-sidebar data-edit-explorer title="Explorer">
  <column
    slot="toolbar"
    shrink="0"
    gap="2"
    sx={style.pad(2).borderBWidth(1).borderB(border-subtle)}
  >
    <tabs
      data-rail-tabs
      active={railTab()}
      size="sm"
      aria-label="Explorer views"
      on:sail-tab-change={onRailTabChange}
      sx={style.w(full)}
    >
      <tab-list variant="pill" sx={style.w(full)}>
        <tab name="routes" grow min-w="0">Routes</tab>
        <tab name="files" grow min-w="0">Files</tab>
        <tab name="components" grow min-w="0">Components</tab>
      </tab-list>
    </tabs>
    <if {isTreeView()}>
      <input
        type="search"
        data-edit-filter
        placeholder="Filter"
        value={filterText()}
        on:input={onFilterInput}
        aria-label="Filter tree"
        size="sm"
        sx={style.w(full)}
      />
    </if>
  </column>

  <column
    data-file-tree
    grow
    gap="0"
    sx={style.minH(0).basis(0).h(full).padX(1).padY(1).overflow(hidden)}
  >
    <!-- Routes + Files: one sail-tree host; data swaps per railTab. Collapse preserved. -->
    <column
      data-rail-panel="tree"
      grow
      gap="0"
      aria-current={isTreeView() ? 'true' : undefined}
      aria-hidden={isTreeView() ? undefined : 'true'}
      sx={style.minH(0).basis(0).h(0).overflow(hidden).when({ current: style.h(full) })}
    >
      <if {!hasTree()}>
        <empty-state variant="placeholder" size="sm" w="full" data-files-empty sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
          <strong slot="title">{emptyTitle()}</strong>
          {emptyBody()}
        </empty-state>
      </if>
      <if {hasTree()}>
        <sail-tree
          data-file-tree-host
          label={treeLabel()}
          selectable="single"
          size="sm"
          show-guides
          on:sail-select={onTreeSelect}
          sx={style.w(full).h(full).minH(0)}
        ></sail-tree>
      </if>
    </column>

    <!-- Components: stay mounted (no <if> remount thrash when flipping tabs). -->
    <column
      data-rail-panel="components"
      data-components-tab
      grow
      gap="0"
      aria-current={isComponents() ? 'true' : undefined}
      aria-hidden={isComponents() ? undefined : 'true'}
      sx={style.minH(0).basis(0).h(0).overflow(hidden).when({ current: style.h(full) })}
    >
      <edit-element-tree grow sx={style.minH(0).h(full)} />
    </column>
  </column>
</studio-sidebar>
