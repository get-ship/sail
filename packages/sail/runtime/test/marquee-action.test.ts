// use:marquee — stamps fx:loop translate-x from host duration/reverse.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { marquee } from '../src/actions/marquee'
import * as motion from '../src/motion'

describe('use:marquee', () => {
  afterEach(() => {
    vi.restoreAllMocks()
    document.body.replaceChildren()
  })

  it('attaches a loop translate-x config using host duration/reverse', () => {
    const attach = vi.spyOn(motion, '__sail_motion_attach').mockReturnValue(() => {})

    const host = document.createElement('div')
    host.setAttribute('data-sail-host', 'ticker')
    host.setAttribute('duration', 'fast')
    host.setAttribute('reverse', '')
    const track = document.createElement('div')
    track.className = 'track'
    host.appendChild(track)
    document.body.appendChild(host)

    const result = marquee(track, undefined as void)
    expect(attach).toHaveBeenCalledTimes(1)
    const cfg = attach.mock.calls[0]![1] as motion.MotionAttachConfig
    expect(cfg.trigger).toBe('loop')
    expect(cfg.motions).toHaveLength(1)
    const m = cfg.motions[0]!
    expect(m.property).toBe('translate-x')
    expect(m.to).toBe('50%')
    expect(m.duration).toBe(16)
    expect(m.easing).toBe('linear')
    expect(m.direction).toBe('normal')
    expect(m.iterations).toBe(Infinity)

    result?.destroy?.()
    expect(attach).toHaveBeenCalledTimes(1)
  })

  it('re-attaches when host duration/reverse attributes change', () => {
    const dispose = vi.fn()
    const attach = vi.spyOn(motion, '__sail_motion_attach').mockReturnValue(dispose)

    const host = document.createElement('div')
    host.setAttribute('data-sail-host', 'ticker')
    const track = document.createElement('div')
    host.appendChild(track)
    document.body.appendChild(host)

    marquee(track, undefined as void)
    expect(attach).toHaveBeenCalledTimes(1)
    const first = attach.mock.calls[0]![1] as motion.MotionAttachConfig
    expect(first.motions[0]!.to).toBe('-50%')
    expect(first.motions[0]!.duration).toBe(24)

    host.setAttribute('duration', 'slow')
    // MutationObserver is async-microtask in jsdom — flush.
    return Promise.resolve().then(() => {
      expect(dispose).toHaveBeenCalled()
      expect(attach.mock.calls.length).toBeGreaterThanOrEqual(2)
      const last = attach.mock.calls.at(-1)![1] as motion.MotionAttachConfig
      expect(last.motions[0]!.duration).toBe(32)
    })
  })
})
