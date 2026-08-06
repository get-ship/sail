<!--
  Color facet specimen — draft paint via theme-draft-paint adapter.
  session={session} identity bind. Host registered for applyScope re-apply.
-->
<controller>
import { signal } from '@ship/sail-runtime'
import type { ThemeSession } from '../theme-session/theme-session'
import { watchPreviewHost } from '../theme-session/bind-preview-host'

export default controller(() => {
  const sessionSig = signal<ThemeSession | null>(null)
  mount((host) => {
    const pre = (host as unknown as { session?: ThemeSession | null }).session
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

<scope>
  const fieldHint = style({ paint: fg(muted-foreground), text: fontSize(xs) })
  const pathMeta = style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })
</scope>

<column specimen-dock gap="0">
  style({  box: grow(1).minW(0).minW(panel-min-w-md).maxW(panel-max-w-md).basis(0).minH(0).h(full).overflow(hidden), paint: borderLWidth(1).borderL(border-subtle).bg(background)  })

  <column gap="0">
    style({  box: grow(1).minH(0).h(full).pad(4)  })

    <!-- Draft paint = applyScope via theme-session bind (not theme). -->
    <card
      color-specimen
      preview-specimen
      mode={paintMode()}
      padding="none"

    >
      style({ box: minW(0).minH(0).h(full).overflow(hidden), paint: shadow(lg) })

      <column gap="0">
        style({  box: grow(1).minH(0).h(full)  })

        <row specimen-chrome align="center" gap="2" nowrap>
          style({  box: shrink(0).padX(3).h(8).minH(8).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1)  })

          <row gap="2" align="center" nowrap hidden="true">
            style({  box: shrink(0).minW(12)  })

            <stack>
              style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(destructive)  })
</stack>
            <stack>
              style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(warning)  })
</stack>
            <stack>
              style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(success)  })
</stack>
          </row>
          <p>
            style({ box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).truncate() })
{nameEdit()}</p>
          <p>
            style([ style(pathMeta), style({ box: shrink(0) }) ])
{paintMode()}</p>
        </row>
        <scroll-area>
          style({  box: grow(1).minH(0).basis(0).h(full), paint: bg(background)  })

          <column specimen-body gap="4">
            style({ box: pad(5).minW(0) })

            <column gap="1">
              <p>
                style({ text: fontSize(md).fontWeight(semibold).fontFamily(heading) })
Product UI</p>
              <p>
                style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed).fontFamily(body) })

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
              <column gap="2">
                style({ box: pad(3), paint: radius(md).borderWidth(1).border(border-subtle).bg(background) })

                <p>
                  style(pathMeta)
bg</p>
                <column gap="2">
                  style({ box: pad(3), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-1) })

                  <p>
                    style(pathMeta)
surface-1</p>
                  <column gap="2">
                    style({ box: pad(3), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-2) })

                    <p>
                      style(pathMeta)
surface-2</p>
                    <column>
                      style({ box: pad(3), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-3) })

                      <p>
                        style(pathMeta)
card</p>
                    </column>
                  </column>
                </column>
              </column>
            </column>
            <column gap="1">
              <p>
                style({ paint: fg(foreground), text: fontSize(sm) })
Foreground on surface</p>
              <p>
                style({ paint: fg(muted-foreground), text: fontSize(xs) })
Muted secondary copy · border + ring below</p>
            </column>
            <column gap="1">
              <p>
                style({ paint: fg(muted-foreground), text: fontSize(xs) })
Email</p>
              <input type="email" size="sm" placeholder="you@company.com" label="Sample email" />
            </column>
            <card padding="sm">
              <column gap="2">
                <p>
                  style({ text: fontSize(sm).fontWeight(semibold) })
Card surface</p>
                <p>
                  style({ paint: fg(muted-foreground), text: fontSize(xs).leading(relaxed) })

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
