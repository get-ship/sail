import {
  suiComponentVitestConfig,
  pinImportMetaUrlPlugin,
  shadowForComponentCompilePlugin,
} from '@ship/test-setup/vitest'

// happy-dom (not jsdom): the renderers compose `@ship/sail-primitives`
// (sail-table, sail-empty-state) which inject modern scoped CSS; jsdom 25's
// parser throws "Could not parse CSS stylesheet" on mount, happy-dom tolerates
// it. The default include globs (`test/**/*.test.ts` + `src/**/*.test.ts`) pick
// up both the relocated renderer suites (under test/) and the co-located engine
// specs (under src/engine/).
const base = suiComponentVitestConfig({ environment: 'happy-dom' })

export default {
  ...base,
  plugins: [
    pinImportMetaUrlPlugin('sail-media'),
    shadowForComponentCompilePlugin('sail-media'),
    ...(base.plugins ?? []),
  ],
}
