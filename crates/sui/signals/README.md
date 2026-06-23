# sui-signals

Fine-grained reactive signals for the Ship daemon. `Send + Sync`, safe to share across tokio tasks.

Mirrors the algorithm in `packages/ui-runtime/src/signal.ts`: signals are sources, `derived` values are memoized computeds, `effect` subscribes and re-runs on dependency change.

## API

```rust
use sui_signals::{Signal, Derived, Disposer, effect, derived, batch, untrack};

// Source signal — any Clone + Send + Sync + 'static type.
let count: Signal<i32> = Signal::new(0);

// Memoized computed.
let doubled: Derived<i32> = derived({
    let c = count.clone();
    move || c.get() * 2
});

// Side effect — runs eagerly, re-runs when deps change.
let dispose: Disposer = effect({
    let d = doubled.clone();
    move || println!("doubled = {}", d.get())
});

// Write — notifies synchronously.
count.set(5);      // prints "doubled = 10"

// Coalesce writes.
batch(|| {
    count.set(1);
    count.set(2);  // effect runs once, not twice
});

// Non-tracking read.
let snapshot = untrack(|| count.get());

// Stop the effect.
drop(dispose);
```

## Signal methods

| Method | Description |
|--------|-------------|
| `Signal::new(T)` | Create a signal with an initial value |
| `sig.get()` | Read and subscribe current scope |
| `sig.peek()` | Read without subscribing |
| `sig.set(T)` | Write; skips notification if value unchanged (`T: PartialEq`) |
| `sig.set_unchecked(T)` | Write; always notifies |
| `sig.clone()` | Cheap Arc clone — all clones share state |

## Signal-backed vs namespace-event SDK sources

The daemon supports two patterns for `<data live />` sources:

**Namespace-event path** (existing, unchanged): `run_data_source` in `sui_sse.rs` spawns a kernel actor per SSE connection, subscribes to `"config"` namespace events, and re-resolves the registry call on each event. Fine-grained, low-overhead for sources with no shared state.

**Signal-backed path** (this crate): a singleton `Signal<T>` is maintained for the source. A single background task subscribes to kernel events and calls `signal.set(new_value)`. Each SSE connection subscribes to the signal via `effect()` — no kernel actor per connection. Choose this pattern when:

- Multiple connections observe the same underlying data (e.g., `git.overview` is the same for every connected client)
- You want other daemon code (not just SSE) to read the current value without spawning an actor
- The data update cadence is tied to a long-lived background task rather than per-request resolution

## SDK source authoring — signal pattern

```rust
// 1. Static singleton
static MY_SIGNAL: OnceLock<Signal<serde_json::Value>> = OnceLock::new();

pub fn my_signal() -> &'static Signal<serde_json::Value> {
    MY_SIGNAL.get_or_init(|| Signal::new(serde_json::Value::Null))
}

// 2. Background updater (started once, updates signal on events)
async fn run_updater(state: ApiState) { /* subscribe to kernel, call my_signal().set_unchecked(new_val) */ }

// 3. SSE bridge (per connection)
let _d = sui_signals::effect({
    let sig = my_signal().clone();
    let tx = notify_tx.clone();
    move || { let _ = sig.get(); let _ = tx.try_send(()); }
});
```

See `apps/shipd/src/views/git_signal.rs` for the full `git.overview` implementation.

## Send + Sync contract

`Signal<T>` is `Send + Sync` when `T: Clone + Send + Sync + 'static`. It is safe to:

- Store in `static` via `OnceLock<Signal<T>>`
- Clone and move into `tokio::spawn` tasks
- Read/write from multiple threads simultaneously

Effects fire synchronously on the thread that calls `set()`. For async consumers (SSE handlers), use a `tokio::sync::mpsc` or `watch` channel as a bridge — see `run_signal_data_source` in `sui_sse.rs`.
