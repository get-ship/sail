// /components/[tag] — the detail view. route.params.tag() picks the entry out
// of the DETAILS registry (the full L1 surface plus the fx actions); one
// variant signal drives the live stage (each entry maps its chips onto
// whichever attribute its variants mean — button:variant, slider:tone,
// switch:size, tooltip:placement).
//
// Every read the template does is a zero-arg call (`entry()`, `stage.button()`),
// because that is the only call shape a renderable expression allows — the
// stage map mirrors the browser page's `show` map.

import { controller, signal, derived, effect, type ViewScope } from '@ship/sui-runtime'
import { spotlight, dotGrid } from '@ship/sui-fx'
import type { Scope } from './page.sui'
import { DETAILS, type DetailEntry } from './registry'

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const tag = derived(() => ctx.route.params.tag?.() ?? '')
    const entry = derived<DetailEntry | null>(() => DETAILS[tag()] ?? null)
    const known = derived(() => entry() !== null)

    const variant = signal('')
    // A navigation between two detail pages reuses this scope — re-seed the
    // variant whenever the tag changes.
    effect(() => {
      variant.set(entry()?.variants[0] ?? '')
    })
    const pick = (v: string): void => variant.set(v)

    const variants = () => entry()?.variants ?? []
    const hasVariants = derived(() => (entry()?.variants.length ?? 0) > 0)
    const variantAttr = () => entry()?.variantAttr ?? ''
    const blurb = () => entry()?.blurb ?? ''
    const props = () => entry()?.props ?? []
    const usageCode = () => entry()?.usage ?? ''
    const heading = () => entry()?.heading ?? tag()
    const isFx = derived(() => entry()?.pack === 'fx')
    const packLabel = () => (entry()?.pack === 'fx' ? '@ship/sui-fx' : 'In the language')

    // One zero-arg gate per stage — `stage.<key>()`, derived from the entry.
    const stage = Object.fromEntries(
      Object.values(DETAILS).map((e) => [e.key, () => entry()?.key === e.key]),
    ) as Record<string, () => boolean>

    // Stage state — real signals the stages mutate.
    const presses = signal(0)
    const press = (): void => presses.set((n) => n + 1)
    const notify = signal(true)
    const agreed = signal(false)
    const load = signal(62)
    const name = signal('')
    const bio = signal('')
    const team = signal('Design')
    const fw = signal('')
    const pickFw = (e: CustomEvent<{ value: string }>): void => fw.set(e.detail?.value ?? '')
    const pageNo = signal('2')
    const goPage = (e: CustomEvent<{ value: string }>): void => pageNo.set(e.detail?.value ?? '1')
    const kept = signal(true)
    const dropChip = (): void => kept.set(false)
    const restoreChip = (): void => kept.set(true)
    const sheetSide = () => (variant() === 'sheet' ? 'end' : null)
    const people = () => [
      { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'active' },
      { id: 3, name: 'Alan Turing', team: 'Research', status: 'away' },
    ]

    return {
      tag, entry, known, heading, isFx, packLabel,
      variant, pick, variants, hasVariants, variantAttr,
      blurb, props, usageCode, stage,
      presses, press, notify, agreed, load, name, bio, team,
      fw, pickFw, pageNo, goPage, kept, dropChip, restoreChip,
      sheetSide, people,
      spotlight, dotGrid,
    }
  },
})
