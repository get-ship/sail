// Code samples for the File-routing page. Kept in a plain data module (not the
// paired controller.ts) so the SUI compiler never scope-scans them for signals.

export const routeTree = `routes/
  page.sail              → /
  about/page.sail        → /about
  blog/posts/page.sail   → /blog/posts`

export const colocated = `routes/blog/
  page.sail          the /blog markup
  controller.ts     reactive state for the page
  style.css         styles for the page`
