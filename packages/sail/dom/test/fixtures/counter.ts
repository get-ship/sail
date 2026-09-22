import { signal } from '@ship/sail-runtime'

export default function setup() {
  return { count: signal(0) }
}
