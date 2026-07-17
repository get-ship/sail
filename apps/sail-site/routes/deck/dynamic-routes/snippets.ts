// Code snippets for the deck dynamic-routes page.

export const routeExample = `routes/
└─ posts/
   └─ [id]/
      └─ page.sui      →  /posts/:id`

export const readExample = `<!-- posts/[id]/page.sui -->
<h1>Post {route.params.id()}</h1>`
