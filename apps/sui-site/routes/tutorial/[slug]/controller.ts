// Dynamic tutorial lesson page — an INTERACTIVE, side-by-side experience
// (svelte.dev/tutorial-style), ported from ship-marketing's proven TutorialApp.
// Prose renders above two panes: LEFT an editable template.sui / controller.ts
// code area using <sui-code-editor>; RIGHT a sandboxed <iframe src="/tut-host.html">
// that compiles + RUNS the lesson live in full isolation.
//
// The slug comes from the route param (`ctx.route.params.slug()`). All
// editor↔preview wiring (debounce, regex TS→JS, postMessage, theme sync)
// lives in the imported `tutorial-boot` module — kept OUT of this controller
// body so the compiler's scope-scanner never trips on lesson code strings or
// the `signal(…)` / `derived(…)` text snippets inside.
import {
  controller,
  derived,
  effect,
  ref,
  type ViewScope,
} from '@ship/sui-runtime'
import { ensureWasm, makeCompile } from '@ship/sui-viewer'
import type { Scope } from './page.sui'
import { TUTORIAL_LESSONS, type TutorialLesson } from '../../../src/tutorial-data'
import { bootTutorialApp, type CompileFn, type TutorialHandle } from '../../../src/tutorial-boot'

const BY_SLUG: Record<string, TutorialLesson> = {}
for (const l of TUTORIAL_LESSONS) BY_SLUG[l.slug] = l

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const lesson = derived(() => BY_SLUG[ctx.route.params.slug() ?? ''])
    const prevSlug = derived(() => lesson()?.prev ?? '')
    const nextSlug = derived(() => lesson()?.next ?? '')

    const rootEl = ref<HTMLElement>(null)
    let compile: CompileFn | null = null
    ensureWasm('/sui_compiler_bg.wasm')
      .then(() => { compile = makeCompile() as unknown as CompileFn })
      .catch((err) => console.error('[sui-site] wasm compile seam failed:', err))

    let handle: TutorialHandle | null = null
    let lastSlug = ''
    effect(() => {
      const root = rootEl()
      const l = lesson()
      if (!root || !compile) return
      if (!handle) handle = bootTutorialApp(root, compile)
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
