<!--
  Color facet specimen — draft paint via theme-session host bind.
-->
<scope>
  style fieldHint = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style pathMeta = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ family: body, size: sm, leading: relaxed }) }
  style mutedRelaxed = { paint: ink({ color: muted-foreground }), text: type({ size: xs, leading: relaxed }) }
  style specimenDock = { box: flow({ grow: 1 }).size({ h: full, min-w: 0, min-h: 0, max-w: panel-max-w-md }).basis(0).overflow({ all: hidden }), paint: border({ edge: l, color: border-subtle, width: 1 }).fill({ color: background }) }
  style specimenPad = { box: flow({ grow: 1 }).size({ h: full, min-h: 0 }).space({ pad: 4 }) }
  style specimenFrame = { box: size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }), paint: elevation(lg) }
  style fillCol = { box: flow({ grow: 1 }).size({ h: full, min-h: 0 }) }
  style specimenChrome = { box: flow({ shrink: 0 }).space({ pad-x: 3 }).size({ h: 8, min-w: 0, min-h: 8 }), paint: border({ edge: b, color: border-subtle, width: 1 }).fill({ color: surface-1 }) }
  style trafficDots = { box: flow({ shrink: 0 }).size({ min-w: 12 }) }
  style trafficDotDestructive = { box: flow({ shrink: 0 }).size({ w: 2, h: 2, min-w: 2, min-h: 2 }), paint: radius(full).fill({ color: destructive }) }
  style trafficDotWarning = { box: flow({ shrink: 0 }).size({ w: 2, h: 2, min-w: 2, min-h: 2 }), paint: radius(full).fill({ color: warning }) }
  style trafficDotSuccess = { box: flow({ shrink: 0 }).size({ w: 2, h: 2, min-w: 2, min-h: 2 }), paint: radius(full).fill({ color: success }) }
  style chromeTitle = { box: flow({ grow: 1 }).size({ min-w: 0 }), paint: ink({ color: muted-foreground }), text: type({ size: xs, wrap: truncate }) }
  style modeMeta = { box: flow({ shrink: 0 }), paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style scrollGrow = { box: flow({ grow: 1 }).size({ h: full, min-h: 0 }).basis(0), paint: fill({ color: background }) }
  style bodyPad = { box: space({ pad: 5 }).size({ min-w: 0 }) }
  style productTitle = { text: type({ family: heading, size: md, weight: semibold }) }
  style fgOnSurface = { paint: ink({ color: foreground }), text: type({ size: sm }) }
  style cardTitle = { text: type({ size: sm, weight: semibold }) }
  style surfaceBg = { box: space({ pad: 3 }), paint: radius(md).border({ color: border-subtle, width: 1 }).fill({ color: background }) }
  style surface1 = { box: space({ pad: 3 }), paint: radius(md).border({ color: border-subtle, width: 1 }).fill({ color: surface-1 }) }
  style surface2 = { box: space({ pad: 3 }), paint: radius(md).border({ color: border-subtle, width: 1 }).fill({ color: surface-2 }) }
  style surface3 = { box: space({ pad: 3 }), paint: radius(md).border({ color: border-subtle, width: 1 }).fill({ color: surface-3 }) }
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
