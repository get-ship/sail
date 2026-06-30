// Dynamic tutorial lesson page — an INTERACTIVE, side-by-side experience
// (svelte.dev/tutorial-style), ported from ship-marketing's proven TutorialApp.
// Prose renders above two panes: LEFT an editable template.sui / controller.ts
// code area, RIGHT a sandboxed <iframe src="/tutorial/host.html"> that compiles
// + RUNS the lesson live (its own isolated /_deck/runtime.js).
//
// The slug comes from the route param (`ctx.route.params.slug()` — a signal),
// so the page re-targets the matched lesson on navigation. All lesson DATA and
// the editor↔preview wiring (regex TS→JS, postMessage, theme sync) live in
// imported modules (`tutorial-data`, `tutorial-boot`) — kept OUT of this
// controller body so the compiler's scope-scanner never trips on the lesson
// code strings / `signal(…)` text they contain.
import {
  controller,
  signal,
  derived,
  effect,
  untrack,
  onCleanup,
  ref,
  type ViewScope,
} from '@ship/sui-runtime'
import { ensureWasm, makeCompile } from '@ship/sui-viewer'
import type { Scope } from './page.sui'
import { TUTORIAL_LESSONS, type TutorialLesson } from '../../../src/tutorial-data'
import {
  bootTutorialApp,
  type CompileFn,
  type LessonCode,
  type TutorialHandle,
} from '../../../src/tutorial-boot'

const BY_SLUG: Record<string, TutorialLesson> = {}
for (const l of TUTORIAL_LESSONS) BY_SLUG[l.slug] = l

function toCode(l: TutorialLesson): LessonCode {
  return {
    slug: l.slug,
    starterTemplate: l.starterTemplate,
    starterController: l.starterController,
    solvedTemplate: l.solvedTemplate,
    solvedController: l.solvedController,
  }
}

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const lesson = derived(() => BY_SLUG[ctx.route.params.slug() ?? ''])
    const prevSlug = derived(() => lesson()?.prev ?? '')
    const nextSlug = derived(() => lesson()?.next ?? '')

    const rootEl = ref<HTMLElement>(null)
    const compile = signal<CompileFn | null>(null)
    ensureWasm('/sui_compiler_bg.wasm')
      .then(() => compile.set(() => makeCompile() as unknown as CompileFn))
      .catch((err) => console.error('[sui-site] wasm compile seam failed:', err))

    // Boot the editor↔preview wiring once the root element is mounted AND the
    // wasm compile seam is ready; (re-)load the matched lesson on slug change.
    let handle: TutorialHandle | null = null
    let lastSlug = ''
    effect(() => {
      const root = rootEl()
      const c = compile()
      const l = lesson()
      if (!root || !c) return
      untrack(() => {
        if (!handle) handle = bootTutorialApp(root, c)
        if (l && l.slug !== lastSlug) {
          lastSlug = l.slug
          handle.setLesson(toCode(l))
        }
      })
    })
    onCleanup(() => handle?.dispose())

    return {
      title: derived(() => lesson()?.title ?? 'Lesson not found'),
      description: derived(() => lesson()?.description ?? ''),
      proseHtml: derived(() => lesson()?.proseHtml ?? ''),
      rootEl,
      prevHref: derived(() => (prevSlug() ? '/tutorial/' + prevSlug() : '/tutorial')),
      nextHref: derived(() => (nextSlug() ? '/tutorial/' + nextSlug() : '/tutorial')),
      prevLabel: derived(() =>
        prevSlug() ? '← ' + (BY_SLUG[prevSlug()]?.title ?? '') : '← All lessons',
      ),
      nextLabel: derived(() =>
        nextSlug() ? (BY_SLUG[nextSlug()]?.title ?? '') + ' →' : 'All lessons →',
      ),
    }
  },
})
