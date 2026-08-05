/**
 * Studio CSS containment — host shadow + no documentElement cursor; drag box in shadow.
 */
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { attachStudioBridge, STUDIO_HOST_ID } from '@ship/sail-studio-bridge'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')

describe('studio style containment', () => {
  it('bridge host uses shadow DOM + contain:strict + all:initial', () => {
    const bridge = readFileSync(
      join(REPO, 'packages/sail/studio-bridge/src/bridge.ts'),
      'utf8',
    )
    expect(bridge).toMatch(/attachShadow/)
    expect(bridge).toMatch(/contain:\s*strict/)
    expect(bridge).toMatch(/all:\s*initial/)
    expect(bridge).toMatch(/sail-studio-root/)
    // Drag box in shadow, not light-DOM documentElement
    expect(bridge).toMatch(/getElementById\('box'\)|id=box/)
    expect(bridge).not.toMatch(/documentElement\.style\.cursor/)
    // CSS links go into shadow, not document.head
    expect(bridge).not.toMatch(/document\.head\.appendChild/)
  })

  it('studio shell marks its root for isolation tests', () => {
    const chrome = readFileSync(
      join(REPO, 'apps/studio/src/sail-studio-chrome/sail-studio-chrome.sail'),
      'utf8',
    )
    expect(chrome).toMatch(/sail-studio-shell/)
    expect(chrome).toMatch(/studio-rail/)
  })

  it('runtime: host is outside #root and has shadow root', () => {
    document.documentElement.innerHTML =
      '<meta name="deck-mode" content="development" /><div id="root"><main id="app">app</main></div>'
    const handle = attachStudioBridge()
    const host = document.getElementById(STUDIO_HOST_ID)!
    const root = document.getElementById('root')!
    expect(host).toBeTruthy()
    expect(host.shadowRoot).toBeTruthy()
    expect(root.contains(host)).toBe(false)
    // App element is not inside studio host
    const app = document.getElementById('app')!
    expect(host.contains(app)).toBe(false)
    handle.close()
  })
})
