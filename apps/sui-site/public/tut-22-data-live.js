// Scenario for tutorial step 22 — `<data live source="ship.users.list()">`.
// The `ship.users.list()` proxy in host.html resolves to
// `/_deck/data/users/list`, which the service worker plays back from
// this timeline.
export default {
  '/users/list': {
    initial: { kind: 'loading' },
    frames: [
      {
        afterMs: 600,
        payload: {
          kind: 'ok',
          value: {
            items: [
              { id: 1, name: 'alpha' },
              { id: 2, name: 'beta' },
            ],
          },
        },
      },
      {
        afterMs: 2400,
        payload: {
          kind: 'ok',
          value: {
            items: [
              { id: 1, name: 'alpha' },
              { id: 2, name: 'beta' },
              { id: 3, name: 'gamma' },
            ],
          },
        },
      },
      {
        afterMs: 4800,
        payload: { kind: 'error', error: { message: 'connection lost' } },
      },
    ],
  },
}
