<!--
  <toggle-group> — segmented Style match; structural orientation stamp on cells.
  Size/variant paint is authored on each toggle.
  Vertical: <row> dir(column) + align=stretch so cells share one column width.

  State: this file's <controller> (value + select + keys + focus). No DOM.
  Toggles take `groupValue` / `select` / `focus` / `tabStop` / `orientation`
  from this instance (signal cells; `value` on the cell stays the cell's own).
  Listen: use:hotkeys={keys} on the host; focus moves are `use:focus` on each
  toggle; the roving tabindex is a derived binding. Click is the platform
  <button> (Space/Enter included).
-->

<scope>
  props: {
    type: 'single' | 'multiple' = 'single'
    value = ''
    variant: 'outline' | 'ghost' | 'segmented' = 'outline'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    onToggleChange: (detail: { value: string }) => void
  }

  style host = {display(inline-flex)}
  style horiz = {dir(row)}
  style vert = {dir(column)}
  style segmented = {pad(1).bg(muted).border({ color: border, width: 1 }).radius(md)}
</scope>

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  const values = Array.isArray(init.names) ? init.names : []
  const isMulti = () => read(init.type) === 'multiple'
  const value = signal(String(read(init.value) || ''))
  const focusValue = signal('')
  const orientation = derived(() => (read(init.orientation) === 'vertical' ? 'vertical' : 'horizontal'))

  function invoke(v) {
    const cb = init.onToggleChange
    if (typeof cb === 'function') cb({ value: v })
  }
  function select(v) {
    const mine = String(v ?? '')
    if (!mine) return
    const set = new Set(value().split(/\s+/).filter(Boolean))
    if (isMulti()) {
      if (set.has(mine)) set.delete(mine)
      else set.add(mine)
      value.set([...set].join(' '))
    } else {
      value.set(set.has(mine) ? '' : mine)
    }
    invoke(value())
  }
  function go(v) {
    if (!v) return
    focusValue.set(v)
    if (!isMulti()) {
      value.set(v)
      invoke(v)
    }
  }
  function noteFocus(v) {
    if (typeof v === 'string' && v) focusValue.set(v)
  }
  function move(delta) {
    if (!values.length) return
    let i = values.indexOf(focusValue())
    if (i < 0) i = 0
    go(values[(i + delta + values.length) % values.length])
  }
  const horiz = orientation() !== 'vertical'
  const keys = {
    [horiz ? 'ArrowRight' : 'ArrowDown']: () => move(1),
    [horiz ? 'ArrowLeft' : 'ArrowUp']: () => move(-1),
    Home: () => go(values[0]),
    End: () => go(values[values.length - 1]),
  }
  const tabStop = derived(() => focusValue() || (isMulti() ? (values[0] || '') : value()))
  return { value, groupValue: value, select, focus: focusValue, noteFocus, tabStop, keys, orientation }
})
</controller>

<row inline align="stretch" gap="0" use:hotkeys={keys} style(host, {match: { orientation: { horizontal: horiz, vertical: vert }, variant: {outline: bg(transparent), ghost: bg(transparent), segmented: segmented}}})>
  <slot></slot>
</row>
