/**
 * Agent brief text for Copy agent brief — plain structured fields for agents.
 */
import type { ComponentRecord } from './types'

export type BriefInput = {
  record: ComponentRecord
  /** Optional composition child tags when known. */
  composition?: string[]
  issuesSummary?: string
}

/** Build clipboard text for a library (or instance-mapped) component. */
export function buildAgentBrief(input: BriefInput): string {
  const { record } = input
  const props =
    record.props.length > 0
      ? record.props
          .map((p) => `${p.name}${p.required ? '!' : ''}`)
          .join(', ')
      : '—'
  const issues =
    record.issueCount > 0
      ? `${record.issueCount}${input.issuesSummary ? ` (${input.issuesSummary})` : ''}`
      : '0'
  const composition =
    input.composition && input.composition.length
      ? input.composition.join(', ')
      : '—'
  const intent = record.description?.trim() || 'none'
  const pack = record.package && record.package !== '—' ? record.package : '—'
  const path = record.path?.trim() || '—'
  return [
    `Component: ${record.id} (${record.origin})`,
    `Path: ${path}`,
    `Pack: ${pack}`,
    `Props: ${props}`,
    `Issues: ${issues}`,
    `SSR: ${record.ssrTierSummary || 'unknown'}`,
    `Composition: ${composition}`,
    `Intent: ${intent}`,
  ].join('\n')
}

/** Brief for New… scaffold (not yet a real component). */
export function buildNewComponentBrief(opts: {
  name: string
  intent: string
  layer: string
}): string {
  const id = opts.name.trim() || 'unnamed'
  return [
    `Component: ${id} (project)`,
    `Path: —`,
    `Pack: —`,
    `Props: —`,
    `Issues: 0`,
    `SSR: unknown`,
    `Composition: —`,
    `Intent: ${opts.intent.trim() || 'none'}`,
    `Layer: ${opts.layer.trim() || 'app'}`,
    `Status: draft brief (not scaffolded on disk)`,
  ].join('\n')
}
