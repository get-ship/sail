// Controller for the <sui-app-shell> layout chrome.
//
// Owns the theme toggle: light is the token default (`:root`), dark is the
// `.dark` class on <html> (see @ship/tokens). The `dark` signal seeds from the
// live DOM so the toggle stays correct even though `mountRouter` rebuilds the
// chrome on every navigation (the class lives on <html>, outside the chrome).
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const root = document.documentElement
    const dark = signal(root.classList.contains('dark'))
    const toggleTheme = (): void => {
      const next = !dark()
      dark.set(next)
      root.classList.toggle('dark', next)
    }
    return { dark, toggleTheme }
  },
})
