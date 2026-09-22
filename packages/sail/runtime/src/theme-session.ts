// Theme session writers — tokens owns DATA/impl; runtime owns ambient API
// (SPEC-AMBIENT-SURFACE-1 §3). Re-export for RUNTIME_AMBIENT inject.
export {
  applyTheme,
  persistTheme,
  readStoredTheme,
  applyMode,
  bootTheme,
  persistMode,
  readStoredMode,
} from '@ship/tokens/themes'
