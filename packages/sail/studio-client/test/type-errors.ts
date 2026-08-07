/**
 * Type-level contract tests: wrong arg types must fail to compile.
 * Included in tsconfig; @ts-expect-error proves wrong types are rejected.
 */
import {
  sailAuthoringSurface,
  sailThemeList,
  sailThemeFontSearch,
  sailAuthoringEdit,
  sailTaskGet,
} from '../src/index'

// @ts-expect-error topic must be string, not number
void sailAuthoringSurface({ topic: 123 })

// @ts-expect-error path must be string, not number
void sailThemeList({ path: 42 })

// @ts-expect-error id is required on TaskGetArgs
void sailTaskGet({ project: 'apps/x' })

// @ts-expect-error limit must be number, not string
void sailThemeFontSearch({ query: 'x', limit: 'many' })

// @ts-expect-error writeSource must be boolean
void sailAuthoringEdit({ ops: [], writeSource: 'yes' })

// Valid calls — ensure the file typechecks when contracts are correct
void sailAuthoringSurface({ topic: 'core' })
void sailThemeFontSearch({ query: 'inter', category: 'sans-serif', limit: 10 })
void sailAuthoringEdit({
  path: 'x.sail',
  ops: [],
  source: '<page></page>',
  writeSource: true,
})
