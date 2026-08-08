<!--
  <config-surface> — project/runtime settings from shell + project root.
  Nav: buttons. Fields: table. Product copy only.
-->
<scope>
  style eyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
</scope>

<controller>
import { createConfigSurface } from '../compositions/config/config-surface-vm'
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

<column surface="config" config-surface gap="0" style({  box: grow(1).minH(0).h(full).minW(0).overflow(hidden), paint: bg(background)  })>
  <row config-workbench gap="0" nowrap align="stretch" style({  box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden)  })>
    <column config-nav gap="0" style({  box: grow(1).minH(0).h(full).minW(panel-min-w-sm).maxW(panel-max-w-sm).basis(0).overflow(hidden), paint: borderRWidth(1).borderR(border-subtle).bg(surface-1)  })>
      <column style({  box: shrink(0).padX(3).padT(3).padB(2), paint: borderBWidth(1).borderB(border-subtle)  })>
        <p style(eyebrow)>Config</p>
      </column>
      <scroll-area config-cat-list on:click={onCatClick} style({  box: grow(1).minH(0).basis(0).h(full)  })>
        <column gap="1" style({ box: pad(2) })>
          <for {categories() as cat} key={cat.key}>
            <button
              type="button"
              config-cat={cat.id}
              size="sm"
              variant={cat.selected ? 'soft' : 'ghost'}
              tone={cat.selected ? 'primary' : 'muted'}
              current={cat.selected ? 'true' : undefined} style({ box: w(full) })>
              {cat.label}
            </button>
          </for>
        </column>
      </scroll-area>
    </column>

    <column config-main gap="0" style({  box: grow(1).minH(0).h(full).minW(0).basis(0).overflow(hidden), paint: bg(background)  })>
      <column config-head gap="1" style({  box: shrink(0).padX(5).padY(3), paint: borderBWidth(1).borderB(border-subtle).bg(surface-0)  })>
        <p config-section-title style({ text: fontSize(lg).fontWeight(semibold).tracking(tight) })>
          {sectionTitle()}
        </p>
        <p config-section-lead style(muted)>{sectionLead()}</p>
      </column>

      <scroll-area config-body style({  box: grow(1).minH(0).basis(0).h(full)  })>
        <column gap="3" style({ box: pad(4).minW(0) })>
          <for {sections() as sec} key={sec.key}>
            <if {sec.selected}>
              <column
                config-block={sec.id}
                gap="0" style({ box: minW(0).overflow(hidden), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-0) })>
                <if {sec.showShipCloud}>
                  <column gap="0" style({ box: pad(3), paint: borderBWidth(1).borderB(border-subtle) })>
                    <card plugin="ship-cloud" padding="md" style({ box: minW(0), paint: borderWidth(1).border(primary).bg(surface-1) })>
                      <row gap="3" align="start" nowrap>
                        <column gap="1" style({ box: grow(1).minW(0) })>
                          <row gap="2" align="center" nowrap>
                            <p style({ text: fontSize(md).fontWeight(semibold) })>Ship Cloud</p>
                            <badge tone="primary" size="sm" variant="soft" pill>official</badge>
                          </row>
                          <p style(muted)>Account, sync, and hosted agents when the plugin host is available.</p>
                        </column>
                        <button type="button" size="sm" variant="default" disabled title="Install path not available" install-ship-cloud style({ box: shrink(0) })>
                          Install…
                        </button>
                      </row>
                    </card>
                  </column>
                </if>

                <if {sec.showDanger}>
                  <column section="danger" gap="3" style({ box: pad(4), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) })>
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
                  <table config-fields config-table={sec.id} dense hoverable style({  box: w(full)  })>
                    <thead>
                      <tr>
                        <th>Setting</th>
                        <th>Value</th>
                        <th>Source</th>
                      </tr>
                    </thead>
                    <tbody>
                      <for {sec.fields as f} key={f.key}>
                        <tr config-field={f.key}>
                          <td>
                            <p style({ text: fontSize(sm).fontWeight(semibold) })>{f.label}</p>
                          </td>
                          <td>
                            <p style({ text: fontSize(sm).fontFamily(mono) })>{f.value}</p>
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
