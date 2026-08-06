
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
<div${density}>
  style({ box: pad(6), paint: bg(background).fg(foreground) })
  <column gap="5">
    <column gap="2">
      <span>
        style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold) })
        Buttons</span>
      <row gap="2" align="center">
        <button type="button" tone="primary">Primary</button>
        <button type="button" variant="outline">Outline</button>
        <button type="button" tone="destructive">Destructive</button>
      </row>
    </column>
    <column gap="2">
      <span>
        style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold) })
        Card</span>
      <card>
        style({ box: pad(4), paint: radius(lg).borderWidth(1).border(border-subtle).bg(surface-1) })
        <p>
          style({ text: fontWeight(medium) })
          ${escapeText(note)}</p>
        <p>
          style({ box: padT(1), paint: fg(muted-foreground), text: fontSize(sm) })
          Density: ${opts.density}</p>
      </card>
    </column>
    <column gap="2">
      <span>
        style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold) })
        Badge · alert · field</span>
      <row gap="2" align="center">
        <badge tone="primary" size="sm">primary</badge>
        <badge tone="success" size="sm">success</badge>
        <badge tone="warning" size="sm">warning</badge>
      </row>
      <alert tone="info">Alert · status copy</alert>
      <input type="search" placeholder="Search…" />
    </column>
    <column gap="2">
      style({ box: pad(4), paint: radius(md).bg(surface-0).fg(foreground) })
      <span>
        style({ paint: fg(muted-foreground), text: fontSize(xs) })
        Type</span>
      <p>
        style({ text: fontSize(xl).fontWeight(bold).fontFamily(display) })
        Display / ${escapeText(display)}</p>
      <p>
        style({ text: fontSize(lg).fontWeight(semibold).fontFamily(body) })
        Heading / ${escapeText(body)}</p>
      <p>
        style({ text: fontSize(sm).fontFamily(body) })
        Body — quick brown fox</p>
      <p>
        style({ text: fontSize(xs).fontFamily(mono) })
        code / ${escapeText(mono)}</p>
    </column>
  </column>
</div>
`
}

function escapeText(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

export const SPECIMEN_VIEWPORT = { width: 720, height: 640 } as const
