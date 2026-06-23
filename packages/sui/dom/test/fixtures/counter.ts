import { signal } from '@ship/sui-runtime'

export default function setup() {
  return { count: signal(0) }
}
