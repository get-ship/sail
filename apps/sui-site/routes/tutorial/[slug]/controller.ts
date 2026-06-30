// Dynamic tutorial lesson page. The slug comes from the route param
// (`ctx.route.params.slug()` — a signal), so every binding below is a `derived`
// over it: change the URL and the whole page re-renders against the matched
// lesson. All lesson DATA (including the code strings) lives in the imported
// `tutorial-data` module — keeping code strings OUT of this controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` text.
import { controller, derived, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { TUTORIAL_LESSONS, type TutorialLesson } from '../../../src/tutorial-data'

const BY_SLUG: Record<string, TutorialLesson> = {}
for (const l of TUTORIAL_LESSONS) BY_SLUG[l.slug] = l

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const lesson = derived(() => BY_SLUG[ctx.route.params.slug() ?? ''])
    const prevSlug = derived(() => lesson()?.prev ?? '')
    const nextSlug = derived(() => lesson()?.next ?? '')
    return {
      title: derived(() => lesson()?.title ?? 'Lesson not found'),
      description: derived(() => lesson()?.description ?? ''),
      proseHtml: derived(() => lesson()?.proseHtml ?? ''),
      starterTemplate: derived(() => lesson()?.starterTemplate ?? ''),
      starterController: derived(() => lesson()?.starterController ?? ''),
      solvedTemplate: derived(() => lesson()?.solvedTemplate ?? ''),
      solvedController: derived(() => lesson()?.solvedController ?? ''),
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
