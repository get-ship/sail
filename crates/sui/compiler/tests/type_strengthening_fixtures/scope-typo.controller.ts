import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ count: signal(0) }),
})
