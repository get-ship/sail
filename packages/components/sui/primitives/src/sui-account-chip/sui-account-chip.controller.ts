// <sui-account-chip> — signed-in identity display. The account
// state is the LiveState from `ship.account.state()` (SUI's
// hard rule #2: never call ship.* outside <data> or <form mutation>).
// The compiler auto-generates `account` as a Signal<LiveState<AccountState>>
// in scope. The controller normalises it via the shared
// `@ship/sdk-client` helpers and exposes `name` + `initials` to
// the template.
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
