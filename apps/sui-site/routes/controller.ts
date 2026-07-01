// Ship UI landing controller. The landing is a static umbrella page (hero +
// product cards); no scope state to own.
import { controller } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: () => ({}),
})
