// contracts-seam.ts — the light-DOM contracts installer, behind an injection seam.
//
// WHY THIS EXISTS. The framework contracts are ~30KB of CSS. They used to be a
// string constant that `component-factory` referenced unconditionally at mount, so
// every bundle that mounted a SAIL component carried the entire design system inside
// its JAVASCRIPT — 49% of the js-framework-benchmark bundle was this stylesheet, in
// an app that renders plain <div>s with Bootstrap classes and uses no SAIL components
// at all. A bundler cannot drop a string the code demonstrably reads.
//
// The right home for a stylesheet is a stylesheet: a <link> is cacheable, parses on
// its own thread in parallel with the JS, and costs the JS bundle nothing. That was
// always the plan — the injector's own comment said the JS fallback "dies when the
// last JS-only host converts." It never did, so it kept charging every app.
//
// So: this seam is EMPTY by default. Nothing references the CSS, so the CSS shakes
// out. Two ways to get the contracts:
//
//   1. LINK THEM (preferred, and what a real app should do):
//        <link rel="stylesheet" href="/sail.css" data-sail-contracts>
//      The file is already materialized at @ship/sail-tokens/contracts.css.
//
//   2. IMPORT THE INSTALLER (for JS-only hosts, and convenient in dev):
//        import '@ship/sail-runtime/contracts'
//      That module fills this seam, and only then does the 30KB enter the bundle —
//      which is the honest deal: you pay for it exactly when you ask for it.

type Installer = () => void

let installer: Installer | null = null
let warned = false

/** Called by `@ship/sail-runtime/contracts` when an app opts into JS injection. */
export function setContractsInstaller(fn: Installer): void {
  installer = fn
}

/**
 * Called at component mount. A no-op unless an app opted into JS injection — which
 * is the entire point: with no installer registered, the contracts CSS is
 * unreferenced and the bundler drops it.
 */
export function ensureLightContracts(): void {
  if (installer) {
    installer()
    return
  }
  if (warned || typeof document === 'undefined') return
  warned = true
  // Failing silently here means an app renders unstyled and the developer has no
  // idea why — the single worst outcome of this change. Say it once, plainly.
  if (document.querySelector('link[data-sail-contracts], style[data-sail-contracts]')) {
    return // the host serves the stylesheet — the good path
  }
  // ...or the host has DELIBERATELY declined them: `<html data-sail-contracts="none">`.
  // A framework benchmark rendering plain markup, an embedded widget inheriting a
  // parent design system — both are legitimate, and neither should have to fake a
  // stylesheet tag to quiet a warning. Saying "no" is different from forgetting.
  if (document.documentElement.getAttribute('data-sail-contracts') === 'none') return
  console.warn(
    '[sail] A component mounted but the framework contracts are not present. Either link ' +
      'the stylesheet (`<link rel="stylesheet" href="…/contracts.css" data-sail-contracts>`, ' +
      'shipped as @ship/sail-tokens/contracts.css) or `import "@ship/sail-runtime/contracts"` ' +
      'to inject it from JS. Linking is preferred: it keeps ~30KB of CSS out of your JS bundle.',
  )
}
