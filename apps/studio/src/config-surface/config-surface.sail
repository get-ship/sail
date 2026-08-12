<!--
  <config-surface> — project/runtime settings. Nav categories + field tables.
-->
<scope>
  style eyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style surfaceRoot = { box: grow(1).minH(0).h(full).minW(0).overflow(hidden), paint: bg(background) }
  style workbench = { box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden) }
  style navPane = { box: grow(1).minH(0).h(full).minW(panel-min-w-sm).maxW(panel-max-w-sm).basis(0).overflow(hidden), paint: borderRWidth(1).borderR(border-subtle).bg(surface-1) }
  style navHead = { box: shrink(0).padX(3).padT(3).padB(2), paint: borderBWidth(1).borderB(border-subtle) }
  style scrollGrow = { box: grow(1).minH(0).basis(0).h(full) }
  style navListPad = { box: pad(2) }
  style fullBtn = { box: w(full) }
  style mainPane = { box: grow(1).minH(0).h(full).minW(0).basis(0).overflow(hidden), paint: bg(background) }
  style mainHead = { box: shrink(0).padX(5).padY(3), paint: borderBWidth(1).borderB(border-subtle).bg(surface-0) }
  style sectionTitle = { text: fontSize(lg).fontWeight(semibold).tracking(tight) }
  style bodyPad = { box: pad(4).minW(0) }
  style configBlock = { box: minW(0).overflow(hidden), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-0) }
  style blockPad = { box: pad(3), paint: borderBWidth(1).borderB(border-subtle) }
  style dangerBand = { box: pad(4), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style cloudCard = { box: minW(0), paint: borderWidth(1).border(primary).bg(surface-1) }
  style growMin = { box: grow(1).minW(0) }
  style shrink0 = { box: shrink(0) }
  style cardTitle = { text: fontSize(md).fontWeight(semibold) }
  style fieldLabel = { text: fontSize(sm).fontWeight(semibold) }
  style fieldValue = { text: fontSize(sm).fontFamily(mono) }
  style tableFill = { box: w(full) }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'
import { createConfigSurface } from '../compositions/config/config-surface-vm'
export default controller(() => {
  const c = createConfigSurface()
  return {
    categories: c.categories,
    sections: c.sections,
    sectionTitle: c.sectionTitle,
    sectionLead: c.sectionLead,
    onCatClick: (e: UiEvent) => c.onCatClick(e),
  }
})
</controller>

<column surface="config" config-surface gap="0" style(surfaceRoot)>
  <row config-workbench gap="0" nowrap align="stretch" style(workbench)>
    <column config-nav gap="0" style(navPane)>
      <column style(navHead)>
        <p style(eyebrow)>Config</p>
      </column>
      <scroll-area config-cat-list on:click={onCatClick} style(scrollGrow)>
        <column gap="1" style(navListPad)>
          <for {categories() as cat} key={cat.key}>
            <button
              type="button"
              config-cat={cat.id}
              size="sm"
              variant={cat.selected ? 'soft' : 'ghost'}
              tone={cat.selected ? 'primary' : 'muted'}
              current={cat.selected ? 'true' : undefined}
              style(fullBtn)
            >
              {cat.label}
            </button>
          </for>
        </column>
      </scroll-area>
    </column>

    <column config-main gap="0" style(mainPane)>
      <column config-head gap="1" style(mainHead)>
        <p config-section-title style(sectionTitle)>
          {sectionTitle()}
        </p>
        <p config-section-lead style(muted)>{sectionLead()}</p>
      </column>

      <scroll-area config-body style(scrollGrow)>
        <column gap="3" style(bodyPad)>
          <for {sections() as sec} key={sec.key}>
            <if {sec.selected}>
              <column config-block={sec.id} gap="0" style(configBlock)>
                <if {sec.showShipCloud}>
                  <column gap="0" style(blockPad)>
                    <card plugin="ship-cloud" padding="md" style(cloudCard)>
                      <row gap="3" align="start" nowrap>
                        <column gap="1" style(growMin)>
                          <row gap="2" align="center" nowrap>
                            <p style(cardTitle)>Ship Cloud</p>
                            <badge tone="primary" size="sm" variant="soft" pill>official</badge>
                          </row>
                          <p style(muted)>Account, sync, and hosted agents when the plugin host is available.</p>
                        </column>
                        <button type="button" size="sm" variant="default" disabled title="Install path not available" install-ship-cloud style(shrink0)>
                          Install…
                        </button>
                      </row>
                    </card>
                  </column>
                </if>

                <if {sec.showDanger}>
                  <column section="danger" gap="3" style(dangerBand)>
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
                  <table config-fields config-table={sec.id} dense hoverable style(tableFill)>
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
                            <p style(fieldLabel)>{f.label}</p>
                          </td>
                          <td>
                            <p style(fieldValue)>{f.value}</p>
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
