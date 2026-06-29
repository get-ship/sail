// The app's LOCAL data client — the `clients` seam, with no ship product,
// no daemon, no SDK. `deck.jsonc#entry` names this module + the `data` export;
// deck's generated `entry.gen.ts` imports it and hands it to
// `mountRouter({ clients: { data } })`, so every view reads it as
// `ctx.clients.data`.
//
// State lives in a runtime `signal`, so the SAME instance shared across views
// is reactive: the kitchen's "add recipe" form mutates `recipes`, and the
// recipes list (a different route) re-renders because it reads the signal.

import { signal, type Signal } from '@ship/sui-runtime'

/** One recipe in the box. */
export interface Recipe {
  id: string
  title: string
  /** Active cooking time, in minutes. */
  minutes: number
  /** Short, human flavor label (cuisine / vibe). */
  cuisine: string
  /** Free-form tags for the detail page. */
  tags: string[]
  /** Ordered prep steps. */
  steps: string[]
}

/** The shape views consume via `ctx.clients.data`. */
export interface DataClient {
  /** Reactive list of every recipe. Read it (`recipes()`) inside a template
   *  or controller to subscribe; it updates when `add()` runs. */
  recipes: Signal<Recipe[]>
  /** Look up one recipe by id (untracked point read). */
  get(id: string | undefined): Recipe | undefined
  /** Append a recipe built from a title + minutes; returns the new row. */
  add(input: { title: string; minutes: number }): Recipe
}

const SEED: Recipe[] = [
  {
    id: 'shakshuka',
    title: 'Skillet Shakshuka',
    minutes: 30,
    cuisine: 'North African',
    tags: ['brunch', 'one-pan', 'vegetarian'],
    steps: [
      'Sweat onion, pepper, and garlic in olive oil until soft.',
      'Stir in cumin, paprika, and crushed tomatoes; simmer 10 minutes.',
      'Make wells, crack in eggs, cover, and poach to taste.',
      'Finish with feta, herbs, and warm bread.',
    ],
  },
  {
    id: 'miso-ramen',
    title: 'Weeknight Miso Ramen',
    minutes: 25,
    cuisine: 'Japanese',
    tags: ['soup', 'umami', 'fast'],
    steps: [
      'Bloom ginger and garlic in sesame oil.',
      'Whisk in miso and stock; bring to a gentle simmer.',
      'Cook noodles separately, then combine.',
      'Top with corn, scallion, soft egg, and chili oil.',
    ],
  },
  {
    id: 'lemon-pasta',
    title: 'Lemon Butter Pasta',
    minutes: 20,
    cuisine: 'Italian',
    tags: ['pantry', 'fast', 'vegetarian'],
    steps: [
      'Boil pasta in well-salted water; reserve a mug of starchy water.',
      'Melt butter with lemon zest and a splash of pasta water.',
      'Toss pasta through the emulsion off heat.',
      'Add parmesan, lemon juice, and black pepper; loosen as needed.',
    ],
  },
  {
    id: 'sheet-tofu',
    title: 'Crispy Sheet-Pan Tofu',
    minutes: 35,
    cuisine: 'Sheet-pan',
    tags: ['roasted', 'meal-prep', 'vegan'],
    steps: [
      'Press and cube tofu; toss with cornstarch, soy, and oil.',
      'Roast at 220°C, flipping once, until edges crisp.',
      'Roast broccoli on the same tray.',
      'Glaze with maple-soy and toasted sesame.',
    ],
  },
]

/** Turn a title into a stable, URL-safe id. */
function slugify(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

function createDataClient(): DataClient {
  const recipes = signal<Recipe[]>(SEED)
  return {
    recipes,
    get: (id) => (id ? recipes().find((r) => r.id === id) : undefined),
    add: ({ title, minutes }) => {
      const base = slugify(title) || 'recipe'
      // De-dupe ids so two "Soup" entries don't collide.
      const existing = new Set(recipes().map((r) => r.id))
      let id = base
      let n = 2
      while (existing.has(id)) id = `${base}-${n++}`
      const recipe: Recipe = {
        id,
        title: title.trim(),
        minutes,
        cuisine: 'Yours',
        tags: ['new'],
        steps: ['Add your steps from the recipe detail page.'],
      }
      recipes.set([...recipes(), recipe])
      return recipe
    },
  }
}

/** The named export `deck.jsonc#entry.exposeNames` hands to `mountRouter`. */
export const data: DataClient = createDataClient()

// Teach the runtime's `Clients` map that `data` exists, so every view's
// `ctx.clients.data` is typed (the interface is host-augmentable by design).
declare module '@ship/sui-runtime' {
  interface Clients {
    data: DataClient
  }
}
