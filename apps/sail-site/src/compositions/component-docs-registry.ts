// Bridge: route controllers import from table homes (components/compositions/stores/actions/themes).
// Re-export the docs-package registry so the thin detail shell can resolve meta.
export {
  DOC_BY_SLUG,
  DOC_PACKAGES,
  FIRST_CLASS_HOSTS,
  docMeta,
} from '../component-docs/registry'
export type { DocCategory, DocKind, DocMeta } from '../component-docs/types'
