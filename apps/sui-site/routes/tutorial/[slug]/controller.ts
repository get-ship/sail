// Dynamic tutorial lesson page — the "Tutorial — refined (2a)" layout. Prose +
// progress stepper on the LEFT; an interactive playground (editor card + live
// preview + output console) on the RIGHT. Ported from ship-marketing's proven
// TutorialApp.
//
// The slug comes from the route param (`ctx.route.params.slug()`). All
// editor↔preview wiring (debounce, regex TS→JS, postMessage, theme sync, the
// output-console log) lives in the imported `tutorial-boot` module — kept OUT of
// this controller body so the compiler's scope-scanner never trips on lesson
// code strings or the `signal(…)` / `derived(…)` text snippets inside.
import {
  controller,
  derived,
  effect,
  ref,
  signal,
  type ViewScope,
} from '@ship/sui-runtime'
import { ensureWasm, makeCompile } from '@ship/sui-viewer'
import type { Scope } from './page.sui'
import { TUTORIAL_LESSONS, type TutorialLesson } from '../../../src/tutorial-data'
import { bootTutorialApp, type CompileFn, type TutorialHandle } from '../../../src/tutorial-boot'

const BY_SLUG: Record<string, TutorialLesson> = {}
for (const l of TUTORIAL_LESSONS) BY_SLUG[l.slug] = l

// Lessons in presentation order — drives the progress stepper. RAW_LESSONS is
// authored per-chapter, so sort by `order` to guarantee a monotonic 1..N strip.
const ORDERED_LESSONS: TutorialLesson[] = [...TUTORIAL_LESSONS].sort((a, b) => a.order - b.order)

// One thin segment per lesson. `cls` is precomputed so the template stays a pure
// read (no class interpolation) and `current` drives aria-current + the tint.
interface StepSegment {
  slug: string
  href: string
  cls: string
  title: string
  current: boolean
}

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const lesson = derived(() => BY_SLUG[ctx.route.params.slug() ?? ''])
    const nextSlug = derived(() => lesson()?.next ?? '')

    const rootEl = ref<HTMLElement>(null)
    // compile MUST be a signal: it's set asynchronously after the wasm loads,
    // and the boot effect below has to re-run when it becomes available. As a
    // plain `let` the effect ran once (compile still null), returned, and never
    // re-ran — so the tutorial never booted (empty editors, no preview).
    const compile = signal<CompileFn | null>(null)
    // Store makeCompile() as the signal's VALUE via a thunk. `compile` holds a
    // function, and signal.set(fn) treats fn as an UPDATER (calls it with the
    // current value) — so set(makeCompile()) would invoke the compiler with a
    // null source. The `() => …` thunk returns the fn to store, uncalled.
    ensureWasm('/sui_compiler_bg.wasm')
      .then(() => compile.set(() => makeCompile() as unknown as CompileFn))
      .catch((err) => console.error('[sui-site] wasm compile seam failed:', err))

    let handle: TutorialHandle | null = null
    let lastSlug = ''
    effect(() => {
      const root = rootEl()
      const l = lesson()
      const c = compile()
      if (!root || !c) return
      if (!handle) handle = bootTutorialApp(root, c)
      if (l && l.slug !== lastSlug) {
        lastSlug = l.slug
        handle.setLesson({
          slug: l.slug,
          starterTemplate: l.starterTemplate,
          starterController: l.starterController,
          solvedTemplate: l.solvedTemplate,
          solvedController: l.solvedController,
        })
      }
    })

    const steps = derived((): StepSegment[] => {
      const cur = lesson()?.order ?? 0
      return ORDERED_LESSONS.map((l) => {
        const state = l.order < cur ? 'done' : l.order === cur ? 'current' : 'todo'
        return {
          slug: l.slug,
          href: '/tutorial/' + l.slug,
          cls: 'tut-step tut-step--' + state,
          title: 'Lesson ' + l.order + ' — ' + l.title,
          current: state === 'current',
        }
      })
    })

    const pad2 = (n: number): string => (n < 10 ? '0' + n : String(n))

    return {
      title: derived(() => lesson()?.title ?? 'Lesson not found'),
      description: derived(() => lesson()?.description ?? ''),
      proseHtml: derived(() => lesson()?.proseHtml ?? ''),
      rootEl,
      steps,
      lessonNumLabel: derived(() => pad2(lesson()?.order ?? 0)),
      totalLessons: derived(() => ORDERED_LESSONS.length),
      hasNext: derived(() => Boolean(nextSlug())),
      nextHref: derived(() => (nextSlug() ? '/tutorial/' + nextSlug() : '/tutorial')),
      nextTitle: derived(() => (BY_SLUG[nextSlug()]?.title ?? '') + ' →'),
    }
  },
})
