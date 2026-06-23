import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ n: signal(0) }),
})
