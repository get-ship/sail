// Code samples for the deck File-routing page. Plain data module — never
// scope-scanned by the compiler.

export const routeTree = `routes/
  page.sui              → /
  about/page.sui        → /about
  blog/posts/page.sui   → /blog/posts`

export const colocated = `<!-- routes/blog/page.sui — the whole /blog route -->
<controller>
  …reactive state for the page…
</controller>

<article>…the /blog markup…</article>`
