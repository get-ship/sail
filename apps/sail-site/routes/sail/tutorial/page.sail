<!--
  /sail/tutorial — table of contents for the nine-lesson track.
-->

<scope>
  style page = { size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style eyebrow = { margin({ b: 2 }).fg(accent).type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { fg(muted-foreground).type({ size: lg, leading: relaxed }) }
  style lessonCard = { pad(4).radius(lg).border({ color: border, width: 1 }).bg(surface-1), when: { hover: bg(surface-2).border({ color: border-strong, width: 1 }) } }
  style lessonBody = { grow(1) }
  style lessonDesc = { fg(muted-foreground) }
  style footNav = { pad({ t: 4 }).margin({ t: 8 }).border({ color: border, width: 1, edge: t }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { TUTORIAL_GROUPS, TUTORIAL_LESSONS, type TutorialGroup } from '../../../src/compositions/tutorial-data'

export default controller((_ctx: ControllerScope<Scope>) => ({
  groups: (): TutorialGroup[] => TUTORIAL_GROUPS,
  total: (): number => TUTORIAL_LESSONS.length,
}))
</controller>

<column gap="5" style(page)>
  <p style(eyebrow)>Tutorial</p>
  <h1>Learn Sail</h1>
  <p style(lead)>
    Nine hands-on lessons — one per decision surface the language kept, then
    the loop it exists for: agents author, humans review. Each lesson pairs a
    starter to edit with a solved screen running live.
  </p>

  <for {groups() as group} key={group.chapter}>
    <column gap="3">
      <h2>{group.title}</h2>
      <column gap="2">
        <for {group.lessons as lesson} key={lesson.slug}>
          <a href={lesson.href} style(lessonCard)>
            <row gap="3" align="center">
              <badge tone="primary" variant="soft">{lesson.order}</badge>
              <column gap="1" style(lessonBody)>
                <strong>{lesson.title}</strong>
                <small style(lessonDesc)>{lesson.description}</small>
              </column>
            </row>
          </a>
        </for>
      </column>
    </column>
  </for>

  <row style(footNav)>
    <a variant="default" href="/docs/concepts/state">← State</a>
    <a variant="solid" tone="primary" href="/sail/tutorial/01-a-page-is-markup">Start lesson 1 →</a>
  </row>
</column>
