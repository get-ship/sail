// <sui-language-showcase> — the "Language" surface mounted on /components.
//
// Five panels showcasing the compiler features that shipped with the language
// surface change (the deletion of sui-stack / sui-cluster, the introduction
// of <row>/<column>/<grid> layout keywords, the `sx` token utility, the
// `<theme>` channel contract, the `<variants>` prop→style table, and the
// motion directive + reduce-motion prefix).
//
// Each panel pairs a live rendered demo (authored inline in the .sui template)
// with its source — the source strings live here in the controller scope so
// the template stays a pure read of zero-arg signals (the compiler's lint
// rejects renderable expressions with arguments).
//
// Lives at the top of /components/page.sui, above <sui-component-catalog>.

import { controller, derived, type Derived } from '@ship/sui-runtime'

export interface LanguagePanel {
  /** Stable id — keyed reuse on `{#for}`. */
  id: string
  /** Short eyebrow ("Layout", "Styling", "Motion"). */
  eyebrow: string
  /** Display title. */
  title: string
  /** One-sentence framing of what the panel demonstrates. */
  lead: string
  /** Source string rendered into a <sui-codeblock lang="sui">. Already
   *  formatted for the syntax highlighter — leading whitespace trimmed. */
  source: string
  /** data-test id the runtime attaches to the panel card. */
  testId: string
  /** `part` value applied to the panel card. */
  partName: string
}

export interface ShowcaseScope {
  panels: Derived<LanguagePanel[]>
}

/** Tiny builder so each panel stays a single readable block. The `testId` and
 *  `partName` fields are derived from `id` once at module load — the template
 *  binds to the precomputed strings (zero-arg reads), not to `+`-concats the
 *  compiler rejects as statements-in-expressions. */
function panel(
  id: string,
  eyebrow: string,
  title: string,
  lead: string,
  source: string,
): LanguagePanel {
  return { id, eyebrow, title, lead, source, testId: 'lang-panel-' + id, partName: 'panel-' + id }
}

const PANELS: LanguagePanel[] = [
  panel('layout', 'Layout', '<row>, <column>, <grid>',
    'The deleted sui-stack / sui-cluster did not survive. Layout is now a ' +
    'first-class keyword surface that lowers to <div> + atomic token classes ' +
    'at build time.',
    [
      '<column gap="4" sx="p:5">',
      '  <row gap="3" justify="between" align="center">',
      '    <h3>Released in 0.3</h3>',
      '    <span tone="success" sx="bg:tone-bg border:tone-border fg:tone-fg">stable</span>',
      '  </row>',
      '  <p>Three CSS keywords, zero token literals.</p>',
      '</column>',
    ].join('\n')),

  panel('sx', 'Styling', 'sx utilities',
    'Token-only inline styling — every key is a design-system token, no ' +
    'hex / px / raw keywords. The compiler validates the keys against ' +
    'the closed sx vocabulary at build time.',
    [
      '<column gap="2" sx="p:5 bg:surface-2 radius:lg">',
      '  <strong sx="fg:foreground">Token-only inline styling</strong>',
      '  <span sx="fg:muted-foreground text:sm">',
      '    Never a hex, never a px — always a token.',
      '  </span>',
      '</column>',
    ].join('\n')),

  panel('theme', 'Styling', '<theme> channel contract',
    'A prop value fills the declared channels — the contract between the ' +
    'component authoring surface and the tokens that fill per tone. One ' +
    '<theme> block per prop; one rule per value (or `default:`).',
    [
      '<!-- Declared on the component file -->',
      '<theme prop="tone" channels="tone, tone-fg, tone-border">',
      '  default:       primary, foreground,   border',
      '  tone=success:  success,  foreground,   success-border',
      '  tone=warning:  warning,  tone-fg,      warning-border',
      '  tone=destructive: destructive, tone-fg, destructive-border',
      '</theme>',
      '',
      '<!-- Consumed by the component: bg:var(--tone), color:var(--tone-fg) -->',
      '<sui-card tone="warning">',
      '  <sui-icon slot="icon" name="alert-triangle"></sui-icon>',
      '  <span slot="title">Theme channels, in action</span>',
      '  The card surface reads <code>var(--tone)</code>.',
      '</sui-card>',
    ].join('\n')),

  panel('variants', 'Styling', '<variants> prop→style table',
    'The declarative prop→style table. Each line is a `<selector>: <sx-list>`; ' +
      'the compiler lowers it to `:host([prop=value]) { … }` CSS. The ' +
      '`part="…"` variant targets a shadow part, not the host.',
    [
      '<!-- Variant table; the tag may be a bare `<variants>` (host) or ',
      '     `<variants part="…">` (a shadow part). Each line lowers to a ',
      '     `:host([attr]) { … }` rule. -->',
      '<variants>',
      '  tone=success:    bg:success-bg fg:success',
      '  tone=destructive:bg:destructive-bg fg:destructive',
      '  [disabled]:      opacity:50 cursor:not-allowed',
      '</variants>',
      '',
      '<button>',
      '  <span tone="success" sx="bg:success-bg fg:success">Passing</span>',
      '  <span tone="destructive" sx="bg:destructive-bg fg:destructive">Failing</span>',
      '</button>',
    ].join('\n')),

  panel('motion', 'Motion', 'motion: directive + reduce-motion',
    'The `motion:` directive on a shadow part selects a spring/tween preset ' +
    'at build time; the `motion-reduce:` prefix in `sx` gates the style on ' +
    'the OS reduce-motion preference. Both are compiler features — no ' +
    'runtime motion calculator, no inline keyframes.',
    [
      '<!-- motion: applies the named preset to the shadow part;',
      '     the spring preset handles the runtime motion calculus. -->',
      '<button',
      '  motion:hover={{ scale: \'spring:snappy\' }}',
      '  motion:press={{ scale: \'spring:bouncy\' }}',
      '>',
      '  Hover me',
      '</button>',
      '',
      '<!-- The motion-reduce: sx prefix lowers into a media query -->',
      '<div sx="p:4 motion-reduce:opacity:50">',
      '  Pause me when the OS says to.',
      '</div>',
    ].join('\n')),
]

export default controller({
  scope: (): ShowcaseScope => {
    // The list is constant — derive it so the template reads it through the
    // read-only signal convention the lint enforces, identical to the
    // catalog's `chips`/`components` pattern. No `.set` is ever issued.
    return {
      panels: derived<LanguagePanel[]>(() => PANELS),
    }
  },
})
