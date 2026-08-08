// File-routing page samples. Plain data module — never scope-scanned.

export const routeTree = `routes/
  page.sail              → /
  about/page.sail        → /about
  blog/posts/page.sail   → /blog/posts`

export const colocated = `routes/blog/
  page.sail          the /blog markup
  controller.ts     reactive state for the page
  style.css         styles for the page`
