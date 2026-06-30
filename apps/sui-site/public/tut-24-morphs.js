// Scenario for tutorial step 24 — mutations + server push.
//
// POST /_deck/api/users/add  → SW runs `mutations['/users/add']`
// against the current /users/list value, then pushes the new
// envelope on every active SSE controller subscribed to that source.

export default {
  '/users/list': {
    initial: {
      kind: 'ok',
      value: {
        items: [
          { id: 1, name: 'alpha' },
          { id: 2, name: 'beta' },
        ],
      },
    },
    mutations: {
      '/users/add': (value, body) => ({
        items: [
          ...(value?.items ?? []),
          { id: Date.now(), name: String((body?.name ?? '').trim() || 'anon') },
        ],
      }),
    },
  },
}
