// Side-effect CSS loads for the site shell. Controllers may not side-effect
// import even `@ship/*` CSS (sail-controller-import-convention); value-import
// this module so themes/products/sail-base attach with the header graph.
import '@ship/tokens/themes.css'
import '@ship/tokens/products.css'
import '@ship/tokens/sail-base.css'

/** Touch so the import is a value binding, not a side-effect-only edge. */
export const siteTokenCss = true as const
