import { setContractsInstaller } from './styles/contracts-seam'
import sailBaseCss from '../../tokens/src/sail-base.css?raw'

let installed = false

setContractsInstaller((): void => {
  if (installed || typeof document === 'undefined') return
  installed = true
  const id = 'sail-light-contracts'
  if (document.getElementById(id)) return
  if (document.querySelector('link[data-sail-contracts], style[data-sail-contracts]')) return
  const style = document.createElement('style')
  style.id = id
  style.setAttribute('data-sail-contracts', '')
  style.textContent = sailBaseCss
  document.head.prepend(style)
})
