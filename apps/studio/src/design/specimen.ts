import type { DensityId, TypeRoles } from './types'

export type SpecimenOpts = {
  density: DensityId
  typeRoles?: TypeRoles
  /** Optional note under card. */
  note?: string
}

/**
 * Inline .sail specimen for headless SailRender (Themes / design preview).
 * Style MUST attach on the open tag — body-level `style({…})` is invalid IR
 * and fails SailRender with "unknown statement".
 */
export function specimenSource(opts: SpecimenOpts): string {
  const density = opts.density === 'default' ? '' : ` data-density="${opts.density}"`
  const display = opts.typeRoles?.display ?? 'Fraunces'
  const body = opts.typeRoles?.body ?? 'DM Sans'
  const mono = opts.typeRoles?.mono ?? 'JetBrains Mono'
  const note = opts.note ?? 'Sample card'
  return `<!-- design specimen -->
<div${density} style({ pad(6).bg(background).fg(foreground) })>
  <column gap="5">
    <column gap="2">
      <span style({ fg(muted-foreground).type({ size: xs, weight: semibold }) })>Buttons</span>
      <row gap="2" align="center">
        <button type="button" tone="primary">Primary</button>
        <button type="button" variant="outline">Outline</button>
        <button type="button" tone="danger">Danger</button>
      </row>
    </column>
    <column gap="2">
      <span style({ fg(muted-foreground).type({ size: xs, weight: semibold }) })>Card</span>
      <card style({ pad(4).radius(lg).border({ width: 1, color: border-subtle }).bg(surface-1) })>
        <p style({ type({ weight: medium }) })>${escapeText(note)}</p>
        <p style({ pad({ t: 1 }).fg(muted-foreground).type({ size: sm }) })>Density: ${opts.density}</p>
      </card>
    </column>
    <column gap="2">
      <span style({ fg(muted-foreground).type({ size: xs, weight: semibold }) })>Badge · alert · field</span>
      <row gap="2" align="center">
        <badge tone="primary" size="sm">primary</badge>
        <badge tone="success" size="sm">success</badge>
        <badge tone="warning" size="sm">warning</badge>
      </row>
      <alert tone="info">Alert · status copy</alert>
      <input type="search" placeholder="Search…" />
    </column>
    <column gap="2" style({ pad(4).radius(md).bg(surface-0).fg(foreground) })>
      <span style({ fg(muted-foreground).type({ size: xs }) })>Type</span>
      <p style({ type({ size: xl, weight: bold, family: display }) })>Display / ${escapeText(display)}</p>
      <p style({ type({ size: lg, weight: semibold, family: body }) })>Heading / ${escapeText(body)}</p>
      <p style({ type({ size: sm, family: body }) })>Body — quick brown fox</p>
      <p style({ type({ size: xs, family: mono }) })>code / ${escapeText(mono)}</p>
    </column>
  </column>
</div>
`
}

function escapeText(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

export const SPECIMEN_VIEWPORT = { width: 720, height: 640 } as const
