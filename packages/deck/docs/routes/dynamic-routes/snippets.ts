// Code snippets for the dynamic-routes page (kept out of controller.ts).

export const routeExample = `routes/
└─ posts/
   └─ [id]/
      ├─ page.sui
      └─ controller.ts      →  /posts/:id`

export const readExample = `<!-- posts/[id]/page.sui -->
<h1>Post {route.params.id()}</h1>`
