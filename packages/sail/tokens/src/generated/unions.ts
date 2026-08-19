/**
 * GENERATED FILE — do not edit by hand.
 *
 * Closed-set TypeScript unions derived from the DTCG model + token registry.
 * Source of truth: packages/sail/tokens/model/** (groups + themes).
 * Regenerate: cargo run -p sail-compiler -- tokens
 *            (or: pnpm --filter @ship/tokens generate)
 *
 * L1 / app props import these instead of hand-maintaining literal unions:
 *   import type { ToneName, CssTokenName, ThemeId } from '@ship/tokens'
 *   // or: import type { ToneName } from '@ship/tokens/unions'
 */

/** Where {@link TONE_NAMES} membership is sourced from. */
export const TONE_NAMES_SOURCE = 'dtcg-groups:color.{primary,info,success,warning,danger,accent,muted}' as const

/** Semantic paint tone groups (Style bound `tone` / host prop). No `'default'`. */
export const TONE_NAMES = [
  'primary',
  'info',
  'success',
  'warning',
  'danger',
  'accent',
  'muted',
] as const
export type Tone = (typeof TONE_NAMES)[number]

export type ToneName = Tone

/** Tone group → members → public token name (Style / tokens.css). */
export const TONE = {
  primary: {
    default: 'primary',
    foreground: 'primary-foreground',
    hover: 'primary-hover',
  },
  info: {
    default: 'info',
    foreground: 'info-foreground',
    hover: 'info-hover',
  },
  success: {
    default: 'success',
    foreground: 'success-foreground',
    hover: 'success-hover',
  },
  warning: {
    default: 'warning',
    foreground: 'warning-foreground',
    hover: 'warning-hover',
  },
  danger: {
    default: 'danger',
    foreground: 'danger-foreground',
    hover: 'danger-hover',
  },
  accent: {
    default: 'accent',
    foreground: 'accent-foreground',
    hover: 'accent-hover',
  },
  muted: {
    default: 'muted',
    foreground: 'muted-foreground',
  },
} as const
export type ToneGroup = keyof typeof TONE
export type ToneMember<G extends ToneGroup = ToneGroup> = keyof (typeof TONE)[G]

/** Theme catalog ids from packages/sail/tokens/model/themes/*.json. */
export const THEME_IDS = [
  'sail',
  'paper',
  'halo',
  'console',
  'glass',
  'brut',
  '8bit',
  'synthwave',
] as const
export type ThemeId = (typeof THEME_IDS)[number]

/** Control height scale steps (suffix of control-h-*). */
export const CONTROL_HEIGHT_STEPS = [
  '2xs',
  'xs',
  'sm',
  'md',
  'lg',
  'xl',
] as const
export type ControlHeight = (typeof CONTROL_HEIGHT_STEPS)[number]

/** Control height token cssNames (control-h-*). */
export const CONTROL_HEIGHT_TOKENS = [
  'control-h-2xs',
  'control-h-xs',
  'control-h-sm',
  'control-h-md',
  'control-h-lg',
  'control-h-xl',
] as const
export type ControlHeightToken = (typeof CONTROL_HEIGHT_TOKENS)[number]

/** Typography size scale steps (suffix of text-*). */
export const TEXT_SIZE_STEPS = [
  'xs',
  'sm',
  'md',
  'base',
  'lg',
  'xl',
  '2xl',
  '3xl',
  '4xl',
  'display',
] as const
export type TextSize = (typeof TEXT_SIZE_STEPS)[number]

/** Typography size token cssNames (text-*). */
export const TEXT_SIZE_TOKENS = [
  'text-xs',
  'text-sm',
  'text-md',
  'text-base',
  'text-lg',
  'text-xl',
  'text-2xl',
  'text-3xl',
  'text-4xl',
  'text-display',
] as const
export type TextSizeToken = (typeof TEXT_SIZE_TOKENS)[number]

/** Spacing scale token cssNames (space-*). */
export const SPACE_TOKENS = [
  'space-1',
  'space-10',
  'space-11',
  'space-12',
  'space-14',
  'space-16',
  'space-1_5',
  'space-2',
  'space-20',
  'space-24',
  'space-28',
  'space-2_5',
  'space-3',
  'space-32',
  'space-36',
  'space-4',
  'space-40',
  'space-44',
  'space-48',
  'space-5',
  'space-52',
  'space-56',
  'space-6',
  'space-60',
  'space-64',
  'space-7',
  'space-8',
  'space-9',
  'space-half',
  'space-px',
] as const
export type SpaceToken = (typeof SPACE_TOKENS)[number]

/** Radius scale token cssNames. */
export const RADIUS_TOKENS = [
  'radius-full',
  'radius-lg',
  'radius-md',
  'radius-sm',
  'radius-xl',
] as const
export type RadiusToken = (typeof RADIUS_TOKENS)[number]

/** Motion duration token cssNames (duration-*). */
export const DURATION_TOKENS = [
  'duration-deliberate',
  'duration-fast',
  'duration-instant',
  'duration-medium',
  'duration-slow',
] as const
export type DurationToken = (typeof DURATION_TOKENS)[number]

/** Brand / semantic surface paint tokens under color.* (excl. tone ramps & product accents). */
export const SEMANTIC_COLOR_TOKENS = [
  'accent',
  'accent-border',
  'accent-foreground',
  'accent-hover',
  'accent-subtle',
  'background',
  'danger',
  'danger-border',
  'danger-foreground',
  'danger-hover',
  'danger-subtle',
  'foreground',
  'info',
  'info-border',
  'info-foreground',
  'info-hover',
  'info-subtle',
  'inset',
  'inverse',
  'muted',
  'muted-foreground',
  'neutral',
  'neutral-border',
  'neutral-foreground',
  'neutral-hover',
  'neutral-subtle',
  'overlay',
  'primary',
  'primary-border',
  'primary-foreground',
  'primary-hover',
  'primary-subtle',
  'scrim',
  'secondary',
  'secondary-border',
  'secondary-foreground',
  'secondary-hover',
  'secondary-subtle',
  'success',
  'success-border',
  'success-foreground',
  'success-hover',
  'success-subtle',
  'surface',
  'warning',
  'warning-border',
  'warning-foreground',
  'warning-hover',
  'warning-subtle',
] as const
export type SemanticColorName = (typeof SEMANTIC_COLOR_TOKENS)[number]

/** Every color-typed token cssName in the registry (paint tokens). */
export type ColorTokenName =
  | 'accent'
  | 'accent-border'
  | 'accent-foreground'
  | 'accent-hover'
  | 'accent-subtle'
  | 'active-bg'
  | 'background'
  | 'bg-app-color'
  | 'bg-inset-color'
  | 'bg-inverse-color'
  | 'bg-overlay-color'
  | 'bg-subtle-color'
  | 'bg-surface-color'
  | 'border'
  | 'border-default-color'
  | 'border-none-color'
  | 'border-strong'
  | 'border-strong-color'
  | 'border-subtle'
  | 'border-subtle-color'
  | 'danger'
  | 'danger-border'
  | 'danger-foreground'
  | 'danger-hover'
  | 'danger-subtle'
  | 'fg-foreground-color'
  | 'fg-muted-color'
  | 'foreground'
  | 'hover-bg'
  | 'info'
  | 'info-border'
  | 'info-foreground'
  | 'info-hover'
  | 'info-subtle'
  | 'inset'
  | 'inverse'
  | 'muted'
  | 'muted-foreground'
  | 'neutral'
  | 'neutral-border'
  | 'neutral-foreground'
  | 'neutral-hover'
  | 'neutral-subtle'
  | 'overlay'
  | 'primary'
  | 'primary-border'
  | 'primary-foreground'
  | 'primary-hover'
  | 'primary-subtle'
  | 'ring'
  | 'ring-color'
  | 'scrim'
  | 'scrollbar-thumb'
  | 'scrollbar-thumb-hover'
  | 'secondary'
  | 'secondary-border'
  | 'secondary-foreground'
  | 'secondary-hover'
  | 'secondary-subtle'
  | 'selected-bg'
  | 'selected-border'
  | 'success'
  | 'success-border'
  | 'success-foreground'
  | 'success-hover'
  | 'success-subtle'
  | 'surface'
  | 'syntax-attr'
  | 'syntax-comment'
  | 'syntax-function'
  | 'syntax-keyword'
  | 'syntax-number'
  | 'syntax-string'
  | 'syntax-tag'
  | 'syntax-type'
  | 'syntax-variable'
  | 'warning'
  | 'warning-border'
  | 'warning-foreground'
  | 'warning-hover'
  | 'warning-subtle'

/** Every public token cssName in the registry (without the -- prefix). */
export type CssTokenName =
  | 'accent'
  | 'accent-border'
  | 'accent-foreground'
  | 'accent-hover'
  | 'accent-subtle'
  | 'active-bg'
  | 'active-strength'
  | 'background'
  | 'bg-app-color'
  | 'bg-inset-color'
  | 'bg-inverse-color'
  | 'bg-overlay-color'
  | 'bg-subtle-color'
  | 'bg-surface-color'
  | 'border'
  | 'border-default-color'
  | 'border-default-style'
  | 'border-default-width'
  | 'border-none-color'
  | 'border-none-style'
  | 'border-none-width'
  | 'border-strong'
  | 'border-strong-color'
  | 'border-strong-style'
  | 'border-strong-width'
  | 'border-subtle'
  | 'border-subtle-color'
  | 'border-subtle-style'
  | 'border-subtle-width'
  | 'border-width'
  | 'breakpoint-desktop'
  | 'breakpoint-mobile'
  | 'breakpoint-tablet'
  | 'control-frame'
  | 'control-h-2xs'
  | 'control-h-lg'
  | 'control-h-md'
  | 'control-h-sm'
  | 'control-h-xl'
  | 'control-h-xs'
  | 'control-lift'
  | 'control-stroke'
  | 'danger'
  | 'danger-border'
  | 'danger-foreground'
  | 'danger-hover'
  | 'danger-subtle'
  | 'density-icon'
  | 'density-row-gap'
  | 'density-row-px'
  | 'density-row-py'
  | 'density-text'
  | 'duration-deliberate'
  | 'duration-fast'
  | 'duration-instant'
  | 'duration-medium'
  | 'duration-slow'
  | 'ease-in'
  | 'ease-in-out'
  | 'ease-out'
  | 'ease-out-soft'
  | 'ease-spring'
  | 'elevation-lg-shadow'
  | 'elevation-md-shadow'
  | 'elevation-none-shadow'
  | 'elevation-sm-shadow'
  | 'elevation-xl-shadow'
  | 'elevation-xs-shadow'
  | 'fg-foreground-color'
  | 'fg-muted-color'
  | 'filter-backdrop'
  | 'font-body'
  | 'font-display'
  | 'font-heading'
  | 'font-mono'
  | 'font-weight-bold'
  | 'font-weight-medium'
  | 'font-weight-normal'
  | 'font-weight-semibold'
  | 'foreground'
  | 'glow-accent'
  | 'glow-primary'
  | 'gradient-accent'
  | 'gradient-primary'
  | 'gradient-surface'
  | 'hover-bg'
  | 'hover-strength'
  | 'icon-size-2xl'
  | 'icon-size-2xs'
  | 'icon-size-3xl'
  | 'icon-size-3xs'
  | 'icon-size-lg'
  | 'icon-size-md'
  | 'icon-size-sm'
  | 'icon-size-xl'
  | 'icon-size-xs'
  | 'image-texture'
  | 'image-texture-blend'
  | 'image-texture-opacity'
  | 'info'
  | 'info-border'
  | 'info-foreground'
  | 'info-hover'
  | 'info-subtle'
  | 'inset'
  | 'inverse'
  | 'leading-none'
  | 'leading-normal'
  | 'leading-relaxed'
  | 'leading-snug'
  | 'leading-tight'
  | 'mix-bg'
  | 'mix-border'
  | 'mix-hover'
  | 'muted'
  | 'muted-foreground'
  | 'neutral'
  | 'neutral-border'
  | 'neutral-foreground'
  | 'neutral-hover'
  | 'neutral-subtle'
  | 'overlay'
  | 'overlay-blend'
  | 'overlay-fill'
  | 'overlay-max-h-lg'
  | 'overlay-max-h-md'
  | 'overlay-max-h-sm'
  | 'overlay-max-w-lg'
  | 'overlay-max-w-md'
  | 'overlay-max-w-sm'
  | 'overlay-opacity'
  | 'overlay-vh-lg'
  | 'overlay-vh-md'
  | 'overlay-vh-sm'
  | 'pad-page-x'
  | 'play-colors-property'
  | 'play-control-property'
  | 'play-surface-property'
  | 'press-scale'
  | 'press-shadow'
  | 'press-translate'
  | 'primary'
  | 'primary-border'
  | 'primary-foreground'
  | 'primary-hover'
  | 'primary-subtle'
  | 'radius-full'
  | 'radius-lg'
  | 'radius-md'
  | 'radius-sm'
  | 'radius-xl'
  | 'ring'
  | 'ring-color'
  | 'ring-offset'
  | 'ring-opacity'
  | 'ring-width'
  | 'scrim'
  | 'scrollbar-radius'
  | 'scrollbar-thumb'
  | 'scrollbar-thumb-hover'
  | 'scrollbar-track'
  | 'scrollbar-width'
  | 'scrollbar-width-thin'
  | 'secondary'
  | 'secondary-border'
  | 'secondary-foreground'
  | 'secondary-hover'
  | 'secondary-subtle'
  | 'selected-bg'
  | 'selected-border'
  | 'selected-strength'
  | 'shadow-accent'
  | 'shadow-card-rest'
  | 'shadow-edge-highlight'
  | 'shadow-edge-top'
  | 'shadow-hover'
  | 'shadow-inset-ring'
  | 'shadow-inset-soft'
  | 'shadow-lg'
  | 'shadow-md'
  | 'shadow-sm'
  | 'shadow-sticky'
  | 'shadow-xl'
  | 'shadow-xs'
  | 'size-control-2xs-h'
  | 'size-control-lg-h'
  | 'size-control-md-h'
  | 'size-control-sm-h'
  | 'size-control-xl-h'
  | 'size-control-xs-h'
  | 'size-icon-2xl-h'
  | 'size-icon-2xl-w'
  | 'size-icon-2xs-h'
  | 'size-icon-2xs-w'
  | 'size-icon-3xl-h'
  | 'size-icon-3xl-w'
  | 'size-icon-3xs-h'
  | 'size-icon-3xs-w'
  | 'size-icon-lg-h'
  | 'size-icon-lg-w'
  | 'size-icon-md-h'
  | 'size-icon-md-w'
  | 'size-icon-sm-h'
  | 'size-icon-sm-w'
  | 'size-icon-xl-h'
  | 'size-icon-xl-w'
  | 'size-icon-xs-h'
  | 'size-icon-xs-w'
  | 'space-1'
  | 'space-10'
  | 'space-11'
  | 'space-12'
  | 'space-14'
  | 'space-16'
  | 'space-1_5'
  | 'space-2'
  | 'space-20'
  | 'space-24'
  | 'space-28'
  | 'space-2_5'
  | 'space-3'
  | 'space-32'
  | 'space-36'
  | 'space-4'
  | 'space-40'
  | 'space-44'
  | 'space-48'
  | 'space-5'
  | 'space-52'
  | 'space-56'
  | 'space-6'
  | 'space-60'
  | 'space-64'
  | 'space-7'
  | 'space-8'
  | 'space-9'
  | 'space-half'
  | 'space-px'
  | 'success'
  | 'success-border'
  | 'success-foreground'
  | 'success-hover'
  | 'success-subtle'
  | 'surface'
  | 'syntax-attr'
  | 'syntax-comment'
  | 'syntax-function'
  | 'syntax-keyword'
  | 'syntax-number'
  | 'syntax-string'
  | 'syntax-tag'
  | 'syntax-type'
  | 'syntax-variable'
  | 'text-2xl'
  | 'text-3xl'
  | 'text-4xl'
  | 'text-base'
  | 'text-display'
  | 'text-lg'
  | 'text-md'
  | 'text-sm'
  | 'text-xl'
  | 'text-xs'
  | 'tooltip-hide-delay'
  | 'tooltip-show-delay'
  | 'tracking-normal'
  | 'tracking-tight'
  | 'tracking-tighter'
  | 'tracking-wide'
  | 'tracking-wider'
  | 'type-body-family'
  | 'type-body-leading'
  | 'type-body-size'
  | 'type-body-sm-family'
  | 'type-body-sm-leading'
  | 'type-body-sm-size'
  | 'type-body-sm-tracking'
  | 'type-body-sm-weight'
  | 'type-body-tracking'
  | 'type-body-weight'
  | 'type-display-family'
  | 'type-display-leading'
  | 'type-display-size'
  | 'type-display-sm-family'
  | 'type-display-sm-leading'
  | 'type-display-sm-size'
  | 'type-display-sm-tracking'
  | 'type-display-sm-weight'
  | 'type-display-tracking'
  | 'type-display-weight'
  | 'type-heading-case'
  | 'type-heading-family'
  | 'type-heading-leading'
  | 'type-heading-size'
  | 'type-heading-tracking'
  | 'type-heading-weight'
  | 'type-label-case'
  | 'type-label-family'
  | 'type-label-leading'
  | 'type-label-size'
  | 'type-label-tracking'
  | 'type-label-weight'
  | 'type-mono-family'
  | 'type-mono-leading'
  | 'type-mono-size'
  | 'type-mono-sm-family'
  | 'type-mono-sm-leading'
  | 'type-mono-sm-size'
  | 'type-mono-sm-tracking'
  | 'type-mono-sm-weight'
  | 'type-mono-tracking'
  | 'type-mono-weight'
  | 'warning'
  | 'warning-border'
  | 'warning-foreground'
  | 'warning-hover'
  | 'warning-subtle'
  | 'z-base'
  | 'z-dialog'
  | 'z-dropdown'
  | 'z-elevated'
  | 'z-max'
  | 'z-popover'
  | 'z-sticky'
  | 'z-toast'

