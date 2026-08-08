<!--
  <sail-timeline> — vertical event list host. Default slot = items.
  Host role=list (set on mount when absent).
-->

<controller>

export default controller(() => {
  mount((host) => {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'list')
  })
  return {}
})

</controller>
<column gap="1" style({ box: position(relative) })><slot></slot></column>
