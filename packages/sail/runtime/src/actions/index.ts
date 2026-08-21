// Built-in `use:` actions — reusable element behaviors the L1 components lean
// on. Each is a plain `Action` (see ../action.ts): it receives the element,
// optionally a reactive param, and returns `{ update?, destroy? }`. Authored
// once here, attached anywhere with `use:<name>` — the primitive that removes
// hand-wired refs + effects for element behavior.

export { autosize } from './autosize'
export { clipboard } from './clipboard'
export type { ClipboardParam } from './clipboard'
export { clickOutside } from './click-outside'
export { hotkeys } from './hotkeys'
export type { HotkeyEntry, HotkeyHandler, HotkeysMap } from './hotkeys'
export { drawerSwipe } from './drawer-swipe'
export { focus } from './focus'
export { inView } from './in-view'
export type { InViewHandler } from './in-view'
export { mediaFallback } from './media-fallback'

export { recoverySurface } from './recovery-surface'
export { resizable } from './resizable'
export { catchRender } from './catch-render'
export { syntaxHighlight } from './syntax-highlight'
export { registerSailHighlighter } from './highlight'
export type { SailHtmlHighlighter } from './highlight'

export { chromeOffset } from './chrome-offset'
export { describedControl } from './described-control'
export { autoDismiss } from './auto-dismiss'
export { dismissControl } from './dismiss-control'

export { themeSelect,
  setEffectiveThemeIds,
  readEffectiveThemeIds,
  parseEffectiveThemeIdsContent,
  effectiveProductThemes,
} from './theme-select'
export type { EffectiveSessionIds } from './theme-select'
export { menu } from './menu'
export { virtualRow, virtualSpacer, cellPad } from './virtual-row'
export { eventSpan } from './event-span'
export type { RectSpan } from './event-span'
export { rect, pin } from './rect-box'
export { portEdge } from './port-edge'
export type { PortSide } from './port-edge'
export type { BoxRect, PinPoint } from './rect-box'
export { worldTransform } from './world-transform'
export { zoomPan, maxWidth, swatch, liveBox } from './zoom-pan'
export type { ZoomPanState } from './zoom-pan'
export { gridTracks } from './grid-tracks'
export { spotlight } from './spotlight'
export { targetCursor } from './target-cursor'
