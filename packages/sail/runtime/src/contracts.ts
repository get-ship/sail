// @ship/sail-runtime/contracts — the OPT-IN JS injection of the framework contracts.
//
//     import '@ship/sail-runtime/contracts'
//
// Importing this module (for side effect) installs the ~30KB light-DOM contracts
// stylesheet from JavaScript at first component mount. It exists for JS-only hosts
// that cannot add a <link>, and for convenience in dev.
//
// Prefer linking the stylesheet. It is already materialized:
//
//     <link rel="stylesheet" href="…/contracts.css" data-sail-contracts>
//
// (from `@ship/sail-tokens/contracts.css`). A <link> is cacheable, parses in parallel
// with the JS, and costs the JS bundle nothing. Importing this module puts the whole
// design system inside your JavaScript — which is fine if you asked for it, and was
// NOT fine when the framework did it to you unconditionally (it was half the
// js-framework-benchmark bundle, in an app that uses no SUI components).

import { LIGHT_CONTRACTS_CSS, layered } from './styles/index'
import { setContractsInstaller } from './styles/contracts-seam'

let installed = false

setContractsInstaller((): void => {
  if (installed || typeof document === 'undefined') return
  installed = true
  const id = 'sail-light-contracts'
  if (document.getElementById(id)) return
  // A host that already SERVES the contracts asset wins — never inject twice.
  if (document.querySelector('link[data-sail-contracts], style[data-sail-contracts]')) return
  const style = document.createElement('style')
  style.id = id
  // Injected EARLY (head, before per-component scoped styles) so a component's own
  // rules win ties by source order.
  style.textContent = layered(LIGHT_CONTRACTS_CSS)
  document.head.prepend(style)
})
