/**
 * Confirm hooks for destructive / ambiguous theme actions.
 * Default: window.confirm (platform dialog). Tests inject via setThemeConfirm.
 */
export type ThemeConfirmFn = (message: string) => boolean

let confirmFn: ThemeConfirmFn = (message) => {
  if (typeof window === 'undefined' || typeof window.confirm !== 'function') {
    // Headless tests without DOM — allow (callers that need deny inject).
    return true
  }
  return window.confirm(message)
}

export function setThemeConfirm(fn: ThemeConfirmFn | null): void {
  confirmFn =
    fn ??
    ((message) => {
      if (typeof window === 'undefined' || typeof window.confirm !== 'function') {
        return true
      }
      return window.confirm(message)
    })
}

export function themeConfirm(message: string): boolean {
  return confirmFn(message)
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

export function confirmRemixTheme(sourceName: string, newId: string): boolean {
  return themeConfirm(
    `Remix “${sourceName}” into a new project theme “${newId}”?`,
  )
}
