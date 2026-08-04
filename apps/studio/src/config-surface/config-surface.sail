<!--
  <config-surface> — project / compiler / deck / studio / plugins scaffold.
  Read-only fixture fields; Ship Cloud official plugin card; danger zone disabled.
  Height chain: side cat nav + main sections. Ban-clean.
-->
<scope>
  const eyebrow = style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground)
  const muted = style.fontSize(sm).fg(muted-foreground)
  const monoVal = style.fontSize(sm).fontFamily(mono).fg(foreground)
</scope>

<controller>
import { createConfigSurface } from '../config/config-surface-vm'
export default controller(() => {
  const vm = createConfigSurface()
  return {
    categories: vm.categories,
    sectionTitle: vm.sectionTitle,
    sectionLead: vm.sectionLead,
    fields: vm.fields,
    showShipCloud: vm.showShipCloud,
    showDanger: vm.showDanger,
    onCatClick: (e: Event) => vm.onCatClick(e),
  }
})
</controller>

<column
  data-surface="config"
  data-config-surface
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden).bg(background)}
>
  <row
    data-config-workbench
    grow
    gap="0"
    nowrap
    align="stretch"
    sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden)}
  >
    <column
      data-config-nav
      grow
      gap="0"
      sx={style.minH(0).h(full).minW(panel-min-w-sm).maxW(panel-max-w-sm).basis(0).borderRWidth(1).borderR(border-subtle).bg(surface-0).overflow(hidden)}
    >
      <column shrink="0" gap="1" sx={style.padX(4).padT(4).padB(3).borderBWidth(1).borderB(border-subtle)}>
        <p sx={eyebrow}>Config</p>
      </column>
      <scroll-area data-config-cat-list grow on:click={onCatClick} sx={style.minH(0).basis(0).h(full)}>
        <column gap="1" sx={style.pad(2)}>
          <for {categories()} as="cat" key={cat.key}>
            <studio-list-row
              data-config-cat={cat.id}
              title={cat.label}
              selected={cat.selected}
            ></studio-list-row>
          </for>
        </column>
      </scroll-area>
    </column>

    <column
      data-config-main
      grow
      gap="0"
      sx={style.minH(0).h(full).minW(0).basis(0).bg(background).overflow(hidden)}
    >
      <column
        data-config-head
        shrink="0"
        gap="1"
        sx={style.padX(5).padY(4).borderBWidth(1).borderB(border-subtle).bg(surface-0)}
      >
        <p data-config-section-title sx={style.fontSize(lg).fontWeight(semibold).tracking(tight)}>
          {sectionTitle()}
        </p>
        <p data-config-section-lead sx={muted}>{sectionLead()}</p>
      </column>

      <scroll-area data-config-body grow sx={style.minH(0).basis(0).h(full)}>
        <column gap="4" sx={style.pad(5).minW(0)}>
          <if {showShipCloud()}>
            <card data-plugin="ship-cloud" padding="md" sx={style.minW(0).borderWidth(1).border(primary)}>
              <row gap="3" align="start" nowrap>
                <center
                  shrink="0"
                  sx={style.w(10).h(10).minW(10).minH(10).radius(md).bg(surface-2).borderWidth(1).border(border-subtle).fg(primary)}
                >
                  <p sx={style.fontSize(xs).fontWeight(semibold)}>SC</p>
                </center>
                <column grow min-w="0" gap="1">
                  <row gap="2" align="center" nowrap>
                    <p sx={style.fontSize(md).fontWeight(semibold)}>Ship Cloud</p>
                    <badge tone="primary" size="sm" variant="soft" pill>official</badge>
                  </row>
                  <p sx={muted}>
                    Account, project memory, sync, hosted agents. Fills Studio slots when installed —
                    OSS keeps empty honest states until then.
                  </p>
                </column>
                <button
                  type="button"
                  size="sm"
                  variant="default"
                  disabled
                  title="Plugin host install path not wired in scaffold"
                  data-install-ship-cloud
                  shrink="0"
                >
                  Install…
                </button>
              </row>
            </card>
          </if>

          <card data-config-fields padding="md" sx={style.minW(0)}>
            <column gap="0">
              <for {fields()} as="f" key={f.key}>
                <row
                  data-config-field={f.key}
                  gap="3"
                  align="center"
                  nowrap
                  sx={style.padY(3).borderBWidth(1).borderB(border-subtle)}
                >
                  <p shrink="0" sx={style.fontSize(sm).fontWeight(semibold).w(36)}>{f.label}</p>
                  <p grow min-w="0" sx={monoVal}>{f.value}</p>
                  <badge tone="muted" size="sm" variant="outline" pill shrink="0">{f.source}</badge>
                </row>
              </for>
            </column>
          </card>

          <if {showDanger()}>
            <card data-section="danger" padding="md" tone="destructive" sx={style.minW(0)}>
              <column gap="3">
                <p sx={style.fontSize(sm).fontWeight(semibold).fg(destructive)}>Danger zone</p>
                <p sx={muted}>Destructive ops are disabled in this scaffold.</p>
                <row gap="2" nowrap>
                  <button
                    type="button"
                    size="sm"
                    variant="default"
                    tone="destructive"
                    disabled
                    title="Not wired"
                    data-reset-studio
                  >
                    Reset Studio state…
                  </button>
                  <button
                    type="button"
                    size="sm"
                    variant="ghost"
                    tone="destructive"
                    disabled
                    title="Not wired"
                    data-purge-theme-drafts
                  >
                    Purge theme drafts…
                  </button>
                </row>
              </column>
            </card>
          </if>
        </column>
      </scroll-area>
    </column>
  </row>
</column>
