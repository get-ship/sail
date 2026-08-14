<!-- Getting started — page file, controller, sail dev. -->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { pageExample, controllerExample, devCmd } from '../../src/compositions/deck-getting-started-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ pageExample, controllerExample, devCmd }))
</controller>

<scope>
  style page = { maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { mb(2).fg(accent).fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { fg(muted-foreground).fontSize(lg).leading(relaxed) }
  style callout = { padY(3).padX(4).borderWidth(1).border(border).borderLWidth(3).borderL(primary).radius(md).bg(surface-1).fontSize(sm) }
  style calloutStrong = { fg(foreground) }
  style footNav = { mt(8).padT(4).borderTWidth(1).borderT(border) }
</scope>

<article style(page)>
  <p style(eyebrow)>Introduction</p>
  <h1>Getting started</h1>
  <p style(lead)>You need the <code>sail</code> CLI and a folder. No project scaffold, no install, no config to start.</p>

  <h2>1. A page is a file</h2>
  <p>Create <code>routes/page.sail</code>. Whatever markup it holds is your page.</p>
  <code-block code={pageExample} lang="sail"></code-block>

  <h2>2. Add a controller</h2>
  <p>Co-locate <code>routes/controller.ts</code> for reactive state. It returns a scope the template reads by name.</p>
  <code-block code={controllerExample} lang="sail"></code-block>

  <h2>3. Run the dev server</h2>
  <code-block code={devCmd} lang="bash"></code-block>
  <div style(callout)>
    <strong style(calloutStrong)>No build step to develop.</strong> deck compiles <code>.sail</code> in-process and serves the SPA with HMR — edit a route and it soft-reloads with state preserved.
  </div>

  <row justify="between" style(footNav)>
    <a href="/">← Overview</a>
    <a href="/routing">File routing →</a>
  </row>
</article>
