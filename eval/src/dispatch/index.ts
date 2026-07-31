import type { DispatchMode } from '../types.ts'
import { OmpDispatch } from './omp.ts'
import { RecordedDispatch } from './recorded.ts'
import type { DispatchAdapter } from './types.ts'

export type { DispatchAdapter, DispatchRequest } from './types.ts'
export {
  OmpDispatch,
  buildOmpArgs,
  buildDispatchPrompt,
  resolveModelLabel,
} from './omp.ts'
export { RecordedDispatch, readTrajectoryJsonl } from './recorded.ts'

export function createDispatch(mode: DispatchMode): DispatchAdapter {
  switch (mode) {
    case 'recorded':
      return new RecordedDispatch()
    case 'omp':
      return new OmpDispatch()
    default: {
      const _exhaustive: never = mode
      throw new Error(`unknown dispatch mode: ${_exhaustive}`)
    }
  }
}
