/**
 * Type-level contract: unknown studio ops / free-form bodies must fail compile.
 * Checked by channel-typecheck.test.ts (tsc --noEmit on this file + channel.ts).
 */
import type { StudioOp, StudioMessage } from './channel'

const _close: StudioOp = { type: 'close' }
const _select: StudioOp = {
  type: 'select',
  nodeId: null,
  path: 'routes/page.sail',
  line: 1,
  col: 1,
}
const _preview: StudioOp = {
  type: 'theme-preview',
  themeId: 'sail',
  density: 'compact',
  patch: { '--radius-md': '6px' },
}
const _settings: StudioOp = { type: 'settings', fabVisible: true, open: false }
const _wire: StudioMessage = { channel: 'sail-studio', type: 'close' }
void _close
void _select
void _preview
void _settings
void _wire

// @ts-expect-error unknown channel op type
const _badType: StudioOp = { type: 'not-a-real-op' }
void _badType

// @ts-expect-error free-form Record body is not a StudioOp
const _badRecord: StudioOp = { type: 'select', arbitrary: true } as Record<string, unknown>
void _badRecord

// @ts-expect-error theme-preview requires patch
const _badPreview: StudioOp = { type: 'theme-preview', themeId: 'sail', density: 'default' }
void _badPreview

// @ts-expect-error annotate requires rect
const _badAnnotate: StudioOp = { type: 'annotate' }
void _badAnnotate
