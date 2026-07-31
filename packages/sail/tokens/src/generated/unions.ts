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
export const TONE_NAMES_SOURCE = 'l1-convention:TONE_FALLBACK (host prop axis; not design-token ramps)' as const

/** L1 host tone prop axis (component convention). Not design-token ramps. */
export const TONE_NAMES = [
  'default',
  'primary',
  'info',
  'success',
  'warning',
  'destructive',
  'accent',
  'muted',
] as const
export type ToneName = (typeof TONE_NAMES)[number]

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
  'my-ship',
  'studio',
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
  | 'active-bg'
  | 'alert'
  | 'alert-foreground'
  | 'alert-stripe'
  | 'app-accent-agents'
  | 'app-accent-canvas'
  | 'app-accent-chat'
  | 'app-accent-collections'
  | 'app-accent-dashboard'
  | 'app-accent-events'
  | 'app-accent-git'
  | 'app-accent-mesh'
  | 'app-accent-overview'
  | 'app-accent-pipelines'
  | 'app-accent-settings'
  | 'app-accent-setup'
  | 'app-accent-skills'
  | 'app-accent-specs'
  | 'app-accent-theme'
  | 'avatar'
  | 'avatar-foreground'
  | 'backdrop'
  | 'background'
  | 'badge'
  | 'badge-border'
  | 'badge-foreground'
  | 'border'
  | 'border-strong'
  | 'border-subtle'
  | 'card'
  | 'card-foreground'
  | 'chip'
  | 'chip-border'
  | 'chip-foreground'
  | 'code-block'
  | 'code-block-foreground'
  | 'combobox'
  | 'combobox-foreground'
  | 'context-menu'
  | 'context-menu-foreground'
  | 'destructive'
  | 'destructive-foreground'
  | 'divider'
  | 'dropdown'
  | 'dropdown-foreground'
  | 'empty-state'
  | 'empty-state-border'
  | 'empty-state-foreground'
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
  | 'field-error'
  | 'field-hint'
  | 'foreground'
  | 'hover-bg'
  | 'hover-card'
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
  | 'input'
  | 'label-foreground'
  | 'label-required'
  | 'menu-item-foreground'
  | 'menu-item-hover-bg'
  | 'muted'
  | 'muted-foreground'
  | 'navbar'
  | 'navbar-foreground'
  | 'page-current-bg'
  | 'page-current-foreground'
  | 'page-foreground'
  | 'page-hover-bg'
  | 'popover'
  | 'popover-foreground'
  | 'primary'
  | 'primary-foreground'
  | 'primary-hover'
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
  | 'tab-foreground'
  | 'tab-list'
  | 'tab-list-border'
  | 'tab-list-rail'
  | 'tab-selected'
  | 'tab-selected-foreground'
  | 'theme-picker'
  | 'theme-picker-foreground'
  | 'theme-picker-option-foreground'
  | 'theme-picker-option-hover-bg'
  | 'theme-picker-trigger-foreground'
  | 'toast'
  | 'toast-border'
  | 'toast-foreground'
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
  | 'active-bg'
  | 'active-strength'
  | 'alert'
  | 'alert-foreground'
  | 'alert-stripe'
  | 'app-accent-agents'
  | 'app-accent-canvas'
  | 'app-accent-chat'
  | 'app-accent-collections'
  | 'app-accent-dashboard'
  | 'app-accent-events'
  | 'app-accent-git'
  | 'app-accent-mesh'
  | 'app-accent-overview'
  | 'app-accent-pipelines'
  | 'app-accent-settings'
  | 'app-accent-setup'
  | 'app-accent-skills'
  | 'app-accent-specs'
  | 'app-accent-theme'
  | 'avatar'
  | 'avatar-foreground'
  | 'backdrop'
  | 'background'
  | 'badge'
  | 'badge-border'
  | 'badge-foreground'
  | 'body-font'
  | 'body-tracking'
  | 'body-weight'
  | 'border'
  | 'border-strong'
  | 'border-subtle'
  | 'border-width'
  | 'breakpoint-desktop'
  | 'breakpoint-mobile'
  | 'breakpoint-tablet'
  | 'card'
  | 'card-backdrop-filter'
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
  | 'display-font'
  | 'divider'
  | 'dropdown'
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
  | 'empty-state'
  | 'empty-state-border'
  | 'empty-state-foreground'
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
  | 'field-error'
  | 'field-hint'
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
  | 'glow-default'
  | 'glow-destructive'
  | 'glow-info'
  | 'glow-muted'
  | 'glow-primary'
  | 'glow-success'
  | 'glow-warning'
  | 'gradient-accent'
  | 'gradient-primary'
  | 'gradient-surface'
  | 'heading-font'
  | 'heading-tracking'
  | 'heading-transform'
  | 'heading-weight'
  | 'hover-bg'
  | 'hover-card'
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
  | 'input'
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
  | 'leading-normal-plus'
  | 'leading-relaxed'
  | 'leading-snug'
  | 'leading-tight'
  | 'menu-item-foreground'
  | 'menu-item-hover-bg'
  | 'menu-min-w'
  | 'mix-bg'
  | 'mix-border'
  | 'mix-hover'
  | 'mono-font'
  | 'muted'
  | 'muted-foreground'
  | 'navbar'
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
  | 'panel-min-h-cell'
  | 'panel-min-h-daycol'
  | 'panel-min-w-lg'
  | 'panel-min-w-md'
  | 'panel-min-w-sm'
  | 'panel-pad-card'
  | 'panel-pad-menu'
  | 'panel-pad-x'
  | 'panel-rail-w'
  | 'panel-vh-lg'
  | 'panel-vh-md'
  | 'panel-vh-sm'
  | 'panel-w-side'
  | 'panel-w-toast'
  | 'popover'
  | 'popover-foreground'
  | 'press-scale'
  | 'press-shadow'
  | 'press-translate'
  | 'primary'
  | 'primary-foreground'
  | 'primary-hover'
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
  | 'tab-foreground'
  | 'tab-list'
  | 'tab-list-border'
  | 'tab-list-rail'
  | 'tab-selected'
  | 'tab-selected-foreground'
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
  | 'toast-region-width'
  | 'toast-region-z'
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

