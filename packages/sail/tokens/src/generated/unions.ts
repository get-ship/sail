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
export const TONE_NAMES_SOURCE = 'dtcg-groups:color.{primary,info,success,warning,destructive,brandAccent→accent,muted}' as const

/** Semantic paint tone groups (Style bound `tone` / host prop). No `'default'`. */
export const TONE_NAMES = [
  'primary',
  'info',
  'success',
  'warning',
  'destructive',
  'accent',
  'muted',
] as const
export type Tone = (typeof TONE_NAMES)[number]

/** @deprecated Use {@link Tone}. */
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
  },
  success: {
    default: 'success',
    foreground: 'success-foreground',
  },
  warning: {
    default: 'warning',
    foreground: 'warning-foreground',
  },
  destructive: {
    default: 'destructive',
    foreground: 'destructive-foreground',
  },
  accent: {
    default: 'accent',
    foreground: 'accent-foreground',
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
  'ship',
  'neutral',
  'paper',
  'halo',
  'console',
  'contrast',
  'glass',
  'brut',
  '8bit',
  'synthwave',
  'sail',
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
  'space-0_75',
  'space-1',
  'space-10',
  'space-11',
  'space-12',
  'space-14',
  'space-16',
  'space-1_25',
  'space-1_5',
  'space-2',
  'space-20',
  'space-24',
  'space-28',
  'space-2_5',
  'space-3',
  'space-32',
  'space-36',
  'space-3_5',
  'space-4',
  'space-40',
  'space-44',
  'space-48',
  'space-4_5',
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
  'radius',
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
  'duration-glacial',
  'duration-instant',
  'duration-medium',
  'duration-slow',
] as const
export type DurationToken = (typeof DURATION_TOKENS)[number]

/** Brand / semantic surface paint tokens under color.* (excl. tone ramps & product accents). */
export const SEMANTIC_COLOR_TOKENS = [
  'accent',
  'accent-foreground',
  'backdrop',
  'background',
  'destructive',
  'destructive-foreground',
  'divider',
  'foreground',
  'info',
  'info-foreground',
  'input',
  'muted',
  'muted-foreground',
  'primary',
  'primary-foreground',
  'primary-hover',
  'ring',
  'ring-color',
  'scrim',
  'secondary',
  'secondary-foreground',
  'success',
  'success-foreground',
  'warning',
  'warning-foreground',
] as const
export type SemanticColorName = (typeof SEMANTIC_COLOR_TOKENS)[number]

/** Every color-typed token cssName in the registry (paint tokens). */
export type ColorTokenName =
  | 'accent'
  | 'accent-agents'
  | 'accent-canvas'
  | 'accent-chat'
  | 'accent-collections'
  | 'accent-dashboard'
  | 'accent-events'
  | 'accent-foreground'
  | 'accent-git'
  | 'accent-mesh'
  | 'accent-overview'
  | 'accent-pipelines'
  | 'accent-settings'
  | 'accent-setup'
  | 'accent-skills'
  | 'accent-specs'
  | 'accent-theme'
  | 'accordion'
  | 'accordion-border'
  | 'accordion-chevron'
  | 'accordion-foreground'
  | 'accordion-hover-bg'
  | 'active-bg'
  | 'alert'
  | 'alert-border'
  | 'alert-foreground'
  | 'alert-stripe'
  | 'avatar'
  | 'avatar-foreground'
  | 'backdrop'
  | 'background'
  | 'badge'
  | 'badge-border'
  | 'badge-foreground'
  | 'border'
  | 'border-default-color'
  | 'border-strong'
  | 'border-strong-color'
  | 'border-subtle'
  | 'border-subtle-color'
  | 'breadcrumb-current'
  | 'breadcrumb-foreground'
  | 'button'
  | 'button-border'
  | 'button-foreground'
  | 'button-hover-bg'
  | 'card'
  | 'card-border'
  | 'card-foreground'
  | 'chip'
  | 'chip-border'
  | 'chip-foreground'
  | 'code-block'
  | 'code-block-foreground'
  | 'combobox'
  | 'combobox-foreground'
  | 'context-menu'
  | 'context-menu-border'
  | 'context-menu-foreground'
  | 'destructive'
  | 'destructive-foreground'
  | 'dialog'
  | 'dialog-border'
  | 'dialog-divider'
  | 'dialog-footer'
  | 'dialog-foreground'
  | 'dialog-muted-foreground'
  | 'divider'
  | 'dropdown'
  | 'dropdown-border'
  | 'dropdown-foreground'
  | 'empty-state'
  | 'empty-state-border'
  | 'empty-state-foreground'
  | 'empty-state-title'
  | 'error-boundary'
  | 'error-boundary-border'
  | 'error-boundary-foreground'
  | 'error-boundary-icon'
  | 'error-boundary-message-foreground'
  | 'error-boundary-retry-bg'
  | 'error-boundary-retry-foreground'
  | 'error-state'
  | 'error-state-border'
  | 'error-state-foreground'
  | 'field'
  | 'field-border'
  | 'field-border-hover'
  | 'field-error'
  | 'field-foreground'
  | 'field-hint'
  | 'fieldset-border'
  | 'fill-button-color'
  | 'fill-card-color'
  | 'fill-muted-color'
  | 'fill-surface-color'
  | 'foreground'
  | 'hover-bg'
  | 'hover-card'
  | 'hover-card-border'
  | 'hover-card-foreground'
  | 'hue-amber'
  | 'hue-blue'
  | 'hue-cyan'
  | 'hue-fuchsia'
  | 'hue-green'
  | 'hue-indigo'
  | 'hue-lime'
  | 'hue-orange'
  | 'hue-pink'
  | 'hue-purple'
  | 'hue-red'
  | 'hue-rose'
  | 'hue-sky'
  | 'hue-teal'
  | 'hue-violet'
  | 'hue-yellow'
  | 'info'
  | 'info-foreground'
  | 'ink-button-color'
  | 'ink-card-color'
  | 'ink-foreground-color'
  | 'ink-muted-color'
  | 'input'
  | 'kbd'
  | 'kbd-border'
  | 'kbd-foreground'
  | 'label-foreground'
  | 'label-required'
  | 'legend-foreground'
  | 'menu-item-foreground'
  | 'menu-item-hover-bg'
  | 'meter'
  | 'meter-optimum'
  | 'meter-sub-suboptimum'
  | 'meter-suboptimum'
  | 'muted'
  | 'muted-foreground'
  | 'navbar'
  | 'navbar-border'
  | 'navbar-foreground'
  | 'page-current-bg'
  | 'page-current-foreground'
  | 'page-foreground'
  | 'page-hover-bg'
  | 'popover'
  | 'popover-border'
  | 'popover-foreground'
  | 'primary'
  | 'primary-foreground'
  | 'primary-hover'
  | 'progress'
  | 'progress-fill'
  | 'prose-foreground'
  | 'radio-foreground'
  | 'ring'
  | 'ring-color'
  | 'scrim'
  | 'scroll-area-thumb'
  | 'scroll-area-thumb-hover'
  | 'scrollbar-thumb'
  | 'scrollbar-thumb-hover'
  | 'secondary'
  | 'secondary-foreground'
  | 'selected-bg'
  | 'selected-border'
  | 'separator-line'
  | 'sidebar'
  | 'sidebar-accent'
  | 'sidebar-accent-foreground'
  | 'sidebar-border'
  | 'sidebar-foreground'
  | 'sidebar-primary'
  | 'sidebar-primary-foreground'
  | 'sidebar-ring'
  | 'skeleton'
  | 'skeleton-highlight'
  | 'slider-fill'
  | 'slider-track'
  | 'slider-track-border'
  | 'spinner-foreground'
  | 'spinner-track'
  | 'status-amber'
  | 'status-blue'
  | 'status-gray'
  | 'status-green'
  | 'status-orange'
  | 'status-purple'
  | 'status-red'
  | 'status-yellow'
  | 'success'
  | 'success-foreground'
  | 'surface-0'
  | 'surface-1'
  | 'surface-2'
  | 'surface-3'
  | 'surface-inverse'
  | 'switch'
  | 'switch-border'
  | 'switch-thumb'
  | 'syntax-attr'
  | 'syntax-comment'
  | 'syntax-function'
  | 'syntax-keyword'
  | 'syntax-number'
  | 'syntax-string'
  | 'syntax-tag'
  | 'syntax-type'
  | 'syntax-variable'
  | 'tab-border'
  | 'tab-foreground'
  | 'tab-list'
  | 'tab-list-border'
  | 'tab-list-rail'
  | 'tab-selected-bg'
  | 'tab-selected-foreground'
  | 'table-foreground'
  | 'table-muted-foreground'
  | 'table-sticky-bg'
  | 'table-stripe'
  | 'theme-picker'
  | 'theme-picker-foreground'
  | 'theme-picker-option-foreground'
  | 'theme-picker-option-hover-bg'
  | 'theme-picker-trigger-foreground'
  | 'toast'
  | 'toast-border'
  | 'toast-foreground'
  | 'toast-stripe'
  | 'toggle'
  | 'toggle-border'
  | 'toggle-foreground'
  | 'toggle-group'
  | 'toggle-group-border'
  | 'tooltip'
  | 'tooltip-foreground'
  | 'warning'
  | 'warning-foreground'

/** Every public token cssName in the registry (without the -- prefix). */
export type CssTokenName =
  | 'accent'
  | 'accent-agents'
  | 'accent-canvas'
  | 'accent-chat'
  | 'accent-collections'
  | 'accent-dashboard'
  | 'accent-events'
  | 'accent-foreground'
  | 'accent-git'
  | 'accent-mesh'
  | 'accent-overview'
  | 'accent-pipelines'
  | 'accent-settings'
  | 'accent-setup'
  | 'accent-skills'
  | 'accent-specs'
  | 'accent-theme'
  | 'accordion'
  | 'accordion-border'
  | 'accordion-chevron'
  | 'accordion-foreground'
  | 'accordion-hover-bg'
  | 'active-bg'
  | 'active-strength'
  | 'alert'
  | 'alert-border'
  | 'alert-foreground'
  | 'alert-stripe'
  | 'avatar'
  | 'avatar-foreground'
  | 'backdrop'
  | 'background'
  | 'badge'
  | 'badge-border'
  | 'badge-foreground'
  | 'body-font'
  | 'body-weight'
  | 'border'
  | 'border-default-color'
  | 'border-default-style'
  | 'border-default-width'
  | 'border-strong'
  | 'border-strong-color'
  | 'border-strong-style'
  | 'border-strong-width'
  | 'border-subtle'
  | 'border-subtle-color'
  | 'border-subtle-style'
  | 'border-subtle-width'
  | 'border-width'
  | 'breadcrumb-current'
  | 'breadcrumb-foreground'
  | 'breakpoint-desktop'
  | 'breakpoint-mobile'
  | 'breakpoint-tablet'
  | 'button'
  | 'button-border'
  | 'button-foreground'
  | 'button-hover-bg'
  | 'card'
  | 'card-backdrop-filter'
  | 'card-border'
  | 'card-border-style'
  | 'card-border-width'
  | 'card-foreground'
  | 'card-texture'
  | 'card-texture-blend'
  | 'card-texture-opacity'
  | 'chip'
  | 'chip-border'
  | 'chip-foreground'
  | 'code-block'
  | 'code-block-foreground'
  | 'combobox'
  | 'combobox-foreground'
  | 'context-menu'
  | 'context-menu-border'
  | 'context-menu-foreground'
  | 'control-frame'
  | 'control-h-2xs'
  | 'control-h-lg'
  | 'control-h-md'
  | 'control-h-sm'
  | 'control-h-xl'
  | 'control-h-xs'
  | 'control-lift'
  | 'control-stroke'
  | 'corner-image'
  | 'density-icon'
  | 'density-row-gap'
  | 'density-row-px'
  | 'density-row-py'
  | 'density-text'
  | 'destructive'
  | 'destructive-foreground'
  | 'dialog'
  | 'dialog-border'
  | 'dialog-border-style'
  | 'dialog-border-width'
  | 'dialog-divider'
  | 'dialog-footer'
  | 'dialog-foreground'
  | 'dialog-muted-foreground'
  | 'display-font'
  | 'divider'
  | 'dropdown'
  | 'dropdown-border'
  | 'dropdown-foreground'
  | 'duration-deliberate'
  | 'duration-fast'
  | 'duration-glacial'
  | 'duration-instant'
  | 'duration-medium'
  | 'duration-slow'
  | 'ease-in'
  | 'ease-in-out'
  | 'ease-out'
  | 'ease-out-soft'
  | 'ease-spring'
  | 'edge-highlight'
  | 'elevation-lg-shadow'
  | 'elevation-md-shadow'
  | 'elevation-none-shadow'
  | 'elevation-sm-shadow'
  | 'elevation-xl-shadow'
  | 'elevation-xs-shadow'
  | 'empty-state'
  | 'empty-state-border'
  | 'empty-state-foreground'
  | 'empty-state-title'
  | 'error-boundary'
  | 'error-boundary-border'
  | 'error-boundary-foreground'
  | 'error-boundary-icon'
  | 'error-boundary-message-foreground'
  | 'error-boundary-retry-bg'
  | 'error-boundary-retry-foreground'
  | 'error-state'
  | 'error-state-border'
  | 'error-state-foreground'
  | 'field'
  | 'field-border'
  | 'field-border-hover'
  | 'field-error'
  | 'field-foreground'
  | 'field-hint'
  | 'fieldset-border'
  | 'fill-button-color'
  | 'fill-card-color'
  | 'fill-muted-color'
  | 'fill-surface-color'
  | 'focus-ring'
  | 'focus-ring-offset'
  | 'font-body'
  | 'font-body-app'
  | 'font-display'
  | 'font-display-app'
  | 'font-mono'
  | 'font-mono-app'
  | 'font-press-start'
  | 'font-press-start-app'
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
  | 'heading-font'
  | 'heading-tracking'
  | 'heading-transform'
  | 'heading-weight'
  | 'hover-bg'
  | 'hover-card'
  | 'hover-card-border'
  | 'hover-card-foreground'
  | 'hover-strength'
  | 'hue-amber'
  | 'hue-blue'
  | 'hue-cyan'
  | 'hue-fuchsia'
  | 'hue-green'
  | 'hue-indigo'
  | 'hue-lime'
  | 'hue-orange'
  | 'hue-pink'
  | 'hue-purple'
  | 'hue-red'
  | 'hue-rose'
  | 'hue-sky'
  | 'hue-teal'
  | 'hue-violet'
  | 'hue-yellow'
  | 'icon-size-2xl'
  | 'icon-size-2xs'
  | 'icon-size-3xl'
  | 'icon-size-3xs'
  | 'icon-size-lg'
  | 'icon-size-md'
  | 'icon-size-sm'
  | 'icon-size-xl'
  | 'icon-size-xs'
  | 'info'
  | 'info-foreground'
  | 'ink-button-color'
  | 'ink-card-color'
  | 'ink-foreground-color'
  | 'ink-muted-color'
  | 'input'
  | 'kbd'
  | 'kbd-border'
  | 'kbd-foreground'
  | 'label-font'
  | 'label-foreground'
  | 'label-required'
  | 'label-tracking'
  | 'label-transform'
  | 'label-weight'
  | 'leading-code'
  | 'leading-loose'
  | 'leading-none'
  | 'leading-normal'
  | 'leading-relaxed'
  | 'leading-snug'
  | 'leading-tight'
  | 'legend-foreground'
  | 'menu-item-foreground'
  | 'menu-item-hover-bg'
  | 'meter'
  | 'meter-optimum'
  | 'meter-sub-suboptimum'
  | 'meter-suboptimum'
  | 'mix-bg'
  | 'mix-border'
  | 'mix-hover'
  | 'mono-font'
  | 'muted'
  | 'muted-foreground'
  | 'navbar'
  | 'navbar-border'
  | 'navbar-foreground'
  | 'page-current-bg'
  | 'page-current-foreground'
  | 'page-foreground'
  | 'page-hover-bg'
  | 'panel-body-pad-y'
  | 'panel-ch-w-lg'
  | 'panel-ch-w-md'
  | 'panel-close-inset'
  | 'panel-foot-pad-y'
  | 'panel-head-pad-y'
  | 'panel-max-h-lg'
  | 'panel-max-h-md'
  | 'panel-max-h-sm'
  | 'panel-max-w-lg'
  | 'panel-max-w-md'
  | 'panel-max-w-sm'
  | 'panel-max-w-xs'
  | 'panel-menu-min-w'
  | 'panel-min-h-cell'
  | 'panel-min-h-daycol'
  | 'panel-min-w-lg'
  | 'panel-min-w-md'
  | 'panel-min-w-sm'
  | 'panel-pad-card'
  | 'panel-pad-menu'
  | 'panel-pad-x'
  | 'panel-rail-w'
  | 'panel-toast-region-width'
  | 'panel-vh-lg'
  | 'panel-vh-md'
  | 'panel-vh-sm'
  | 'panel-w-side'
  | 'panel-w-toast'
  | 'play-colors-property'
  | 'popover'
  | 'popover-border'
  | 'popover-foreground'
  | 'press-scale'
  | 'press-shadow'
  | 'press-translate'
  | 'primary'
  | 'primary-foreground'
  | 'primary-hover'
  | 'progress'
  | 'progress-fill'
  | 'prose-foreground'
  | 'radio-foreground'
  | 'radius'
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
  | 'scroll-area-thumb'
  | 'scroll-area-thumb-hover'
  | 'scrollbar-radius'
  | 'scrollbar-thumb'
  | 'scrollbar-thumb-hover'
  | 'scrollbar-track'
  | 'scrollbar-width'
  | 'scrollbar-width-thin'
  | 'secondary'
  | 'secondary-foreground'
  | 'selected-bg'
  | 'selected-border'
  | 'selected-strength'
  | 'separator-line'
  | 'shadow-accent'
  | 'shadow-card-rest'
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
  | 'shape-pill'
  | 'sidebar'
  | 'sidebar-accent'
  | 'sidebar-accent-foreground'
  | 'sidebar-border'
  | 'sidebar-foreground'
  | 'sidebar-primary'
  | 'sidebar-primary-foreground'
  | 'sidebar-ring'
  | 'size-control-lg-h'
  | 'size-control-md-h'
  | 'size-control-sm-h'
  | 'size-full-h'
  | 'size-full-w'
  | 'skeleton'
  | 'skeleton-highlight'
  | 'slider-fill'
  | 'slider-track'
  | 'slider-track-border'
  | 'space-0_75'
  | 'space-1'
  | 'space-10'
  | 'space-11'
  | 'space-12'
  | 'space-14'
  | 'space-16'
  | 'space-1_25'
  | 'space-1_5'
  | 'space-2'
  | 'space-20'
  | 'space-24'
  | 'space-28'
  | 'space-2_5'
  | 'space-3'
  | 'space-32'
  | 'space-36'
  | 'space-3_5'
  | 'space-4'
  | 'space-40'
  | 'space-44'
  | 'space-48'
  | 'space-4_5'
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
  | 'spinner-foreground'
  | 'spinner-track'
  | 'status-amber'
  | 'status-blue'
  | 'status-gray'
  | 'status-green'
  | 'status-orange'
  | 'status-purple'
  | 'status-red'
  | 'status-yellow'
  | 'success'
  | 'success-foreground'
  | 'surface-0'
  | 'surface-1'
  | 'surface-2'
  | 'surface-3'
  | 'surface-inverse'
  | 'surface-overlay'
  | 'surface-overlay-blend'
  | 'surface-overlay-opacity'
  | 'switch'
  | 'switch-border'
  | 'switch-thumb'
  | 'syntax-attr'
  | 'syntax-comment'
  | 'syntax-function'
  | 'syntax-keyword'
  | 'syntax-number'
  | 'syntax-string'
  | 'syntax-tag'
  | 'syntax-type'
  | 'syntax-variable'
  | 'tab-border'
  | 'tab-foreground'
  | 'tab-list'
  | 'tab-list-border'
  | 'tab-list-rail'
  | 'tab-selected-bg'
  | 'tab-selected-foreground'
  | 'table-foreground'
  | 'table-muted-foreground'
  | 'table-sticky-bg'
  | 'table-stripe'
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
  | 'theme-picker'
  | 'theme-picker-foreground'
  | 'theme-picker-option-foreground'
  | 'theme-picker-option-hover-bg'
  | 'theme-picker-trigger-foreground'
  | 'toast'
  | 'toast-border'
  | 'toast-foreground'
  | 'toast-stripe'
  | 'toggle'
  | 'toggle-border'
  | 'toggle-foreground'
  | 'toggle-group'
  | 'toggle-group-border'
  | 'tooltip'
  | 'tooltip-foreground'
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
  | 'type-display-sm-family'
  | 'type-display-sm-leading'
  | 'type-display-sm-size'
  | 'type-display-sm-tracking'
  | 'type-display-sm-weight'
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
  | 'warning-foreground'
  | 'z-base'
  | 'z-dialog'
  | 'z-dropdown'
  | 'z-elevated'
  | 'z-max'
  | 'z-popover'
  | 'z-sticky'
  | 'z-toast'
  | 'z-toast-region'

