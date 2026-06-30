// <sui-account-chip> controller — generic, SDK-free identity display.
//
// The original @ship/chrome version read a live `ship.account.state()`
// subscription and normalised it through `@ship/sdk-client` helpers
// (`readIdentity`, `displayName`, `initials`). Here NONE of that exists:
// identity is plain props (`name`, `image`) and the chip is a pure display
// surface — the consuming app owns the subscription and feeds the chip.
// RUNTIME-FREE — imports `@ship/sui-runtime` only.

import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-account-chip.sui'

/** Two-letter initials from a display name (fallback "··"). */
function deriveInitials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean)
  if (parts.length === 0) return '··'
  if (parts.length === 1) return parts[0]!.slice(0, 2).toUpperCase()
  return (parts[0]![0]! + parts[parts.length - 1]![0]!).toUpperCase()
}

export default controller({
  scope: ({ name }: ViewScope<Scope>) => {
    const initials = derived(() => deriveInitials(name()))
    return { initials }
  },
})
