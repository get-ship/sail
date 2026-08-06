// Template hooks — `<use-*>` elements the runtime ships built-in.
//
// The runtime registers each hook by name at module load. Authors
// don't import these directly — they appear in `.sail` templates
// as `<use-name>` elements and the dom hydrate flow looks them up
// in the registry.
//
// Seven hooks ship — all browser-platform bridges, host-agnostic:
// `<use-kbd>`, `<use-click-outside>`, `<use-focus-trap>`,
// `<use-clipboard>`, `<use-title>`, `<use-scroll-anchor>`,
// `<use-intersection>`. Host-specific hooks (e.g. shell context)
// belong with the host, not the framework.

import { registerTemplateHook } from './registry'
import { useKbdHook } from './kbd'
import { useClickOutsideHook } from './click-outside'
import { useFocusTrapHook } from './focus-trap'
import { useClipboardHook } from './clipboard'
import { useTitleHook } from './title'
import { useScrollAnchorHook } from './scroll-anchor'
import { useIntersectionHook } from './intersection'

export {
  registerTemplateHook,
  getTemplateHook,
  clearRegistry,
  listRegisteredHooks,
} from './registry'

export type { KbdShortcut } from './kbd'

// Built-in hooks — registered eagerly at module load.
registerTemplateHook(useKbdHook)
registerTemplateHook(useClickOutsideHook)
registerTemplateHook(useFocusTrapHook)
registerTemplateHook(useClipboardHook)
registerTemplateHook(useTitleHook)
registerTemplateHook(useScrollAnchorHook)
registerTemplateHook(useIntersectionHook)
