<!--
  <config-surface> — project/runtime settings. Nav categories + field tables.
-->
<scope>
  style eyebrow = { fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style surfaceRoot = { grow(1).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }).bg(background) }
  style workbench = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
  style navPane = { grow(1).size({h: full, min-w: panel-min-w-sm, min-h: 0, max-w: panel-max-w-sm, basis: 0}).overflow({ all: hidden }).border({ edge: r, color: border-subtle, width: 1 }).bg(surface-1) }
  style navHead = { shrink(0).pad({ x: 3, t: 3, b: 2 }).border({ edge: b, color: border-subtle, width: 1 }) }
  style scrollGrow = { grow(1).size({h: full, min-h: 0, basis: 0}) }
  style navListPad = { pad(2) }
  style fullBtn = { size({ w: full }) }
  style mainPane = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }).bg(background) }
  style mainHead = { shrink(0).pad({ x: 5, y: 3 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-0) }
  style sectionTitle = { type({ size: lg, weight: semibold, tracking: tight }) }
  style bodyPad = { pad(4).size({ min-w: 0 }) }
  style configBlock = { size({ min-w: 0 }).overflow({ all: hidden }).radius(md).border({ color: border-subtle, width: 1 }).bg(surface-0) }
  style blockPad = { pad(3).border({ edge: b, color: border-subtle, width: 1 }) }
  style dangerBand = { pad(4).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
  style cloudCard = { size({ min-w: 0 }).border({ color: primary, width: 1 }).bg(surface-1) }
  style growMin = { grow(1).size({ min-w: 0 }) }
  style shrink0 = { shrink(0) }
  style cardTitle = { type({ size: md, weight: semibold }) }
  style fieldLabel = { type({ size: sm, weight: semibold }) }
  style fieldValue = { type({ family: mono, size: sm }) }
  style tableFill = { size({ w: full }) }
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
