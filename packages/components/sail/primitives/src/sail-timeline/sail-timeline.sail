<!--
  <sail-timeline> — vertical event list host. Default slot = items.
  Host role=list (set on mount when absent).
-->

<controller>

export default controller(() => {
  mount((host) => {
    if (!hostHasAttr(host, 'role')) hostSetAttr(host, 'role', 'list')
  })
  return {}
})

</controller>
<column gap="1" style({ box: place({ position: relative }) })><slot></slot></column>
