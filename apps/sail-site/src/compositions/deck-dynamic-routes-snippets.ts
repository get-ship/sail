// Code snippets for the deck dynamic-routes page.

export const routeExample = `routes/
└─ posts/
   └─ [id]/
      └─ page.sail      →  /posts/:id`

export const readExample = `<!-- posts/[id]/page.sail -->
<h1>Post {route.params.id()}</h1>`
