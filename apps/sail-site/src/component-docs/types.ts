/** Docs package meta — SSOT for /components hub tiles. */
export type DocKind = 'host' | 'alias' | 'action' | 'keywords'

export type DocCategory =
  | 'inputs'
  | 'display'
  | 'feedback'
  | 'overlay'
  | 'navigation'
  | 'data'
  | 'layout'
  | 'effects'

export interface DocMeta {
  /** Route slug + hub id: /components/<slug> */
  slug: string
  /** Element tag when kind=host; may match slug. */
  tag?: string
  /** Scope key used by legacy stages / show.<key>(). */
  key: string
  category: DocCategory
  kind: DocKind
  blurb: string
  /** Override h1 (e.g. layout keywords). */
  heading?: string
  /**
   * Hyphenated CE tag for the docs host, or 'legacy' to use doc-legacy-detail.
   * First-class packages set host to doc-<slug>.
   */
  host: string
  /** Stage owns configurator knobs (hide viewer variant strip). */
  stageOwnsKnobs?: boolean
}
