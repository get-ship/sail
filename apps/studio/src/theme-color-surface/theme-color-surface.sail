<!--
  Theme color facet — thin shell: chrome + roles editor + specimen.
  Controller composition: session + color domain.
-->
<scope>
  style sectionTitle = { text: type({ size: sm, weight: semibold }) }
  style fieldHint = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style monoMeta = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style pathMeta = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style doorTitle = { text: type({ weight: semibold }) }
  style editorCol = { box: space({ pad-x: 6, pad-t: 6, pad-b: 10 }).size({ max-w: panel-max-w-lg }) }
  style brandCard = { box: overflow({ all: hidden }).size({ min-w: 0 }), paint: radius(lg).border({ color: border-subtle, width: 1 }).fill({ color: surface-1 }) }
  // position:relative sized box — sail-color-picker host is display:contents; swatch fills this box.
  style brandSwatch = { box: place({ position: relative }).size({ w: full, h: 28, min-h: 28 }).overflow({ all: hidden }), when: { stateEdit: { paint: cursor(pointer) }, stateLocked: { paint: cursor(default) } } }
  // Token-row chip: explicit size + relative so absolute-fill swatch has a real hit target.
  style tokenChip = { box: place({ position: relative }).size({ w: 9, h: 9, min-w: 9, min-h: 9 }).overflow({ all: hidden }), paint: radius(md).border({ color: border-subtle, width: 1 }) }
  style heroMeta = { box: space({ pad-x: 3, pad-y: 2 }) }
  style tokenRow = { box: space({ pad-y: 2 }).size({ min-w: 0 }) }
  style seedDot = { box: size({ w: 4, h: 4, min-w: 4, min-h: 4 }), paint: radius(full).border({ color: border-subtle, width: 1 }).fill({ color: bind(swatchColor) }) }
  style surfaceChip = { box: space({ pad-x: 2, pad-y: 1 }), paint: radius(md).border({ color: border-subtle, width: 1 }).fill({ color: surface-0 }) }
  style surfaceDot = { box: size({ w: 3, h: 3, min-w: 3, min-h: 3 }), paint: radius(full).border({ color: border-subtle, width: 1 }).fill({ color: bind(swatchColor) }) }
  style bandStatus = { box: space({ pad-x: 4, pad-y: 2 }) }
  style bandError = { box: space({ pad-x: 4, pad-y: 2 }), paint: border({ edge: b, color: border, width: 1 }) }
  style surfaceRoot = { box: flow({ grow: 1 }).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }), paint: fill({ color: background }) }
  style workbench = { box: flow({ grow: 1 }).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }), paint: fill({ color: background }) }
  style editorScroll = { box: flow({ grow: 1 }).size({h: full, min-w: 0, min-h: 0, basis: 0}), paint: fill({ color: background }) }
  style pageTitle = { text: type({ size: xl, weight: semibold, tracking: tight }) }
  style seedStrip = { box: space({ pad: 3 }).size({ min-w: 0 }), paint: radius(lg).border({ color: border-subtle, width: 1 }).fill({ color: surface-0 }) }
  style growMin = { box: flow({ grow: 1 }).size({ min-w: 0 }) }
  style errorText = { box: flow({ grow: 1 }).size({ min-w: 0 }), paint: ink({ color: destructive }), text: type({ size: sm, wrap: truncate }) }
  style pickerFill = { box: size({ w: full, h: full, min-h: full }) }
  style footerRule = { box: space({ pad-t: 2 }), paint: border({ edge: t, color: border-subtle, width: 1 }) }
  type ColorTokenRowProps = {
    token: string
    value: string
    label: string
    path: string
    disabled: boolean
    onChange: any
  }
  type BrandRoleCardProps = {
    token: string
    value: string
    label: string
    path: string
    disabled: boolean
    onChange: any
  }
</scope>

<partial color-token-row props: ColorTokenRowProps>
  <row token-row token={token} sem={token} align="center" gap="3" nowrap style(tokenRow)>
    <stack token-chip style(tokenChip, { box: flow({ shrink: 0 }) })>
      <sail-color-picker
        token={token}
        value={value}
        disabled={disabled}
        variant="swatch"
        showHex={false}
        label={label}
        on-change={onChange}
      ></sail-color-picker>
    </stack>
    <column gap="0" style(growMin)>
      <p style(doorTitle)>{label}</p>
      <p style(pathMeta)>{path}</p>
    </column>
    <p style(monoMeta, { box: flow({ shrink: 0 }) })>{value}</p>
  </row>
</partial>

<partial brand-role-card props: BrandRoleCardProps>
  <column brand-card token={token} gap="0" style(brandCard, { box: flow({ grow: 1 }).size({ min-w: 0 }) })>
    <stack token={token} style(brandSwatch)>
      <sail-color-picker
        token={token}
        value={value}
        disabled={disabled}
        variant="swatch"
        showHex={false}
        label={label}
        on-change={onChange}
        style(pickerFill)
      ></sail-color-picker>
    </stack>
    <row align="center" justify="between" style(heroMeta)>
      <column gap="0" style({ box: size({ min-w: 0 }) })>
        <p style(doorTitle)>{label}</p>
        <p style(pathMeta)>{path}</p>
      </column>
      <p style(monoMeta)>{value}</p>
    </row>
  </column>
</partial>

<controller>
import { createThemeSession } from '../compositions/theme-session/theme-session'
import { createColorEdit } from '../compositions/theme-color-surface/theme-color-edit'
export default controller(() => {
  const session = createThemeSession({
    facet: 'color',
    editGuardMessage: 'Add this theme to the project to edit color.',
    saveSuccessMessage: 'Color roles saved to DTCG source.',
  })
  const color = createColorEdit(session)
  return {
    session,
    color,
    primaryHex: color.primaryHex,
    secondaryHex: color.secondaryHex,
    accentHex: color.accentHex,
    bgHex: color.bgHex,
    surface1Hex: color.surface1Hex,
    surface2Hex: color.surface2Hex,
    cardHex: color.cardHex,
    fgHex: color.fgHex,
    mutedHex: color.mutedHex,
    borderHex: color.borderHex,
    ringHex: color.ringHex,
    successHex: color.successHex,
    warningHex: color.warningHex,
    dangerHex: color.dangerHex,
    infoHex: color.infoHex,
  }
})
</controller>

<column surface="themes" theme-color-surface color-chrome gap="0" style(surfaceRoot)>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row color-error align="center" gap="2" style(bandError, { box: flow({ shrink: 0 }) })>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" color-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row color-loading align="center" gap="2" style(bandStatus, { box: flow({ shrink: 0 }) })>
      <spinner size="sm"></spinner>
      <p style(fieldHint)>Loading…</p>
    </row>
  </if>

  <row color-workbench gap="0" nowrap align="stretch" style(workbench)>
    <scroll-area color-editor style(editorScroll)>
      <column gap="8" style(editorCol)>
        <column color-header gap="2">
          <p style(pageTitle)>Color</p>
          <p style(fieldHint)>Brand · surfaces · text · semantic</p>
        </column>

        <row seed-strip align="center" gap="3" nowrap style(seedStrip)>
          <p style(fieldHint, { box: flow({ shrink: 0 }) })>Seed</p>
          <row gap="1" align="center" nowrap hidden="true" style(growMin)>
            <stack use:swatch={primaryHex()} style(seedDot)></stack>
            <stack use:swatch={secondaryHex()} style(seedDot)></stack>
            <stack use:swatch={accentHex()} style(seedDot)></stack>
            <stack use:swatch={bgHex()} style(seedDot)></stack>
            <stack use:swatch={fgHex()} style(seedDot)></stack>
            <stack use:swatch={mutedHex()} style(seedDot)></stack>
            <stack use:swatch={borderHex()} style(seedDot)></stack>
          </row>
          <button type="button" size="sm" variant="outline" tone="muted" seed-generate disabled on:click={color.onSeedGenerate} title="Coming soon">
            Generate…
          </button>
        </row>

        <column section="brand" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p style(sectionTitle)>Brand</p>
            <p style(pathMeta)>color.primary · secondary · accent</p>
          </row>
          <row brand-grid gap="3" align="stretch">
            <brand-role-card
              token="primary"
              value={primaryHex()}
              label="Primary"
              path="color.primary"
              disabled={!session.canEdit()}
              onChange={color.onRoleColor}
            ></brand-role-card>
            <brand-role-card
              token="secondary"
              value={secondaryHex()}
              label="Secondary"
              path="color.secondary"
              disabled={!session.canEdit()}
              onChange={color.onRoleColor}
            ></brand-role-card>
            <brand-role-card
              token="accent"
              value={accentHex()}
              label="Accent"
              path="color.accent"
              disabled={!session.canEdit()}
              onChange={color.onRoleColor}
            ></brand-role-card>
          </row>
          <if {session.catalogHint()}>
            <p catalog-hint style(fieldHint)>Add to project to edit.</p>
          </if>
        </column>

        <!-- 2. Surfaces -->
        <column section="surfaces" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p style(sectionTitle)>Surfaces</p>
            <p style(pathMeta)>color.bg · surface · card</p>
          </row>
          <column token-list gap="1">
            <color-token-row
              token="bg"
              value={bgHex()}
              label="Background"
              path="color.bg"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="surface1"
              value={surface1Hex()}
              label="Surface 1"
              path="color.surface.1"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="surface2"
              value={surface2Hex()}
              label="Surface 2"
              path="color.surface.2"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="card"
              value={cardHex()}
              label="Card"
              path="color.surface.3"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
          </column>
          <row surface-chips gap="2" align="center" nowrap hidden="true">
            <row align="center" gap="1" style(surfaceChip)>
              <stack use:swatch={bgHex()} style(surfaceDot)></stack>
              <p style(pathMeta)>bg</p>
            </row>
            <row align="center" gap="1" style(surfaceChip)>
              <stack use:swatch={surface1Hex()} style(surfaceDot)></stack>
              <p style(pathMeta)>surface-1</p>
            </row>
            <row align="center" gap="1" style(surfaceChip)>
              <stack use:swatch={surface2Hex()} style(surfaceDot)></stack>
              <p style(pathMeta)>surface-2</p>
            </row>
            <row align="center" gap="1" style(surfaceChip)>
              <stack use:swatch={cardHex()} style(surfaceDot)></stack>
              <p style(pathMeta)>card</p>
            </row>
          </row>
        </column>

        <!-- 3. Text & borders -->
        <column section="text" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p style(sectionTitle)>Text &amp; borders</p>
            <p style(pathMeta)>color.fg · muted · border · ring</p>
          </row>
          <column token-list gap="1">
            <color-token-row
              token="fg"
              value={fgHex()}
              label="Foreground"
              path="color.foreground"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="muted"
              value={mutedHex()}
              label="Muted"
              path="color.muted"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="border"
              value={borderHex()}
              label="Border"
              path="color.border"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="ring"
              value={ringHex()}
              label="Ring"
              path="color.ring"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
          </column>
        </column>

        <!-- 4. Semantic -->
        <column section="semantic" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p style(sectionTitle)>Semantic</p>
            <p style(pathMeta)>color.success · warning · danger · info</p>
          </row>
          <column token-list gap="1">
            <color-token-row
              token="success"
              value={successHex()}
              label="Success"
              path="color.success"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="warning"
              value={warningHex()}
              label="Warning"
              path="color.warning"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="danger"
              value={dangerHex()}
              label="Danger"
              path="color.danger"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="info"
              value={infoHex()}
              label="Info"
              path="color.info"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
          </column>
        </column>

        <column color-footer gap="2" style(footerRule)>
          <p style(fieldHint)>Roles write DTCG color paths for this theme mode.</p>
        </column>
      </column>
    </scroll-area>

    <theme-color-specimen session={session}></theme-color-specimen>
  </row>

  <toast-region position="bottom-end" color-toast-region>
    <for {session.toastItems() as t} key={t.key}>
      <toast tone={t.tone} color-toast on-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
