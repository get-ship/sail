// Code samples for the File-routing page. Kept in a plain data module (not the
// paired controller.ts) so the SUI compiler never scope-scans them for signals.

export const routeTree = `routes/
  page.sui              → /
  about/page.sui        → /about
  blog/posts/page.sui   → /blog/posts`

export const colocated = `routes/blog/
  page.sui          the /blog markup
  controller.ts     reactive state for the page
  style.css         styles for the page`
