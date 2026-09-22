/**
 * Confirm hooks for destructive / ambiguous theme actions.
 * Default: platform confirm via studio-shell (Framework). Tests inject via setThemeConfirm.
 */
import { platformConfirm, setPlatformConfirm } from '@ship/sail-studio-shell'

export type ThemeConfirmFn = (message: string) => boolean

export function setThemeConfirm(fn: ThemeConfirmFn | null): void {
  setPlatformConfirm(fn)
}

export function themeConfirm(message: string): boolean {
  return platformConfirm(message)
}

export function confirmDeleteTheme(name: string): boolean {
  return themeConfirm(
    `Delete project theme “${name}”? This removes the theme JSON from the project and cannot be undone.`,
  )
}

export function confirmAddToProject(sourceName: string, newId: string): boolean {
  return themeConfirm(
    `Add “${sourceName}” to the project as “${newId}”?`,
  )
}

/** New theme from home — not “Add catalog pack” wording. */
export function confirmNewTheme(sourceName: string, newId: string): boolean {
  return themeConfirm(
    `Create new project theme “${newId}” based on “${sourceName}”?`,
  )
}

export function confirmRemixTheme(sourceName: string, newId: string): boolean {
  return themeConfirm(
    `Remix “${sourceName}” into a new project theme “${newId}”?`,
  )
}
