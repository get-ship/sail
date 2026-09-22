// Template hook registry — `<use-*>` element name → hook factory.
//
// Hooks are registered once at module load. The dom hydrate flow looks
// each `<use-name>` element up in this registry and invokes the hook
// factory after setup() and before render.
//
// The registry is an in-process singleton; for tests, `clearRegistry()`
// resets it.

import type { TemplateHook } from '../controller'

const registry = new Map<string, TemplateHook<Record<string, unknown>, unknown>>()

/**
 * Register a template hook so `<use-<name>>` elements in `.sail`
 * templates resolve to its factory at hydrate time.
 *
 * Re-registering the same name overwrites — useful for tests; in
 * production the runtime ships its built-in hooks at module load and
 * applications shouldn't re-register them.
 */
export function registerTemplateHook<
  Attrs extends Record<string, unknown>,
  Contributes,
>(hook: TemplateHook<Attrs, Contributes>): void {
  registry.set(
    hook.name,
    hook as unknown as TemplateHook<Record<string, unknown>, unknown>,
  )
}

/**
 * Look up a template hook by `<use-<name>>` tag suffix. Returns
 * `undefined` if no hook is registered under that name.
 */
export function getTemplateHook(
  name: string,
): TemplateHook<Record<string, unknown>, unknown> | undefined {
  return registry.get(name)
}

/**
 * Drop every registered hook. Test-only; production code should never
 * call this. Built-in hook registration is module-load-time, so a
 * `clearRegistry()` call in production strands `<use-*>` elements with
 * no factory.
 */
export function clearRegistry(): void {
  registry.clear()
}

/**
 * Iterate registered hooks (for diagnostics / introspection).
 */
export function listRegisteredHooks(): string[] {
  return Array.from(registry.keys()).sort()
}
