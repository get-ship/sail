<!--
  <tab-panel> — owns its pad. Orientation is this element's prop.
  Selected follow is parent `active` as a prop + Style when { selected }.
-->

<scope>
  props: {
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }

  style base = {display(none).size({ min-w: 0 })}
  style on = {display(block)}
  style horizontal = {pad({ t: 4 }).size({ w: 100%, min-w: 0 })}
  style vertical = {pad({ t: 0, s: 4 }).grow(1).size({ min-w: 0 })}
</scope>

<controller>
export default controller((init) => {
  const name = String(init.name || '')
  const selected = derived(() => {
    const active = init.active
    return !!(typeof active === 'function' && name && active() === name)
  })
  const hidden = derived(() => !selected())
  return { selected, hidden }
})
</controller>

<div hidden={hidden()} style(base, { when: { selected: on }, match: { orientation: { horizontal: horizontal, vertical: vertical } } })
><slot></slot></div>
