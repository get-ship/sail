import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()

// Overlay hosts (dialog, drawer, popover, dropdown, tooltip, hover-card,
// context-menu): rest tests locate the trigger or named target. The panel
// is not required to be in the a11y tree at rest. menu-item is mounted
// inside dropdown / context-menu — it is not its own rest file. page /
// toggle / radio are parts of pagination / toggle-group / radio-group.
