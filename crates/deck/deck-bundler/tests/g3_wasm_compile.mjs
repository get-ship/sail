// G3 cross-producer parity — WASM side.
//
// Drives the SAME wasm `compile` export the docs playground / browser LSP
// worker ship (`packages/sui/compiler-wasm/pkg`, built by `just sui-wasm`)
// over a list of fixtures, and prints each fixture's emitted `.js` so the Rust
// gate (`tests/bundle_parity.rs`) can byte-diff it against the native
// `sail_compiler::compile` output. This is the #160 native-vs-wasm divergence
// check: one Rust grammar, two compile targets (x86_64 native + wasm32), one
// expected emit.
//
// Contract (stdin → stdout, both JSON):
//   stdin : { pkgDir, jobs: [{ name, source, opts }] }
//             opts is the wasm `WasmOpts` JSON-in shape (mode / componentName /
//             shadow / setupUrl / styles) — passed through verbatim.
//   stdout: { ok: true, results: [{ name, kind, js?, message? }] }
//           on a harness-level failure (cannot load wasm): { ok:false, error }
//
// No network, no daemon, no browser — `WebAssembly.instantiate` on the local
// `.wasm` bytes in this Node process. The Rust gate spawns this with a
// per-invocation wall-clock timeout and treats a non-zero exit / unparseable
// stdout as a gate failure.

import { readFileSync } from 'node:fs'
import { join } from 'node:path'

async function readStdin() {
  const chunks = []
  for await (const c of process.stdin) chunks.push(c)
  return Buffer.concat(chunks).toString('utf8')
}

async function main() {
  const input = JSON.parse(await readStdin())
  const { pkgDir, jobs } = input

  // Load the wasm-bindgen ESM shim + instantiate from local bytes. The shim's
  // default export takes `{ module_or_path }`; we pass the raw `.wasm` bytes so
  // there is no fetch / URL — purely in-process.
  const shim = await import(join(pkgDir, 'sui_compiler.js'))
  const wasmBytes = readFileSync(join(pkgDir, 'sui_compiler_bg.wasm'))
  await shim.default({ module_or_path: wasmBytes })

  const results = jobs.map((job) => {
    let res
    try {
      res = shim.compile(job.source, JSON.stringify(job.opts ?? {}))
    } catch (e) {
      return { name: job.name, kind: 'throw', message: String(e && e.message ? e.message : e) }
    }
    // `WasmResult` is a discriminated union: { kind:'ok', js, ... } |
    // { kind:'err', message, ... }.
    if (res && res.kind === 'ok') {
      return { name: job.name, kind: 'ok', js: res.js }
    }
    return { name: job.name, kind: 'err', message: res ? res.message : 'no result' }
  })

  process.stdout.write(JSON.stringify({ ok: true, results }))
}

main().catch((e) => {
  process.stdout.write(JSON.stringify({ ok: false, error: String(e && e.stack ? e.stack : e) }))
  process.exit(1)
})
