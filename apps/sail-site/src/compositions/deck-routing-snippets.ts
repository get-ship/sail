// Code samples for the deck File-routing page. Plain data module — never
// scope-scanned by the compiler.

export const routeTree = `routes/
  page.sail              → /
  about/page.sail        → /about
  blog/posts/page.sail   → /blog/posts`

export const colocated = `<!-- routes/blog/page.sail — the whole /blog route -->
<controller>
  …reactive state for the page…
</controller>

<article>…the /blog markup…</article>`
