import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ count: signal(0) }),
  derived: {
    doubled: ({ count }) => count() * 2,
    label: ({ count }) => `count is ${count()}`,
  },
  on: {
    inc: ({ count }) => count.set(count() + 1),
  },
})
