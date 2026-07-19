// Direct-emit hydrators.
//
// The compiler walks the SSR'd DOM via `walk(root, [path])` and dispatches
// to per-kind hydrators below. Each hydrator returns a disposer; the
// compiler-emitted `hydrate(root, ctx)` collects them into a single
// dispose function returned to `attach()`.

export { walk } from './walk'
export { hydrateText, collectTextOpens } from './text'
export { collectElementMarkers } from './markers'
export { hydrateAttr } from './attr'
export { hydrateClass } from './class'
export { hydrateStyle } from './style'
export { hydrateProp } from './prop'
export { hydrateOn } from './on'
export { hydrateBindValue, hydrateBindChecked } from './bind-value'
export { hydrateWhileLoading, hydrateWhileError } from './while'
