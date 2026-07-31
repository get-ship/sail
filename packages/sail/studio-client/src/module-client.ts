
import { callRpc, opsHealth, type ClientOptions, type JsonObject } from './generated/client'

export type ModuleActivityInfo = {
  id: string
  label: string
  icon?: string
  route: string
  module: string
  devUnsigned?: boolean
}

export type ModuleInfo = {
  name: string
  version: string
  api: number
  signed: boolean
  devUnsigned?: boolean
  methods?: string[]
}

export type ModulesListing = {
  ok: boolean
  api: number
  modules: ModuleInfo[]
  activities: ModuleActivityInfo[]
  methods: string[]
}

export type ModuleMethodFn = (
  args?: object,
  opts?: ClientOptions,
) => Promise<JsonObject>

export async function fetchModules(opts?: ClientOptions): Promise<ModulesListing> {
  const base = (opts?.baseUrl ?? '').replace(/\/$/, '')
  const f = opts?.fetch ?? globalThis.fetch.bind(globalThis)
  try {
    const res = await f(`${base}/sail.ops.v1.modules`, { cache: 'no-store' })
    if (!res.ok) {
      return { ok: false, api: 1, modules: [], activities: [], methods: [] }
    }
    const json = (await res.json()) as ModulesListing
    return {
      ok: !!json.ok,
      api: json.api ?? 1,
      modules: json.modules ?? [],
      activities: json.activities ?? [],
      methods: json.methods ?? [],
    }
  } catch {
    return { ok: false, api: 1, modules: [], activities: [], methods: [] }
  }
}

/**
 * Build a bag of `namespaceMethod` functions from `"Service/Method"` strings.
 * Uses `callRpc` — same transport as core generated methods.
 */
export function createModuleMethodMap(
  methods: string[],
  opts?: ClientOptions,
): Record<string, ModuleMethodFn> {
  const out: Record<string, ModuleMethodFn> = {}
  for (const rpc of methods) {
    const slash = rpc.indexOf('/')
    if (slash <= 0) continue
    const service = rpc.slice(0, slash)
    const method = rpc.slice(slash + 1)
    if (!service || !method) continue
    const fnName = `${service[0]!.toLowerCase()}${service.slice(1)}${method}`
    out[fnName] = (args = {}, callOpts) =>
      callRpc(service, method, args, callOpts ?? opts)
  }
  return out
}

/**
 * Discover module methods from health (includes `modules.methods`) or modules
 * endpoint and return a typed-ish call map without touching core codegen.
 */
export async function createModuleClient(
  opts?: ClientOptions,
): Promise<{
  listing: ModulesListing
  methods: Record<string, ModuleMethodFn>
  /** Convenience: ModuleHello/Ping when the reference module is loaded. */
  moduleHelloPing: ModuleMethodFn
}> {
  let listing = await fetchModules(opts)
  if (!listing.methods.length) {
    try {
      const health = await opsHealth(opts)
      const mods = health.modules as { methods?: string[] } | undefined
      if (mods?.methods?.length) {
        listing = {
          ...listing,
          methods: mods.methods as string[],
          ok: true,
        }
      }
    } catch {
      /* empty */
    }
  }
  const methods = createModuleMethodMap(listing.methods, opts)
  // Only bind ModuleHello convenience when the reference module is actually loaded.
  const moduleHelloPing: ModuleMethodFn =
    methods.moduleHelloPing ??
    (async () => {
      throw new Error('ModuleHello/Ping is not loaded (no reference module in /sail.ops.v1.modules)')
    })
  return { listing, methods, moduleHelloPing }
}
