/**
 * Inventory. A row here is a product slot — authored or planned.
 * Docs composition and takumi read this. Do not invent a second list.
 *
 * kind: 'template' — a page under routes/<id>/page.sail
 * kind: 'block'    — a named composition at src/blocks/<id>/<id>.sail
 */

export type CatalogKind = 'block' | 'template'
export type BlockStatus = 'authored' | 'planned'

export type BlockEntry = {
  id: string
  title: string
  intent: string
  kind: CatalogKind
  status: BlockStatus
  /** L1 family hosts this row is meant to compose. */
  primitives: readonly string[]
  /** Required when kind is 'block'. Head-to-head with ui.mantine.dev. */
  mantineUrl?: string
}

export const BLOCKS: readonly BlockEntry[] = [
  {
    id: 'dashboard',
    title: 'Dashboard',
    intent: 'App chrome + metrics + a tabbed data surface. Defaults only.',
    kind: 'template',
    status: 'authored',
    primitives: [
      'navbar',
      'dialog',
      'card',
      'table',
      'field',
      'input',
      'select',
      'date',
      'textarea',
      'button',
      'badge',
      'alert',
      'pagination',
      'theme-picker',
      'breadcrumb',
      'avatar',
      'tabs',
      'combobox',
      'chip',
      'dropdown',
      'hover-card',
      'kbd',
      'meter',
      'empty-state',
      'toast',
      'drawer',
    ],
  },
  {
    id: 'pricing',
    title: 'Pricing',
    intent: 'Tailwind Plus marketing page: three tiers, comparison, FAQ.',
    kind: 'template',
    status: 'authored',
    primitives: [
      'navbar',
      'card',
      'button',
      'badge',
      'separator',
      'icon',
      'toggle-group',
      'accordion',
      'a',
      'alert',
      'table',
      'avatar',
      'theme-picker',
      'scroll-area',
    ],
  },
  {
    id: 'settings',
    title: 'Settings',
    intent: 'Fieldset groups, fields, switches. Form defaults must hold.',
    kind: 'template',
    status: 'authored',
    primitives: [
      'navbar',
      'fieldset',
      'field',
      'input',
      'select',
      'textarea',
      'switch',
      'radio-group',
      'button',
      'tabs',
      'breadcrumb',
      'avatar',
      'file',
      'checkbox',
      'chip',
      'table',
      'badge',
      'separator',
      'theme-picker',
    ],
  },
  {
    id: 'auth',
    title: 'Auth',
    intent: 'A single card: identity fields + primary action.',
    kind: 'template',
    status: 'planned',
    primitives: ['card', 'field', 'input', 'button', 'a'],
  },
  {
    id: 'empty',
    title: 'Empty',
    intent: 'Empty-state as the page, not a decorated div.',
    kind: 'template',
    status: 'planned',
    primitives: ['empty-state', 'button'],
  },
  {
    id: 'HeroTitle',
    title: 'Hero title',
    intent: 'Display heading, lead, and a pair of actions.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/hero-title/',
    primitives: ['button', 'a'],
  },
  {
    id: 'HeroText',
    title: 'Hero text',
    intent: 'Centered lead block with a single action.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/hero-text/',
    primitives: ['button'],
  },
  {
    id: 'FeaturesCards',
    title: 'Features cards',
    intent: 'Three feature cards with an icon and a short blurb.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/features-cards/',
    primitives: ['card', 'icon'],
  },
  {
    id: 'FaqSimple',
    title: 'FAQ simple',
    intent: 'A short accordion of questions.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/faq-simple/',
    primitives: ['accordion'],
  },
  {
    id: 'GetInTouchSimple',
    title: 'Get in touch',
    intent: 'Compact contact fields and a send action.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/get-in-touch-simple/',
    primitives: ['field', 'input', 'textarea', 'button'],
  },
  {
    id: 'AuthenticationTitle',
    title: 'Authentication title',
    intent: 'Identity fields under a title, one primary action.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/authentication-title/',
    primitives: ['card', 'field', 'input', 'button'],
  },
  {
    id: 'HeaderSimple',
    title: 'Header simple',
    intent: 'Brand, a few links, one action. Not a page.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/header-simple/',
    primitives: ['navbar', 'a', 'button'],
  },
  {
    id: 'NavbarMinimal',
    title: 'Navbar minimal',
    intent: 'Icon rail. Not app chrome for a whole product.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/navbar-minimal/',
    primitives: ['navbar', 'button'],
  },
  {
    id: 'FooterLinks',
    title: 'Footer links',
    intent: 'Link columns and a short mark.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/footer-links/',
    primitives: ['a'],
  },
  {
    id: 'StatsGroup',
    title: 'Stats group',
    intent: 'A row of named figures.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/stats-group/',
    primitives: ['card'],
  },
  {
    id: 'CardWithStats',
    title: 'Card with stats',
    intent: 'One card, a few figures.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/card-with-stats/',
    primitives: ['card'],
  },
  {
    id: 'TableSort',
    title: 'Table sort',
    intent: 'One data table. Not the table engine.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/table-reviews/',
    primitives: ['table'],
  },
  {
    id: 'UserButton',
    title: 'User button',
    intent: 'Avatar, name, email, as one control.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/user-button/',
    primitives: ['button', 'avatar'],
  },
  {
    id: 'InputWithButton',
    title: 'Input with button',
    intent: 'Search field + submit. Defaults only.',
    kind: 'block',
    status: 'authored',
    mantineUrl: 'https://ui.mantine.dev/component/input-with-button/',
    primitives: ['field', 'input', 'button'],
  },
  {
    id: 'ContainedInputs',
    title: 'Contained inputs',
    intent: 'A tight stack of labeled fields.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/contained-inputs/',
    primitives: ['field', 'input'],
  },
  {
    id: 'CommentSimple',
    title: 'Comment simple',
    intent: 'Avatar, author, body.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/comment-simple/',
    primitives: ['avatar'],
  },
  {
    id: 'CookiesBanner',
    title: 'Cookies banner',
    intent: 'A short notice and two actions.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/cookies-banner/',
    primitives: ['alert', 'button'],
  },
  {
    id: 'NothingFound',
    title: 'Nothing found',
    intent: 'Empty as the composition, not a decorated div.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/nothing-found-background/',
    primitives: ['empty-state', 'button'],
  },
  {
    id: 'LeadGrid',
    title: 'Lead grid',
    intent: 'One bento/grid of surfaces.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/lead-grid/',
    primitives: ['card'],
  },
  {
    id: 'ImageActionBanner',
    title: 'Image action banner',
    intent: 'A banner with a title and one action.',
    kind: 'block',
    status: 'planned',
    mantineUrl: 'https://ui.mantine.dev/component/image-action-banner/',
    primitives: ['card', 'button'],
  },
] as const

export function authoredBlocks(): BlockEntry[] {
  return BLOCKS.filter((b) => b.status === 'authored')
}

export function templates(): BlockEntry[] {
  return BLOCKS.filter((b) => b.kind === 'template')
}

export function blocks(): BlockEntry[] {
  return BLOCKS.filter((b) => b.kind === 'block')
}

/** Lab canvas path for a block id. Templates live at /<id>. */
export function blockLabPath(id: string): string {
  return `/lab/${id}`
}
