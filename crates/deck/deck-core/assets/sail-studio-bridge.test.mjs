/**
 * Smoke tests for the baked sail-studio-bridge asset path.
 * Host logic unit tests live in packages/sail/studio-bridge (vitest).
 * Run: node crates/deck/deck-core/assets/sail-studio-bridge.test.mjs
 */
import { readFileSync, existsSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import assert from 'node:assert/strict'

const HERE = dirname(fileURLToPath(import.meta.url))
const SRC = join(HERE, 'sail-studio-bridge.js')
const PACK = join(HERE, '../../../../packages/sail/studio-bridge/src')

assert(existsSync(PACK), 'packages/sail/studio-bridge/src must exist (SSOT)')
assert(existsSync(join(PACK, 'bridge.ts')), 'bridge.ts must exist with attachStudioBridge')
assert(existsSync(join(PACK, 'index.ts')), 'index.ts must export attachStudioBridge')

const bridge = readFileSync(join(PACK, 'bridge.ts'), 'utf8')
assert(/attachStudioBridge/.test(bridge), 'must export attachStudioBridge')
assert(/deck-mode/.test(bridge), 'must gate on deck-mode development')
assert(/documentElement/.test(bridge), 'must mount on documentElement (outside #root)')
assert(!/\/sail\.ops\.v1\//.test(bridge), 'bridge must contain no ops URLs')
assert(/postMessage/.test(bridge), 'bridge talks via postMessage')
assert(/contain:\s*strict/.test(bridge), 'host must use contain:strict')
assert(/attachShadow/.test(bridge), 'host must use shadow DOM')

if (existsSync(SRC)) {
  const source = readFileSync(SRC, 'utf8')
  if (!source.includes('deck bootstrap stub')) {
    assert(/attachStudioBridge/.test(source), 'baked asset must export attachStudioBridge')
    assert(!/\/sail\.ops\.v1\//.test(source), 'baked asset must contain no ops URLs')
    const bakedBytes = Buffer.byteLength(source, 'utf8')
    assert(bakedBytes < 5 * 1024, `baked bridge ${bakedBytes} B must be < 5KB`)
  }
}

console.log('sail-studio-bridge pack smoke checks passed.')
