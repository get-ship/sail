// Motion singleton entry (`@ship/sail-runtime/motion` → `/_deck/motion.js`).
//
// Style `fx:` compiles to `__sail_motion_attach`. Scrub-lane `timeline` /
// `clock` / `scrollProgress` / `springValue` are the same download. Hello-world
// does not import this module.

export { __sail_motion_attach } from './motion'
export type { MotionAttachConfig, MotionConfig } from './motion'
export { timeline, SailTimelineError } from './timeline'
export type {
  TimelineValue,
  TimelineEasing,
  TimelineEasingName,
  TimelineStop,
  TimelineStopOptions,
  TimelineTrack,
  TimelineTracks,
  TimelineSignals,
  TimelineOptions,
} from './timeline'
export { clock, scrollProgress, springValue } from './drivers'
export type { ClockOptions, ClockSignal, ScrollProgressOptions, SpringOptions } from './drivers'
