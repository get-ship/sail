<!--
  Color facet specimen — draft paint via theme-session host bind.
-->
<scope>
  style fieldHint = { paint: fg(muted-foreground), text: fontSize(xs) }
  style pathMeta = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed).fontFamily(body) }
  style mutedRelaxed = { paint: fg(muted-foreground), text: fontSize(xs).leading(relaxed) }
  style specimenDock = { box: grow(1).minW(0).maxW(panel-max-w-md).basis(0).minH(0).h(full).overflow(hidden), paint: borderLWidth(1).borderL(border-subtle).bg(background) }
  style specimenPad = { box: grow(1).minH(0).h(full).pad(4) }
  style specimenFrame = { box: minW(0).minH(0).h(full).overflow(hidden), paint: shadow(lg) }
  style fillCol = { box: grow(1).minH(0).h(full) }
  style specimenChrome = { box: shrink(0).padX(3).h(8).minH(8).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style trafficDots = { box: shrink(0).minW(12) }
  style trafficDotDestructive = { box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(destructive) }
  style trafficDotWarning = { box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(warning) }
  style trafficDotSuccess = { box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(success) }
  style chromeTitle = { box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).truncate() }
  style modeMeta = { box: shrink(0), paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style scrollGrow = { box: grow(1).minH(0).basis(0).h(full), paint: bg(background) }
  style bodyPad = { box: pad(5).minW(0) }
  style productTitle = { text: fontSize(md).fontWeight(semibold).fontFamily(heading) }
  style fgOnSurface = { paint: fg(foreground), text: fontSize(sm) }
  style cardTitle = { text: fontSize(sm).fontWeight(semibold) }
  style surfaceBg = { box: pad(3), paint: radius(md).borderWidth(1).border(border-subtle).bg(background) }
  style surface1 = { box: pad(3), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-1) }
  style surface2 = { box: pad(3), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-2) }
  style surface3 = { box: pad(3), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-3) }
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
