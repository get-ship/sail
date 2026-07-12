import {
  suiComponentVitestConfig,
  pinImportMetaUrlPlugin,
  shadowForComponentCompilePlugin,
} from '@ship/test-setup/vitest'

// jsdom — the controller measures band/copy widths, but every measurement
// path has an explicit zero-width fallback (the minimum seamless shape), so
// jsdom's stubbed layout is enough to assert the cloning + a11y contract;
// tests that need real numbers patch getBoundingClientRect directly.
const base = suiComponentVitestConfig()

export default {
  ...base,
  plugins: [
    pinImportMetaUrlPlugin('sui-band'),
    shadowForComponentCompilePlugin('sui-band'),
    ...(base.plugins ?? []),
  ],
}
