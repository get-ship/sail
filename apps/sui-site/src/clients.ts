// Clients seam for the SUI site — the LIVE component viewer.
//
// Registers the viewing pack (@ship/sui-viewer) + the primitive pack
// (@ship/sui-primitives) so every <sui-*> is defined, then sets up the
// in-browser compile seam: ensureWasm() instantiates @ship/sui-compiler-wasm
// from the host-served URL, makeCompile() binds the pure compile() the viewer
// drives via prop:compile. The gallery store feeds the selected primitive's
// .sui source + CEM attributes into <sui-component-viewer>.
//
// This is the payoff of the @ship/sdk-client decoupling: the whole chain
// (viewer → primitives) is now SDK-free, so the live preview resolves with no
// ship-daemon coupling.
import './styles.css'
import { signal } from '@ship/sui-runtime'
import '@ship/sui-viewer' // side-effect: defines sui-component-viewer + children
import '@ship/sui-primitives' // side-effect: defines the previewed <sui-*> tags
import { ensureWasm, makeCompile, type CompileFn } from '@ship/sui-viewer'
import manifest from './gallery-manifest.json'

/** The viewer's `cem` prop wants the flattened CEM `attributes[]`; the manifest
 *  stores the full CEM module. Pull the custom-element declaration's attrs. */
function cemAttrs(cem: unknown): unknown[] | null {
  const mods = (cem as { modules?: unknown[] } | null)?.modules ?? []
  for (const m of mods as Array<{ declarations?: unknown[] }>) {
    for (const d of (m.declarations ?? []) as Array<{
      customElement?: boolean
      attributes?: unknown[]
    }>) {
      if (d.customElement && Array.isArray(d.attributes)) return d.attributes
    }
  }
  return null
}

export interface GalleryEntry {
  tag: string
  title: string
  source: string
  cem: unknown
  cemAttrs: unknown[] | null
}

const entries: GalleryEntry[] = (manifest as unknown as Omit<GalleryEntry, 'cemAttrs'>[]).map(
  (e) => ({ ...e, cemAttrs: cemAttrs(e.cem) }),
)
const selected = signal<string>(entries[0]?.tag ?? '')

// The bound compile() resolves asynchronously once the wasm instantiates. Until
// then it stays null and <sui-render> shows its idle stage — never an error.
const compile = signal<CompileFn | null>(null)
ensureWasm('/sui_compiler_bg.wasm')
  .then(() => compile.set(() => makeCompile()))
  .catch((err) => {
    // Fail loud: a broken compile seam must not look like an empty gallery.
    console.error('[sui-site] wasm compile seam failed to initialise:', err)
  })

export const gallery = {
  entries,
  selected,
  compile,
  select(tag: string): void {
    selected.set(() => tag)
  },
  entryFor(tag: string): GalleryEntry | undefined {
    return entries.find((e) => e.tag === tag)
  },
}
