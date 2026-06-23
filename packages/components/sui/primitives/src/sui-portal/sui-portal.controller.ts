import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-portal.sui'

export default controller<Scope>({
  mount(state, host) {

    const movedNodes: Node[] = []
    let currentTarget: Element | null = null

    const resolveTarget = (): Element | null => {
      const selector = state.$signals.target() || 'body'
      const el = document.querySelector(selector)
      if (!el) {
        console.warn(`[sui-portal] target "${selector}" not found in document`)
        return null
      }
      return el
    }

    const teleportFromHost = (target: Element) => {
      // Pull every light-DOM child of the portal element into the target.
      // appendChild moves the node (no clone), so it leaves the host's
      // light DOM in the same tick — slotchange will fire async, but the
      // listener is idempotent and only acts on nodes we haven't moved.
      const slot = state.$signals.portalSlot() as HTMLSlotElement | null
      const sources = slot ? slot.assignedNodes() : Array.from(host.childNodes)
      for (const node of sources) {
        target.appendChild(node)
        if (!movedNodes.includes(node)) movedNodes.push(node)
      }
    }

    const retargetAll = (next: Element) => {
      for (const node of movedNodes) next.appendChild(node)
    }

    const removeAll = () => {
      for (const node of movedNodes) {
        if (node.parentNode) node.parentNode.removeChild(node)
      }
      movedNodes.length = 0
    }

    const stopTargetEffect = effect(() => {
      const next = resolveTarget()
      if (!next) {
        currentTarget = null
        return
      }
      if (currentTarget === null) {
        teleportFromHost(next)
      } else if (currentTarget !== next) {
        retargetAll(next)
      }
      currentTarget = next
    })

    const slot = state.$signals.portalSlot() as HTMLSlotElement | null
    const onSlotChange = () => {
      if (!currentTarget || !slot) return
      // Any newly slotted nodes (post-mount appends to host's light DOM)
      // get pulled into the current target. Already-moved nodes have
      // left light DOM and won't show up in assignedNodes() again.
      for (const node of slot.assignedNodes()) {
        if (!movedNodes.includes(node)) {
          currentTarget.appendChild(node)
          movedNodes.push(node)
        }
      }
    }
    if (slot) slot.addEventListener('slotchange', onSlotChange)

    return () => {
      stopTargetEffect()
      if (slot) slot.removeEventListener('slotchange', onSlotChange)
      removeAll()
    }
  },
})
