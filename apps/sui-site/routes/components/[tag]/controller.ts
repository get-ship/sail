// /components/[tag] — the detail view. route.params.tag() picks the entry out
// of the hand-curated DETAILS registry; one variant signal drives the live
// stage (each flagship maps the chips onto whichever attribute its variants
// mean — button:variant, slider:tone, switch:size, dialog:side).
//
// Every read the template does is a zero-arg call (`entry()`, `isButton()`),
// because that is the only call shape a renderable expression allows.

import { controller, signal, derived, effect, type ViewScope } from '@ship/sui-runtime'
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

    // One zero-arg gate per flagship stage.
    const isButton = derived(() => tag() === 'button')
    const isCombobox = derived(() => tag() === 'combobox')
    const isTabs = derived(() => tag() === 'tabs')
    const isDialog = derived(() => tag() === 'dialog')
    const isTable = derived(() => tag() === 'table')
    const isSwitch = derived(() => tag() === 'switch')
    const isSlider = derived(() => tag() === 'slider')
    const isBadge = derived(() => tag() === 'badge')
    const isCard = derived(() => tag() === 'card')
    const isAlert = derived(() => tag() === 'alert')

    // Stage state — real signals the stage mutates.
    const presses = signal(0)
    const press = (): void => presses.set((n) => n + 1)
    const notify = signal(true)
    const load = signal(62)
    const fw = signal('')
    const pickFw = (e: CustomEvent<{ value: string }>): void => fw.set(e.detail?.value ?? '')
    const sheetSide = () => (variant() === 'sheet' ? 'end' : null)
    const people = () => [
      { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'active' },
      { id: 3, name: 'Alan Turing', team: 'Research', status: 'away' },
    ]

    return {
      tag, entry, known,
      variant, pick, variants, hasVariants, variantAttr,
      blurb, props, usageCode,
      isButton, isCombobox, isTabs, isDialog, isTable,
      isSwitch, isSlider, isBadge, isCard, isAlert,
      presses, press, notify, load, fw, pickFw, sheetSide, people,
    }
  },
})
