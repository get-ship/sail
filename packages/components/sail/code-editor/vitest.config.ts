import {
  suiComponentVitestConfig,
  pinImportMetaUrlPlugin,
  shadowForComponentCompilePlugin,
} from '@ship/test-setup/vitest'

// jsdom — the controller is CodeMirror 6, which has no DOM dependency
// beyond a contenteditable host; the rest is canvas- and layout-free.
const base = suiComponentVitestConfig()

export default {
  ...base,
  plugins: [
    pinImportMetaUrlPlugin('sail-code-editor'),
    shadowForComponentCompilePlugin('sail-code-editor'),
    ...(base.plugins ?? []),
  ],
}
