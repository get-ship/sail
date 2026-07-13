import {
  suiComponentVitestConfig,
  pinImportMetaUrlPlugin,
  shadowForComponentCompilePlugin,
} from '@ship/test-setup/vitest'

// happy-dom — the component injects modern scoped CSS (`@scope`), which jsdom
// 25's parser rejects on mount. Layout is stubbed either way: the controller
// measures band/copy widths and every measurement path has an explicit
// zero-width fallback (the minimum seamless shape), so the cloning + a11y
// contract asserts fine; tests that need real numbers patch
// getBoundingClientRect directly.
const base = suiComponentVitestConfig({ environment: 'happy-dom' })

export default {
  ...base,
  plugins: [
    pinImportMetaUrlPlugin('sui-band'),
    shadowForComponentCompilePlugin('sui-band'),
    ...(base.plugins ?? []),
  ],
}
