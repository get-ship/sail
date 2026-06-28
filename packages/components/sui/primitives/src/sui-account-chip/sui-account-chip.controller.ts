// <sui-account-chip> — signed-in identity display. The account
// state is the LiveState from `ship.account.state()` (SUI's
// hard rule #2: never call ship.* outside <data> or <form mutation>).
// The compiler auto-generates `account` as a Signal<LiveState<AccountState>>
// in scope. The controller normalises it via the shared
// `@ship/sdk-client` helpers and exposes `name` + `initials` to
// the template.
//
// NEEDS-REVIEW: this controller carries FOUR upstream-coupled defects
// that block typecheck and a faithful runtime, all outside this pack's
// fix surface (the spec mandates `NEEDS-REVIEW:` rather than local
// stubs that silently drift from the SDK):
//
//   1. `account().value` reads `.value` off a `LiveState<T>` (loading |
//      error | { kind: 'ok'; value: T }) — only the `ok` kind has
//      `.value`. The right read is `unwrap(account())` (or pattern-
//      match on `account().kind`).
//   2. The named imports here — `readAccountIdentity`,
//      `accountDisplayName`, `accountInitials` — do NOT exist in
//      `@ship/sdk-client`. The package exports `readIdentity`,
//      `displayName`, `initials` (note the shorter names).
//   3. The `@ship/sdk-client` `AccountState` union is missing the
//      `'pending'` variant in its generated types — `account.ts` in
//      the SDK already handles `case 'pending'` and reads
//      `state.state_id`, but the type itself is
//      `{ state: "logged_out" } | { state: "logged_in"; user_id; me }`
//      with no `pending` member, so the switch never typechecks.
//   4. The `ship` LiveSource (declared in the .sui via `<client
//      name="ship" />`) returns `AccountState`; the `pending` variant
//      in the live stream is what the chip needs to render "completing
//      login…" — fixing (3) at the SDK source unblocks this.
//
// All four depend on edits inside the `@ship/sdk-client` package
// (republished from `localhost:4873`, NOT in this repo): rename the
// helpers, widen `AccountState` to include
// `{ state: 'pending'; state_id: string }`. Once the SDK ships, this
// controller rewrites in one pass — three renamed imports + the
// proper `unwrap(account())` call. Until then, the file stays broken
// here. Flagged per the spec's NEEDS-REVIEW protocol.
import { controller, derived } from '@ship/sui-runtime'
import {
  readAccountIdentity,
  accountDisplayName,
  accountInitials,
} from '@ship/sdk-client'
import type { Scope } from './sui-account-chip.sui'

export default controller({
  scope: ({ account }: Scope) => {
    const identity = derived(() => readAccountIdentity(account().value))
    const name = derived(() => accountDisplayName(identity()))
    const initials = derived(() => accountInitials(identity()))
    return { name, initials, hideName: () => false }
  },
})
