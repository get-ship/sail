# Tutorial scenarios

Per-step mock-daemon timelines played back by `sw.js`.

A scenario is a plain JS module that default-exports the per-source
map the service worker expects:

```js
export default {
  '/users': {
    initial: { kind: 'ok', value: { items: [{ id: 1, name: 'a' }] } },
    frames: [
      { afterMs: 1500, payload: { kind: 'ok', value: { items: [{ id: 1, name: 'a' }, { id: 2, name: 'b' }] } } },
      { afterMs: 3000, payload: { kind: 'error', error: { message: 'connection lost' } } },
    ],
  },
  // Mock REST endpoint (no SSE):
  'api:/whoami': { name: 'demo-user', id: 'u_42' },
}
```

The parent (`apps/docs/src/components/TutorialApp.astro`) imports the
scenario for the current tutorial step and posts it to the iframe
via `tutorial:scenario` before the first `tutorial:run`.

## Envelope shape

Every `'/path'` frame's `payload` should follow the SUI live-data
envelope contract:

```ts
type Envelope<T> =
  | { kind: 'loading' }
  | { kind: 'error'; error: { message: string } }
  | { kind: 'ok'; value: T }
```

The `initial` field is the first envelope sent on subscribe. Frames
are subsequent envelopes timed via `afterMs` from subscribe time.

## Authoring a scenario

1. Pick a step under `apps/docs/tutorial-examples/<step>/`.
2. Create `apps/docs/public/tutorial/scenarios/<step>.js` exporting
   the per-source map.
3. Wire the import in TutorialApp.astro by adding the step slug to
   the scenario-loader switch.
4. Reference the source in the step's template:
   `<data name="users" source="/users" live />`.
