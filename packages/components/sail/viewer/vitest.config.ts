import { suiComponentVitestConfig } from '@ship/test-setup/vitest'

// happy-dom (not jsdom): the viewer mounts arbitrary specimens through the
// engine — including components that inject modern scoped CSS — and jsdom 25's
// parser throws "Could not parse CSS stylesheet" on mount; happy-dom tolerates
// it. happy-dom also supports `attachShadow` + `adoptedStyleSheets`, the engine
// shadow-isolation path (Slice 1).
export default suiComponentVitestConfig({ environment: 'happy-dom' })
