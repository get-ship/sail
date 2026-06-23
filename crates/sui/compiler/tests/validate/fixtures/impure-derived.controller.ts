import { controller } from '@ship/sui-runtime'

export default controller({
  derived: {
    x: () => Date.now(),
  },
})
