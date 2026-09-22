// sail-media engine — the reactive source→descriptor slice. The ONLY engine
// file that imports `@ship/sail-runtime` (signal/derived). No I/O, no DOM, no
// `ship.*`/`fetch`/`query()`.
//
// Authority: build-spec §"API sketch" (the `MimeRegistry` facade), §"Backbone
// #1 — the headless-engine-behind-a-thin-shell pattern".

import { derived, signal, type Derived, type Signal } from '@ship/sail-runtime'
import type { RegistryEntry, SourceDescriptor, ViewerKind, ViewerSource } from './types'
import { resolveSource } from './resolve'

export interface DescriptorSlice {
  source: Signal<ViewerSource | null>
  entries: Signal<readonly RegistryEntry[]>
  descriptor: Derived<SourceDescriptor>
  kind: Derived<ViewerKind>
  renderer: Derived<string | null>
  isSupported: Derived<boolean>
}

/** Wire the reactive source/entries signals to a memoized descriptor. The
 *  descriptor recomputes only when `source` or `entries` changes — between
 *  reads it returns the same cached object (memoized `derived`). */
export function createDescriptorSlice(
  initialEntries: RegistryEntry[],
  fallbackRenderer: string | null,
): DescriptorSlice {
  const source = signal<ViewerSource | null>(null)
  const entries = signal<readonly RegistryEntry[]>(initialEntries)

  const descriptor = derived<SourceDescriptor>(() =>
    resolveSource(entries(), source(), fallbackRenderer),
  )
  const kind = derived<ViewerKind>(() => descriptor().kind)
  const renderer = derived<string | null>(() => descriptor().renderer)
  const isSupported = derived<boolean>(() => descriptor().kind !== 'unsupported')

  return { source, entries, descriptor, kind, renderer, isSupported }
}
