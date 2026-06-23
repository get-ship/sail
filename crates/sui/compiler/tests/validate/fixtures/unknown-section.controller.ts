import { controller } from '@ship/sui-runtime'

export default controller({
  derived: {
    count: () => 0,
  },
  effects: {
    boom: () => null,
  },
})
