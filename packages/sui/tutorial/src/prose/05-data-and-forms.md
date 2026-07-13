---
title: "Data and forms"
description: "live() makes loading, error, and ok explicit; bind: writes forms; refetch is a signal you bump."
order: 5
slug: "05-data-and-forms"
prev: "04-templates-react"
next: "06-motion-is-a-directive"
chapter: "state"
---

`live()` wraps any async source in a signal that walks a tagged union:

```
const team = live(() => {
  attempt()                     // tracked — bump it and the source re-runs
  return fetchTeam(breakApi())  // tracked too
})

// team() is one of:
//   { kind: 'loading' }
//   { kind: 'ok', value }
//   { kind: 'error', error }
```

The template renders all **three states explicitly** — the value is
only reachable behind `kind === 'ok'`, so a template cannot render data
it has not confirmed arrived:

```
{#while-loading of team}
  <spinner></spinner>
{/while-loading}

{#while-error err of team}
  <alert tone="destructive">{err.message}</alert>
{/while-error}

{#if team().kind === 'ok'}
  {#for m of team().value key={m.id}}…{/for}
{/if}
```

## Refetch is a signal you bump

There is no `refetch()` method. Any tracked signal read *inside* the
source thunk is a dependency; when one changes, the handle flips back
to `loading`, in-flight responses are invalidated, and the source runs
again. So the Reload button is one line — `attempt.set(n => n + 1)` —
and `add()` uses the *same* line after writing to the server: the list
refreshes through the one data path that already handles loading and
error.

## Forms are bind:

`bind:value` on inputs and selects, `bind:checked` on checkboxes —
each is sugar for a read plus the matching write event, the one place
a template both reads and writes:

```
<input bind:value={name} />
<select bind:value={role}>…</select>
<input type="checkbox" bind:checked={breakApi} />
```

## Try it

1. **Track the deps** (TODO 1). Add the `attempt` signal and read both
   `attempt()` and `breakApi()` inside the thunk. Now the "break the
   API" checkbox refetches on toggle — you never wired that.
2. **Cover the states** (TODO 4 + 5). Add the `{#while-loading}` and
   `{#while-error}` boundaries. Check "break the API", reload, and read
   your 503 in a proper alert; uncheck and watch it recover.
3. **Finish add()** (TODO 2 + 3). Push into the fake DB, clear the
   inputs, bump `attempt`. Notice the roster refreshes through the same
   loading state as first paint — no bespoke "optimistic insert" path.
4. **Gate the button.** Add a `canAdd` derived (`name` non-blank) and
   bind `disabled={!canAdd()}` — validation is just another read.
