<!-- Element capture via local use: action. -->
<controller>
import { controller, signal, type Action } from '@ship/sail-runtime'
export default controller((_ctx) => {
  const btnEl = signal<HTMLButtonElement | null>(null)
  const captureBtn: Action<void> = (node) => {
    btnEl.set(node as HTMLButtonElement)
    return { destroy: () => btnEl.set(null) }
  }
  return { captureBtn }
})
</controller>
<button use:captureBtn>click</button>
