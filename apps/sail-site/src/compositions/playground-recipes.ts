// Free-playground recipe catalog. Blank is always first; remaining entries
// seed from tutorial solved sources so demos stay in lockstep with lessons
// without duplicating large bodies.

import { TUTORIAL_LESSONS } from './tutorial-data'

export interface PlaygroundRecipe {
  id: string
  title: string
  description: string
  /** Full `.sail` source loaded into the editor. */
  source: string
}

/** Minimal valid page — always available as the default recipe. */
export const BLANK_SOURCE = `<!--
  Blank playground — a single column and some text.
  Edit freely, then Run (or wait for auto-compile).
-->
<scope>
  style muted = { fg(muted-foreground).type({ size: sm }) }
</scope>


<column gap="3">
  <h2>Hello Sail</h2>
  <p style(muted)>Edit this source and hit Run. Pick a recipe above for a fuller example.</p>
  <button variant="solid" tone="primary">Primary button</button>
</column>
`

export const PLAYGROUND_RECIPES: PlaygroundRecipe[] = [
  {
    id: 'blank',
    title: 'Blank',
    description: 'Minimal valid page — column, text, one button.',
    source: BLANK_SOURCE,
  },
  ...TUTORIAL_LESSONS.map((l) => ({
    id: l.slug,
    title: `${String(l.order).padStart(2, '0')}. ${l.title}`,
    description: l.description,
    // Solved sources are complete demos; starter would leave TODOs empty.
    source: l.solvedSource,
  })),
]

const BY_ID: Record<string, PlaygroundRecipe> = {}
for (const r of PLAYGROUND_RECIPES) BY_ID[r.id] = r

export function recipeById(id: string): PlaygroundRecipe | undefined {
  return BY_ID[id]
}

export const DEFAULT_RECIPE_ID = 'blank'
