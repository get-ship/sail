// Code snippets for the deck reactivity page. Kept OUT of the controller so
// the compiler's scope scan isn't fooled by binding-shaped patterns inside
// these strings. Both are real deck route shapes:
//   - `route` is the reactive URL handle deck injects into every route scope;
//     its fields are signals (packages/sail/runtime/src/route-signals.ts).
//   - `query()` is deck's live-data seam, exactly as the conformance
//     route drives it (crates/deck/deck-conformance/app/routes/feed/page.sail).
export const code = {
  // The URL as a reactive source. Navigating re-runs only the bindings that
  // read `route`, with no document reload and no component re-mount.
  routeSignals: `<!-- routes/posts/[id]/page.sail -->
<!-- \`route\` is ambient in every deck route; its fields are signals. -->
<h1>Post {route.params.id()}</h1>
<p>Sorted by {route.query.sort() ?? 'newest'}</p>

<!-- Follow this link and only the two bindings above re-run.
     No document reload, no component re-mount. -->
<a href="/posts/42?sort=top">Post 42, top →</a>`,

  // A live server-data source. deck subscribes the route to the stream and
  // patches \`t\` as values arrive; only the bindings reading it re-render.
  liveData: `<!-- routes/feed/page.sail — subscribe a route to live server data -->
<controller>
import { query } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ t: query(() => call('api.demo.clock', {})) }))
</controller>

<while-loading {t}><p>Connecting…</p></while-loading>
<while-error {t}> as="err"><p>Lost: {err.message}</p></while-error>
<p>state: {t().kind}</p>`,
}
