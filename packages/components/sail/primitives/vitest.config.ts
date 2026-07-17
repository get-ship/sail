import {
  suiComponentVitestConfig,
  pinImportMetaUrlPlugin,
  shadowForComponentCompilePlugin,
} from '@ship/test-setup/vitest'

// Default env (jsdom) for the whole suite — every existing component test
// relies on it (the form components crash under happy-dom's HTMLParser). The one
// renderComponent suite that needs happy-dom (sail-calendar, whose scoped CSS
// trips jsdom's parser) opts in PER-FILE via a `// @vitest-environment happy-dom`
// docblock, so it can never regress the rest of the suite again.
const base = suiComponentVitestConfig()

export default {
  ...base,
  plugins: [
    pinImportMetaUrlPlugin('sail-primitives'),
    shadowForComponentCompilePlugin('sail-primitives'),
    ...(base.plugins ?? []),
  ],
}
