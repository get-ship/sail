<!--
  /sail/tutorial — table of contents for the nine-lesson track.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { TUTORIAL_GROUPS, TUTORIAL_LESSONS, type TutorialGroup } from '../../../src/compositions/tutorial-data'

export default controller((_ctx: ControllerScope<Scope>) => ({
  groups: (): TutorialGroup[] => TUTORIAL_GROUPS,
  total: (): number => TUTORIAL_LESSONS.length,
}))
</controller>

<column gap="5">
  <p>Tutorial</p>
  <h1>Learn Sail</h1>
  <p>
    Nine hands-on lessons — one per decision surface the language kept, then
    the loop it exists for: agents author, humans review. Each lesson pairs a
    starter to edit with a solved screen running live.
  </p>

  <for {groups() as group} key={group.chapter}>
    <column gap="3">
      <h2>{group.title}</h2>
      <column gap="2">
        <for {group.lessons as lesson} key={lesson.slug}>
          <a href={lesson.href}>
            <card interactive>
              <row gap="3" align="center">
                <badge tone="primary" variant="soft">{lesson.order}</badge>
                <column gap="1">
                  <strong>{lesson.title}</strong>
                  <small>{lesson.description}</small>
                </column>
              </row>
            </card>
          </a>
        </for>
      </column>
    </column>
  </for>

  <separator />
  <row gap="2">
    <a variant="default" href="/docs/concepts/state">← State</a>
    <a variant="solid" tone="primary" href="/sail/tutorial/01-a-page-is-markup">Start lesson 1 →</a>
  </row>
</column>
