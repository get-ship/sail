import {
  sailComponentVitestConfig,
  pinImportMetaUrlPlugin,
  shadowForComponentCompilePlugin,
} from '@ship/test-setup/vitest'

// jsdom — mermaid's own renderer writes SVG to a div it controls; the
// component never reaches outside the host. No measurement-dependent
// layout, so jsdom's stubs are enough.
const base = sailComponentVitestConfig()

export default {
  ...base,
  plugins: [
    pinImportMetaUrlPlugin('sail-mermaid'),
    shadowForComponentCompilePlugin('sail-mermaid'),
    ...(base.plugins ?? []),
  ],
}
