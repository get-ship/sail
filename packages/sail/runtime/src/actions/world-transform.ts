import type { Action } from '../action'

/**
 * `use:worldTransform={cssTransform}` — apply the WORLD layer's compile-time-
 * composed transform string verbatim. The world layer's pan/zoom is CSS
 * (`matrix(...)` or `translate(...) scale(...)`), the authoring gate is "is
 * this a runtime measurement, or could a token express it?"
 *
 * Pan/zoom IS a runtime measurement — fingers on a trackpad, a pinch zoom.
 * No token for "the user has panned 47.3px to the right". The action writes
 * the value as a single custom property, and the template reads it through
 *
 *     style="transform-(--world-tx)"
 *
 * (the CSS `transform` property lowered through the `transform-(--var)` atom).
 *
 * The string is written verbatim — the compiler cannot intercept it, the
 * lint cannot token-check it, BECAUSE it is a measurement, not design. The
 * seam (custom property) holds, but the value itself is unchecked JS — the
 * kept-downside of "data crosses JS". The action exists because NOT having
 * one re-opens `.style.transform = …` in every controller that touches a
 * canvas.
 */
export const worldTransform: Action<string | null | undefined> = (node, initial) => {
  const el = node as HTMLElement
  const apply = (cssTransform: string | null | undefined): void => {
    if (cssTransform == null || cssTransform === '') {
      el.style.removeProperty('--world-tx')
    } else {
      el.style.setProperty('--world-tx', cssTransform)
    }
  }
  apply(initial)
  return {
    update: (next) => apply(next),
  }
}
