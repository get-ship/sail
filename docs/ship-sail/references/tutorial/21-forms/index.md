---
title: "Forms — form()"
description: "Field state, validation, submit lifecycle in one composition."
group: "Tutorial"
order: 21
slug: "21-forms"
prev: "20-controller-hooks"
next: "22-data-live"
chapter: "composition"
---

`form()` bundles every form concern — field signals, validation,
submit handler, pending state, last error — into one plain function
call. You call it once inside the controller, then read its
identifiers in the template the same way you'd read any signal.

```ts
import { controller, form } from '@ship/sail-runtime'

export default controller((ctx) => {
  const signup = form({
    initial: { name: '', email: '', accept: false },
    validate: ({ name, email, accept }) => ({
      name:  name.trim() === ''    ? 'required'              : null,
      email: !email.includes('@')  ? 'must include @'        : null,
      accept: !accept              ? 'you must accept terms' : null,
    }),
    submit: async ({ name, email }) => {
      await fetch('/api/signup', { method: 'POST', body: JSON.stringify({ name, email }) })
    },
  })

  return { form: signup }
})
```

Read in the template:

```sail
<form on:submit|preventDefault={(e) => form.submit(e)}>
  <label>name  <input bind:value={form.fields.name}   /></label>
  {#if form.errors().name}<p class="err">{form.errors().name}</p>{/if}

  <label>email <input bind:value={form.fields.email}  /></label>
  {#if form.errors().email}<p class="err">{form.errors().email}</p>{/if}

  <label><input type="checkbox" bind:checked={form.fields.accept} /> accept terms</label>

  <button type="submit" disabled={!form.valid() || form.pending()}>
    {form.pending() ? 'submitting…' : 'submit'}
  </button>

  {#if form.lastError()}<p class="err">{form.lastError().message}</p>{/if}
</form>
```

Identifiers returned from `form(...)` (here merged into template scope under the `form` key — you could just as easily spread them flat):

| Identifier | Kind | What it is |
|---|---|---|
| `form.fields.<fieldName>` | state signal | One per `initial` key — readable + writable via `bind:value`. |
| `form.values()` | derived | All current values as a plain object — for the submit handler / debug. |
| `form.errors()` | derived | `{ field → error \| null }` — the validate result. |
| `form.valid()` | derived | `true` when no field has an error. |
| `form.dirty()` | derived | `true` when any field differs from initial. |
| `form.pending()` | state signal | `true` while submit is in flight. |
| `form.lastError()` | state signal | The error thrown by the last submit, or `null`. |
| `form.submit(e?)` | handler | Calls `submit` with current values; manages pending + lastError. |
| `form.reset()` | handler | Reverts every field to its initial value. |

## Try it

The example is a small signup form with three fields and async
submit. Try:

1. **Validate**: leave name blank — the submit button stays
   disabled, and the error renders only after you touch the field.
2. **Submit**: fill the form, click submit. The button shows
   "submitting…" for a moment, then "submit" again. The simulated
   handler succeeds 70% of the time; otherwise `form.lastError()`
   surfaces the message.
3. **Reset**: add a "reset" button that calls `form.reset()` — every
   field reverts; `form.dirty()` flips back to `false`.

## Why this matters

Without `form()` you'd write a signal per field, a separate `errors`
signal, a submit handler that flips a `pending` boolean manually, and
a try/catch that captures the error. `form()` makes that the default.
You can still author your own form composition if you need a
different validation shape (zod, valibot, …) — it's just a function
that returns signals and helpers, called inline in the controller
like any other composition. No special interface to implement.
