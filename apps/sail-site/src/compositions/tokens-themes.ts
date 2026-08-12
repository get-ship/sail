/** App-local composition facade for theme catalog (import convention). */
export {
  THEMES,
  type ThemeId,
  type ThemeMeta,
  themeMeta,
  themeSupportsMode,
  applyTheme,
  applyMode,
  readStoredTheme,
  persistTheme,
  readStoredMode,
  persistMode,
} from '../vendor/tokens-themes'
