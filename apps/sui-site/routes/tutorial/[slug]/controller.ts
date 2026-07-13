// /tutorial/[slug] — one lesson: the prose, the SOLVED lesson mounted live,
// and both sources. route.params.slug() picks the lesson out of the
// generated data; the nine solved components are referenced STATICALLY by
// tag in page.sui's {#match}, so the per-view resolver bundles exactly them.
//
// Every read the template does is a zero-arg call, because that is the only
// call shape a renderable expression allows.
import { controller, derived, signal, bindHtml, type Action, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { TUTORIAL_LESSONS, lessonBySlug, type TutorialLesson } from '../../../src/tutorial-data'

// The prose is TRUSTED, pre-rendered HTML: gen-tutorial.mjs converts the
// lesson markdown at build time; nothing user-supplied flows through here.
// An action (it touches the DOM, so it is an action) delegates the write to
// bindHtml — the runtime's one HTML-blob seam.
const prose: Action<string> = (node, html) => {
  const content = signal(html ?? '')
  const dispose = bindHtml(node, () => content())
  return {
    update(next: string): void {
      content.set(next ?? '')
    },
    destroy: dispose,
  }
}

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const slug = derived(() => ctx.route.params.slug?.() ?? '')
    const lesson = derived<TutorialLesson | null>(() => lessonBySlug(slug()))
    const known = derived(() => lesson() !== null)

    const title = (): string => lesson()?.title ?? ''
    const description = (): string => lesson()?.description ?? ''
    const proseHtml = (): string => lesson()?.proseHtml ?? ''
    const starterTemplate = (): string => lesson()?.starterTemplate ?? ''
    const starterController = (): string => lesson()?.starterController ?? ''
    const solvedTemplate = (): string => lesson()?.solvedTemplate ?? ''
    const solvedController = (): string => lesson()?.solvedController ?? ''
    const hasStarterController = (): boolean => (lesson()?.starterController ?? '') !== ''
    const hasSolvedController = (): boolean => (lesson()?.solvedController ?? '') !== ''
    const starterStem = (): string => lesson()?.starterStem ?? ''
    const solvedStem = (): string => lesson()?.solvedStem ?? ''

    const orderLabel = (): string => `${lesson()?.order ?? 0} / ${TUTORIAL_LESSONS.length}`

    const prevLesson = derived<TutorialLesson | null>(() => {
      const p = lesson()?.prev
      return p ? lessonBySlug(p) : null
    })
    const nextLesson = derived<TutorialLesson | null>(() => {
      const n = lesson()?.next
      return n ? lessonBySlug(n) : null
    })
    const hasPrev = derived(() => prevLesson() !== null)
    const hasNext = derived(() => nextLesson() !== null)
    const prevHref = (): string => `/tutorial/${prevLesson()?.slug ?? ''}`
    const nextHref = (): string => `/tutorial/${nextLesson()?.slug ?? ''}`
    const prevTitle = (): string => prevLesson()?.title ?? ''
    const nextTitle = (): string => nextLesson()?.title ?? ''

    return {
      slug,
      known,
      title,
      description,
      proseHtml,
      starterTemplate,
      starterController,
      solvedTemplate,
      solvedController,
      hasStarterController,
      hasSolvedController,
      starterStem,
      solvedStem,
      orderLabel,
      hasPrev,
      hasNext,
      prevHref,
      nextHref,
      prevTitle,
      nextTitle,
      prose,
    }
  },
})
