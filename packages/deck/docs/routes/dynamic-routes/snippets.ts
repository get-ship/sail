// Code snippets for the dynamic-routes page (kept out of controller.ts).

export const routeExample = `routes/
└─ posts/
   └─ [id]/
      ├─ page.sail
      └─ controller.ts      →  /posts/:id`

export const readExample = `<!-- posts/[id]/page.sail -->
<h1>Post {route.params.id()}</h1>`
