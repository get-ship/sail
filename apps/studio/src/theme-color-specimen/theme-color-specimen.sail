<!--
  Color facet specimen — draft paint via theme-session host bind.
-->
<scope>
  style fieldHint = { fg(muted-foreground).type({ size: xs }) }
  style pathMeta = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style mutedSm = { fg(muted-foreground).type({ family: body, size: sm, leading: relaxed }) }
  style mutedRelaxed = { fg(muted-foreground).type({ size: xs, leading: relaxed }) }
  style specimenDock = { grow(1).size({h: full, min-w: 0, min-h: 0, max-w: panel-max-w-md, basis: 0}).overflow({ all: hidden }).border({ edge: l, color: border-subtle, width: 1 }).bg(background) }
  style specimenPad = { grow(1).size({ h: full, min-h: 0 }).pad(4) }
  style specimenFrame = { size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }).elevation(lg) }
  style fillCol = { grow(1).size({ h: full, min-h: 0 }) }
  style specimenChrome = { shrink(0).pad({ x: 3 }).size({ h: 8, min-w: 0, min-h: 8 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
  style trafficDots = { shrink(0).size({ min-w: 12 }) }
  style trafficDotDestructive = { shrink(0).size({ w: 2, h: 2, min-w: 2, min-h: 2 }).radius(full).bg(destructive) }
  style trafficDotWarning = { shrink(0).size({ w: 2, h: 2, min-w: 2, min-h: 2 }).radius(full).bg(warning) }
  style trafficDotSuccess = { shrink(0).size({ w: 2, h: 2, min-w: 2, min-h: 2 }).radius(full).bg(success) }
  style chromeTitle = { grow(1).size({ min-w: 0 }).fg(muted-foreground).type({ size: xs, wrap: truncate }) }
  style modeMeta = { shrink(0).fg(muted-foreground).type({ family: mono, size: xs }) }
  style scrollGrow = { grow(1).size({h: full, min-h: 0, basis: 0}).bg(background) }
  style bodyPad = { pad(5).size({ min-w: 0 }) }
  style productTitle = { type({ family: heading, size: md, weight: semibold }) }
  style fgOnSurface = { fg(foreground).type({ size: sm }) }
  style cardTitle = { type({ size: sm, weight: semibold }) }
  style surfaceBg = { pad(3).radius(md).border({ color: border-subtle, width: 1 }).bg(background) }
  style surface1 = { pad(3).radius(md).border({ color: border-subtle, width: 1 }).bg(surface-1) }
  style surface2 = { pad(3).radius(md).border({ color: border-subtle, width: 1 }).bg(surface-2) }
  style surface3 = { pad(3).radius(md).border({ color: border-subtle, width: 1 }).bg(surface-3) }
</scope>

<controller>
import type { ThemeSession } from '../compositions/theme-session/theme-session'
import { watchPreviewHost } from '../compositions/theme-session/bind-preview-host'

export default controller(() => {
  const sessionSig = signal<ThemeSession | null>(null)
  mount((host) => {
    const pre = (host as { session?: ThemeSession | null }).session
    if (pre != null) sessionSig.set(pre)
    Object.defineProperty(host, 'session', {
      configurable: true,
      get: () => sessionSig(),
      set: (s: ThemeSession | null) => sessionSig.set(s),
    })
    return watchPreviewHost(host, (el) => sessionSig()?.attachPreviewHost?.(el))
  })
  const s = () => sessionSig()
  return {
    paintMode: () => s()?.paintMode() ?? 'light',
    nameEdit: () => s()?.nameEdit() ?? '',
  }
})
</controller>

<column specimen-dock gap="0" style(specimenDock)>
  <column gap="0" style(specimenPad)>
    <card
      color-specimen
      preview-specimen
      mode={paintMode()}
      padding="none"
      style(specimenFrame)
    >
      <column gap="0" style(fillCol)>
        <row specimen-chrome align="center" gap="2" nowrap style(specimenChrome)>
          <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
            <stack style(trafficDotDestructive)></stack>
            <stack style(trafficDotWarning)></stack>
            <stack style(trafficDotSuccess)></stack>
          </row>
          <p style(chromeTitle)>{nameEdit()}</p>
          <p style(modeMeta)>{paintMode()}</p>
        </row>
        <scroll-area style(scrollGrow)>
          <column specimen-body gap="4" style(bodyPad)>
            <column gap="1">
              <p style(productTitle)>Product UI</p>
              <p style(mutedSm)>
                Brand + surfaces paint this host. Mode flips the color table, not the editor chrome.
              </p>
            </column>
            <row gap="2" align="center" nowrap specimen-buttons>
              <button type="button" size="sm" variant="solid" tone="primary">Primary</button>
              <button type="button" size="sm" variant="solid" tone="accent">Accent</button>
              <button type="button" size="sm" variant="ghost" tone="muted">Ghost</button>
            </row>
            <row gap="2" align="center" nowrap specimen-badges>
              <badge tone="muted" size="sm" variant="solid" pill>Secondary</badge>
              <badge tone="primary" size="sm" variant="solid" pill>Primary</badge>
              <badge tone="accent" size="sm" variant="solid" pill>Accent</badge>
            </row>
            <column surface-stack gap="2" label="Surface stack">
              <column gap="2" style(surfaceBg)>
                <p style(pathMeta)>bg</p>
                <column gap="2" style(surface1)>
                  <p style(pathMeta)>surface-1</p>
                  <column gap="2" style(surface2)>
                    <p style(pathMeta)>surface-2</p>
                    <column style(surface3)>
                      <p style(pathMeta)>card</p>
                    </column>
                  </column>
                </column>
              </column>
            </column>
            <column gap="1">
              <p style(fgOnSurface)>Foreground on surface</p>
              <p style(fieldHint)>Muted secondary copy · border + ring below</p>
            </column>
            <column gap="1">
              <p style(fieldHint)>Email</p>
              <input type="email" size="sm" placeholder="you@company.com" label="Sample email" />
            </column>
            <card padding="sm">
              <column gap="2">
                <p style(cardTitle)>Card surface</p>
                <p style(mutedRelaxed)>
                  Focus ring uses color.ring. Borders use color.border.
                </p>
              </column>
            </card>
            <row sem-row gap="2" align="center" nowrap label="Semantic tones">
              <badge tone="success" size="sm" variant="solid" pill>Success</badge>
              <badge tone="warning" size="sm" variant="solid" pill>Warning</badge>
              <badge tone="destructive" size="sm" variant="solid" pill>Danger</badge>
              <badge tone="info" size="sm" variant="solid" pill>Info</badge>
            </row>
          </column>
        </scroll-area>
      </column>
    </card>
  </column>
</column>
