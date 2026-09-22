// invokeCallback — closed-widget helper for L1 typed callback props.
//
// Author surface: kebab `on-*` attrs (`on-select={fn}`) lower to camelCase
// JS properties on the host (`el.onSelect = fn`) via bindProp. Widget
// behaviors call the prop instead of dispatching CustomEvents.
//
// Props flow down. Callbacks report up. bind: syncs a value. There is no
// event bus.

/**
 * Invoke a typed callback prop on `host` when present.
 *
 * @param host Element that may carry `name` as a function property.
 * @param name CamelCase prop name (`onSelect`, `onPageChange`, …).
 * @param args Payload args — typically the former CustomEvent.detail.
 */
export function invokeCallback(
  host: Element,
  name: string,
  ...args: unknown[]
): void {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const fn = (host as any)[name]
  if (typeof fn === 'function') {
    fn(...args)
  }
}
