<!--
  <config-surface> — project/runtime settings from shell + project root.
  Nav: buttons. Fields: table. Product copy only.
-->
<scope>
  const eyebrow = style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground)
  const muted = style.fontSize(sm).fg(muted-foreground)
</scope>

<controller>
import { createConfigSurface } from '../config/config-surface-vm'
export default controller(() => {
  const c = createConfigSurface()
  return {
    categories: c.categories,
    sections: c.sections,
    sectionTitle: c.sectionTitle,
    sectionLead: c.sectionLead,
    onCatClick: (e: Event) => c.onCatClick(e),
  }
})
</controller>

<column
  surface="config"
  config-surface
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden).bg(background)}
>
  <row
    config-workbench
    grow
    gap="0"
    nowrap
    align="stretch"
    sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden)}
  >
    <column
      config-nav
      grow
      gap="0"
      sx={style.minH(0).h(full).minW(panel-min-w-sm).maxW(panel-max-w-sm).basis(0).borderRWidth(1).borderR(border-subtle).bg(surface-1).overflow(hidden)}
    >
      <column shrink="0" sx={style.padX(3).padT(3).padB(2).borderBWidth(1).borderB(border-subtle)}>
        <p sx={eyebrow}>Config</p>
      </column>
      <scroll-area config-cat-list grow on:click={onCatClick} sx={style.minH(0).basis(0).h(full)}>
        <column gap="1" sx={style.pad(2)}>
          <for {categories()} as="cat" key={cat.key}>
            <button
              type="button"
              config-cat={cat.id}
              size="sm"
              variant={cat.selected ? 'soft' : 'ghost'}
              tone={cat.selected ? 'primary' : 'muted'}
              aria-current={cat.selected ? 'true' : undefined}
              sx={style.w(full)}
            >
              {cat.label}
            </button>
          </for>
        </column>
      </scroll-area>
    </column>

    <column
      config-main
      grow
      gap="0"
      sx={style.minH(0).h(full).minW(0).basis(0).bg(background).overflow(hidden)}
    >
      <column
        config-head
        shrink="0"
        gap="1"
        sx={style.padX(5).padY(3).borderBWidth(1).borderB(border-subtle).bg(surface-0)}
      >
        <p config-section-title sx={style.fontSize(lg).fontWeight(semibold).tracking(tight)}>
          {sectionTitle()}
        </p>
        <p config-section-lead sx={muted}>{sectionLead()}</p>
      </column>

      <scroll-area config-body grow sx={style.minH(0).basis(0).h(full)}>
        <column gap="3" sx={style.pad(4).minW(0)}>
          <for {sections()} as="sec" key={sec.key}>
            <if {sec.selected}>
              <column
                config-block={sec.id}
                gap="0"
                sx={style.minW(0).radius(md).borderWidth(1).border(border-subtle).bg(surface-0).overflow(hidden)}
              >
                <if {sec.showShipCloud}>
                  <column gap="0" sx={style.pad(3).borderBWidth(1).borderB(border-subtle)}>
                    <card plugin="ship-cloud" padding="md" sx={style.minW(0).borderWidth(1).border(primary).bg(surface-1)}>
                      <row gap="3" align="start" nowrap>
                        <column grow min-w="0" gap="1">
                          <row gap="2" align="center" nowrap>
                            <p sx={style.fontSize(md).fontWeight(semibold)}>Ship Cloud</p>
                            <badge tone="primary" size="sm" variant="soft" pill>official</badge>
                          </row>
                          <p sx={muted}>Account, sync, and hosted agents when the plugin host is available.</p>
                        </column>
                        <button
                          type="button"
                          size="sm"
                          variant="default"
                          disabled
                          title="Install path not available"
                          install-ship-cloud
                          shrink="0"
                        >
                          Install…
                        </button>
                      </row>
                    </card>
                  </column>
                </if>

                <if {sec.showDanger}>
                  <column section="danger" gap="3" sx={style.pad(4).borderBWidth(1).borderB(border-subtle).bg(surface-1)}>
                    <row gap="2" nowrap>
                      <button
                        type="button"
                        size="sm"
                        variant="default"
                        tone="destructive"
                        disabled
                        title="Not available"
                        reset-studio
                      >
                        Reset Studio state…
                      </button>
                      <button
                        type="button"
                        size="sm"
                        variant="ghost"
                        tone="destructive"
                        disabled
                        title="Not available"
                        purge-theme-drafts
                      >
                        Purge theme drafts…
                      </button>
                    </row>
                  </column>
                </if>

                <if {sec.showFields}>
                  <table
                    config-fields
                    config-table={sec.id}
                    w="full"
                    dense
                    hoverable
                    sx={style.w(full)}
                  >
                    <thead>
                      <tr>
                        <th>Setting</th>
                        <th>Value</th>
                        <th>Source</th>
                      </tr>
                    </thead>
                    <tbody>
                      <for {sec.fields} as="f" key={f.key}>
                        <tr config-field={f.key}>
                          <td>
                            <p sx={style.fontSize(sm).fontWeight(semibold)}>{f.label}</p>
                          </td>
                          <td>
                            <p sx={style.fontSize(sm).fontFamily(mono)}>{f.value}</p>
                          </td>
                          <td>
                            <badge tone="muted" size="sm" variant="outline" pill>{f.source}</badge>
                          </td>
                        </tr>
                      </for>
                    </tbody>
                  </table>
                </if>
              </column>
            </if>
          </for>
        </column>
      </scroll-area>
    </column>
  </row>
</column>
