# @ship/ui-runtime

The browser-side runtime for the ship-set framework. Layer 1 of the stack.

Fine-grained reactive signals, custom-element wrapper, patch-protocol applicator. Compiled `.set` modules import from this package.

**Status:** under construction. v0.1 has the signal engine; v1.0 ships when [03-runtime-architecture.md](../../.artifacts/specs/ship-set/03-runtime-architecture.md) is implementation-complete.

## API

```ts
import { signal, derived, effect, batch, untrack } from '@ship/ui-runtime'

const $count = signal(0)
const $double = derived(() => $count() * 2)

effect(() => {
  console.log('count is', $count(), 'double is', $double())
})

$count.set(5)
// → 'count is 5 double is 10'

batch(() => {
  $count.set(10)
  $count.set(20)
})
// → 'count is 20 double is 40' (one fire, not two)
```

## Public API surface

Every symbol below is re-exported from `@ship/ui-runtime` and is part of
the package's stable contract. Hover any of them in your editor to read
the full JSDoc — examples, parameter docs, and `@see` cross-references.

Anything **not** listed here is internal and may change without a
semver bump. Treat the absence of an export from `src/index.ts` as a
"do not import" signal.

### Reactivity (`signal.ts`)

| Symbol     | Kind     | Summary                                                  |
| ---------- | -------- | -------------------------------------------------------- |
| `signal`   | function | Create a fine-grained reactive signal.                   |
| `derived`  | function | Memoized computed; re-runs when a tracked dep changes.   |
| `effect`   | function | Eager + reactive side effect; returns a disposer.        |
| `batch`    | function | Coalesce multiple writes into one notification cycle.    |
| `untrack`  | function | Read signals without subscribing the current observer.   |
| `Signal`   | type     | Reader + setter pair — the shape every signal exposes.   |
| `Derived`  | type     | Read-only signal-like view over a memoized computation.  |

### DOM bindings (`bind.ts`, `raw-html.ts`)

| Symbol         | Summary                                                            |
| -------------- | ------------------------------------------------------------------ |
| `bindText`     | Reactive `textContent`.                                            |
| `bindShow`     | Toggle visibility via `display:none`.                              |
| `bindAttr`     | Reactive HTML attribute (with presence semantics for `null`/`""`). |
| `bindClass`    | Toggle a single class.                                             |
| `bindProp`     | Reactive JS property (typed, no string round-trip).                |
| `bindValue`    | Two-way bind a string signal to an input/textarea/select.          |
| `bindChecked`  | Two-way bind a boolean signal to a checkbox/radio.                 |
| `bindRawHtml`  | Reactive `innerHTML` (author sanitises).                           |
| `Disposer`     | Type alias for the `() => void` cleanup every binding returns.     |

### Events (`on.ts`)

| Symbol       | Summary                                                  |
| ------------ | -------------------------------------------------------- |
| `on`         | Attach a listener with ship-set's modifier semantics.        |
| `matchesKey` | Match a `KeyboardEvent` against a `key:` modifier spec.  |
| `OnOptions`  | Modifier list + `addEventListener` flags.                |
| `Modifier`   | One pre-handler effect (`preventDefault`, `key:`, etc.). |

### Control flow (`for.ts`, `if.ts`, `await.ts`)

| Symbol             | Summary                                                       |
| ------------------ | ------------------------------------------------------------- |
| `bindFor`          | Keyed list reconciler.                                        |
| `bindForSignal`    | `bindFor` convenience for signal-backed lists.                |
| `bindIf`           | Conditional branch render.                                    |
| `bindAwait`        | Promise-driven branch render (loading / then / catch).        |
| `BindForOptions`   | Inputs to `bindFor`.                                          |
| `RenderedItem`     | Output of a `bindFor` row renderer.                           |
| `BindIfOptions`    | Inputs to `bindIf`.                                           |
| `BranchRender`     | One branch's nodes + cleanup (shared by `bindIf`/`bindAwait`).|
| `BindAwaitOptions` | Inputs to `bindAwait`.                                        |

### Patch protocol (`patch.ts`)

| Symbol           | Summary                                                |
| ---------------- | ------------------------------------------------------ |
| `applyPatch`     | Apply one patch to a signal (immutable, copy-on-write). |
| `applyPatches`   | Apply many patches as one batched update.              |
| `tokenizePath`   | Parse a patch path (`a.b[3].c`) into tokens.           |
| `Patch`          | Wire envelope: op + path + op-specific fields.         |
| `PatchOp`        | Op tag union.                                          |
| `PathToken`      | One parsed path segment.                               |

### Components (`component.ts`)

| Symbol             | Summary                                              |
| ------------------ | ---------------------------------------------------- |
| `defineComponent`  | Register a ship-set component as a custom element.       |
| `camelOf`          | Kebab-case attribute → camelCase prop name.          |
| `ComponentFactory` | Per-instance setup function.                         |
| `ComponentSchema`  | Observed-attrs + decoders + form-association flag.   |

### Prop decoders (`decode.ts`)

| Symbol           | Summary                                          |
| ---------------- | ------------------------------------------------ |
| `decodeString`   | `null` → `''`, otherwise pass through.           |
| `decodeNumber`   | `Number(raw)`; missing/empty → `NaN`.            |
| `decodeBoolean`  | HTML attribute-presence semantics.               |
| `decodeJson`     | `JSON.parse` with a fallback on error.           |
| `decodeEnum`     | Allow-list with a fallback on miss.              |
| `Decoder`        | Type alias `(raw: string \| null) => T`.         |

### Slots (`slot.ts`)

| Symbol             | Summary                                                       |
| ------------------ | ------------------------------------------------------------- |
| `resolveSlots`     | Distribute children into slot anchors at hydrate time.        |
| `snapshotChildren` | Capture host children before innerHTML replacement.           |
| `slotFilled`       | Did the named slot get author-provided content?               |
| `SlotDecl`         | One slot's name + anchor + optional fallback renderer.        |

### Data (`data.ts`)

| Symbol        | Summary                                                          |
| ------------- | ---------------------------------------------------------------- |
| `data`        | Server-owned signal with optional SSE patch subscription.        |
| `DataOpts`    | Initial value + SSE URL + patch / error hooks.                   |
| `DataHandle`  | `{ signal, dispose }` — the value plus a teardown.               |

### HMR / hydration (`morph.ts`)

| Symbol         | Summary                                                |
| -------------- | ------------------------------------------------------ |
| `morph`        | In-place DOM reconciler that preserves node identity.  |
| `MorphOptions` | `skip` predicate to leave subtrees untouched.          |

## Chrome IPC (`chrome.ts`)

Typed wrapper around the shell-owned chrome surfaces. Replaces the
ad-hoc `dispatchEvent(new CustomEvent('sui:notify', …))` pattern with a
single discoverable API. Every call dispatches a `CustomEvent` on
`document`; the shell (`apps/shell/src/chrome/registry.ts`) owns the
listeners. Old event shapes still fire — the shell handles raw +
wrapper-emitted events identically, so legacy call sites keep working.

```ts
import { chrome } from '@ship/sui-runtime'

chrome.notify({ tone: 'success', title: 'Saved' })
chrome.navigate('/agents/foo')
chrome.dock.toggle()
chrome.bar.open('find')
chrome.breadcrumb.set([{ label: 'Agents', href: '/agents' }, { label: 'foo' }])
chrome.status.set('agents.running', { label: '3', tone: 'info' })
chrome.theme.set('--ship-accent', '#ff0000')
chrome.focus.enter()
const action = await chrome.modal.request({
  title: 'Discard changes?',
  actions: [
    { label: 'Cancel', cancel: true },
    { label: 'Discard', tone: 'destructive' },
  ],
})
```

### Slot taxonomy + ownership

| Slot              | Cardinality                    | Auto-cleared on view unmount |
| ----------------- | ------------------------------ | ----------------------------- |
| `notify`          | Fire-and-forget toast          | n/a (toast self-dismisses)    |
| `navigate`        | One-shot URL change            | n/a                           |
| `dock.*`          | Singleton chrome surface       | n/a                           |
| `bar.*`           | Singleton chrome surface       | n/a                           |
| `breadcrumb`      | One source (last set wins)     | yes                           |
| `status`          | Per-key map (one owner per key)| yes (per key)                 |
| `theme`           | Per-token CSS override         | yes (per token)               |
| `focus`           | Boolean stack (push/pop)       | yes (pops view's pushes)      |
| `modal.request`   | Serialized queue               | n/a (Promise resolves)        |

### Auto-clear pattern

State-bearing slots (breadcrumb / status / theme / focus) stamp each
call with the caller's view id. The shell maintains a
`viewId → claims` registry; when a mount disposes the shell fires
`sui:view:unmount` and the registry releases that view's claims —
breadcrumb clears, status keys clear, theme tokens revert, focus pushes
pop. No teardown bookkeeping required in the controller.

The view id is read from `window.__suiCurrentViewId`, which the shell
updates as the mounted route changes. Callers can override per-call
with the optional `{ viewId }` opt — useful for cross-mount handles or
unit tests.

### Modal response

`chrome.modal.request({ title, actions })` returns a `Promise<string>`
that resolves with the label of the chosen action. The shell renders a
queued native `<dialog>` and dispatches `sui:modal:respond` with
`{ id, action }`; the wrapper resolves the matching pending request.
Mark which action triggers on ESC / backdrop close with
`{ label, cancel: true }`.

## Spec

See `.artifacts/specs/ship-set/03-runtime-architecture.md` for the full design.

## Tests

```
bun run test
```
