import type { Action } from '../action'

/**
 * `use:virtualRow={start}` — the runtime geometry of a virtualized row.
 *
 * `start` is the row's offset in px (a virtualizer measurement, NOT a token
 * — there is no token for "row N of a 50000-row list"). The action writes the
 * number to a custom property; sx reads it as a translate:
 *
 *     <grid use:virtualRow={w.start} sx="translate-y-(--row-y) absolute">…</grid>
 *
 * The atom `translate-y-(--row-y)` lowers to
 *     `.sx-translate-y-rv-rowy { translate: 0 var(--row-y) }`
 *
 * The action lives in the framework layer (`packages/sail/runtime/src/actions`)
 * because the only place a `.style` write is legitimate is the seam itself —
 * a primitive that turns an integer into a custom property and stops there.
 *
 * `use:virtualRow` is the primitive; `use:style={{...}}` would have been the
 * hatch. That distinction is the point.
 */
export const virtualRow: Action<number | null | undefined> = (node, initial) => {
  const el = node as HTMLElement
  const apply = (start: number | null | undefined): void => {
    if (start == null || Number.isNaN(start)) {
      el.style.removeProperty('--row-y')
    } else {
      el.style.setProperty('--row-y', `${start}px`)
    }
  }
  apply(initial)
  return {
    update: (next) => apply(next),
  }
}

/**
 * `use:virtualSpacer={totalSize}` — the total-height spacer that gives the
 * windowed list a scrollbar correct for its full size. The property it
 * writes is the spacer's `--rows-h`, the size axis `height-(--rows-h)` reads.
 */
export const virtualSpacer: Action<number | null | undefined> = (node, initial) => {
  const el = node as HTMLElement
  const apply = (total: number | null | undefined): void => {
    if (total == null || Number.isNaN(total)) {
      el.style.removeProperty('--rows-h')
    } else {
      el.style.setProperty('--rows-h', `${total}px`)
    }
  }
  apply(initial)
  return {
    update: (next) => apply(next),
  }
}

/**
 * `use:cellPad={indent}` — a first-column cell's `padding-inline-start`.
 *
 * A tree-grid's first column needs `padding-inline-start: calc(var(--density-row-px)
 * + indent)`; the indent is data (tree depth × INDENT_STEP, a number the
 * layout computes at runtime). For non-first columns, the cell has no
 * indent — the value is `var(--density-row-px)`.
 *
 * The cell's owning action owns the **whole value** because the computation
 * is "row padding + indent", a runtime calc that no static sx form can
 * express without a custom property. The action writes a complete CSS
 * value to `--cell-ps`; sx reads it via `ps-(--cell-ps)` which lowers to
 * `padding-inline-start: var(--cell-ps)`.
 *
 * `null` means "not a first column": the action removes the property so the
 * cell receives no `padding-inline-start` override (the row's default
 * density row padding applies through the `py-` token, separate axis).
 */
export const cellPad: Action<number | null | undefined> = (node, initial) => {
  const el = node as HTMLElement
  const apply = (indent: number | null | undefined): void => {
    if (indent == null || Number.isNaN(indent)) {
      el.style.removeProperty('--cell-ps')
    } else {
      el.style.setProperty(
        '--cell-ps',
        `calc(var(--density-row-px) + ${indent}px)`,
      )
    }
  }
  apply(initial)
  return {
    update: (next) => apply(next),
  }
}
