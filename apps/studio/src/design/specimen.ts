
import type { DensityId, TypeRoles } from './types'

export type SpecimenOpts = {
  density: DensityId
  typeRoles?: TypeRoles
  /** Optional note under card. */
  note?: string
}

/** Inline .sail specimen for headless SailRender audit (secondary path). */
export function specimenSource(opts: SpecimenOpts): string {
  const density = opts.density === 'default' ? '' : ` data-density="${opts.density}"`
  const display = opts.typeRoles?.display ?? 'Fraunces'
  const body = opts.typeRoles?.body ?? 'DM Sans'
  const mono = opts.typeRoles?.mono ?? 'JetBrains Mono'
  const note = opts.note ?? 'Sample card'
  return `<!-- design specimen -->
<div${density} sx={style.pad(6).bg(background).fg(foreground)}>
  <column gap="5">
    <column gap="2">
      <span sx={style.fontSize(xs).fg(muted-foreground).fontWeight(semibold)}>Buttons</span>
      <row gap="2" align="center">
        <button type="button" tone="primary">Primary</button>
        <button type="button" variant="outline">Outline</button>
        <button type="button" tone="destructive">Destructive</button>
      </row>
    </column>
    <column gap="2">
      <span sx={style.fontSize(xs).fg(muted-foreground).fontWeight(semibold)}>Card</span>
      <card sx={style.pad(4).radius(lg).borderWidth(1).border(border-subtle).bg(surface-1)}>
        <p sx={style.fontWeight(medium)}>${escapeText(note)}</p>
        <p sx={style.fontSize(sm).fg(muted-foreground).padT(1)}>Density: ${opts.density}</p>
      </card>
    </column>
    <column gap="2">
      <span sx={style.fontSize(xs).fg(muted-foreground).fontWeight(semibold)}>Badge · alert · field</span>
      <row gap="2" align="center">
        <badge tone="primary" size="sm">primary</badge>
        <badge tone="success" size="sm">success</badge>
        <badge tone="warning" size="sm">warning</badge>
      </row>
      <alert tone="info">Alert · status copy</alert>
      <input type="search" placeholder="Search…" />
    </column>
    <column gap="2" sx={style.pad(4).radius(md).bg(surface-0).fg(foreground)}>
      <span sx={style.fontSize(xs).fg(muted-foreground)}>Type</span>
      <p sx={style.fontSize(xl).fontWeight(bold).fontFamily(display)}>Display / ${escapeText(display)}</p>
      <p sx={style.fontSize(lg).fontWeight(semibold).fontFamily(body)}>Heading / ${escapeText(body)}</p>
      <p sx={style.fontSize(sm).fontFamily(body)}>Body — quick brown fox</p>
      <p sx={style.fontSize(xs).fontFamily(mono)}>code / ${escapeText(mono)}</p>
    </column>
  </column>
</div>
`
}

function escapeText(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

export const SPECIMEN_VIEWPORT = { width: 720, height: 640 } as const
