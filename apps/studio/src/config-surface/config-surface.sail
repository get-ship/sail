<!--
  <config-surface> — project/runtime settings. Nav categories + field tables.
-->
<scope>
  style eyebrow = { fg(muted).type(label) }
  style muted = { fg(muted).type(body-sm) }
  style surfaceRoot = { size(grow).overflow(hidden).bg(app) }
  style workbench = { size(grow).overflow(hidden) }
  // no preset: edge-only border
  style navPane = { size(grow).overflow(hidden).border({ edge: r, color: border-subtle, width: 1 }).bg(surface) }
  // no preset: edge-only border
  style navHead = { size(hug).pad(3).border({ edge: b, color: border-subtle, width: 1 }) }
  style scrollGrow = { size(grow) }
  style navListPad = { pad(2) }
  style fullBtn = { size(max) }
  style mainPane = { size(grow).overflow(hidden).bg(app) }
  // no preset: edge-only border
  style mainHead = { size(hug).pad(5).border({ edge: b, color: border-subtle, width: 1 }).bg(app) }
  style sectionTitle = { type(heading) }
  // no preset: min-w lock
  style bodyPad = { pad(4).size({ min-w: 0 }) }
  // no preset: min-w lock
  style configBlock = { size({ min-w: 0 }).overflow(hidden).radius(md).border(subtle).bg(app) }
  // no preset: edge-only border
  style blockPad = { pad(3).border({ edge: b, color: border-subtle, width: 1 }) }
  // no preset: edge-only border
  style dangerBand = { pad(4).border({ edge: b, color: border-subtle, width: 1 }).bg(surface) }
  // no preset: min-w lock + tone border
  style cloudCard = { size({ min-w: 0 }).border({ color: primary, width: 1 }).bg(surface) }
  style growMin = { size(grow) }
  style shrink0 = { size(hug) }
  style cardTitle = { type(heading) }
  style fieldLabel = { type(body-sm) }
  style fieldValue = { type(mono) }
  style tableFill = { size(max) }
  // no preset: type wrap
  style chipLabel = { type({ wrap: truncate }) }
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
                            <badge tone="primary" size="sm" variant="soft" pill style(chipLabel)>official</badge>
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
