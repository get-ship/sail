/**
 * Type-level contract tests: wrong arg types must fail to compile.
 * Included in tsconfig; @ts-expect-error proves wrong types are rejected.
 */
import {
  sailAuthoringSurface,
  sailThemeList,
  sailTaskGet,
} from '../src/index'

// @ts-expect-error topic must be string, not number
void sailAuthoringSurface({ topic: 123 })

// @ts-expect-error path must be string, not number
void sailThemeList({ path: 42 })

// @ts-expect-error id is required on TaskGetArgs
void sailTaskGet({ project: 'apps/x' })

// Valid call — ensures the file typechecks when contracts are correct
void sailAuthoringSurface({ topic: 'core' })
