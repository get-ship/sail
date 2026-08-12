import { describe, expect, expectTypeOf, it } from 'vitest'
import {
  STUDIO_CHANNEL,
  isStudioMessage,
  type StudioIncoming,
  type StudioMessage,
  type StudioOp,
} from './channel'
import { postStudioMessage } from './host-platform'

describe('studio channel message typing', () => {
  it('StudioOp is a discriminated union on type', () => {
    expectTypeOf<StudioOp['type']>().toEqualTypeOf<
      'select' | 'annotate' | 'screenshot' | 'settings' | 'close' | 'theme-preview'
    >()
    expectTypeOf<StudioMessage>().toMatchTypeOf<StudioIncoming>()
    expectTypeOf<StudioIncoming>().toMatchTypeOf<StudioMessage>()
  })

  it('postStudioMessage accepts known ops and rejects unknown type/fields', () => {
    expectTypeOf(postStudioMessage).parameter(0).toEqualTypeOf<StudioOp>()

    // Valid call sites (mirror real senders).
    postStudioMessage({ type: 'close' })
    postStudioMessage({
      type: 'select',
      nodeId: null,
      path: 'routes/page.sail',
      line: 1,
      col: 1,
    })
    postStudioMessage({
      type: 'theme-preview',
      themeId: 'ship',
      density: 'compact',
      patch: { '--radius-md': '6px' },
    })
    postStudioMessage({ type: 'settings', fabVisible: false })
    postStudioMessage({
      type: 'annotate',
      rect: { left: 0, top: 0, width: 10, height: 10 },
    })

    // @ts-expect-error unknown channel op type
    postStudioMessage({ type: 'not-a-real-op' })

    // @ts-expect-error free-form Record body is not allowed
    postStudioMessage({ type: 'select', arbitrary: true } as Record<string, unknown>)

    // @ts-expect-error theme-preview requires patch
    postStudioMessage({ type: 'theme-preview', themeId: 'ship', density: 'default' })
  })

  it('isStudioMessage narrows wire payloads with channel', () => {
    const raw: unknown = { channel: STUDIO_CHANNEL, type: 'close' }
    expect(isStudioMessage(raw)).toBe(true)
    if (isStudioMessage(raw)) {
      expectTypeOf(raw).toMatchTypeOf<StudioIncoming>()
      if (raw.type === 'close') {
        expect(raw.channel).toBe(STUDIO_CHANNEL)
      }
    }
    expect(isStudioMessage({ type: 'close' })).toBe(false)
    expect(isStudioMessage({ channel: 'other', type: 'close' })).toBe(false)
  })
})
