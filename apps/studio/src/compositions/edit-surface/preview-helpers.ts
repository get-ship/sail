/** Preview fetch single-flight + region paint glue for edit-surface VM. */
import { getShellState } from '../../vendor/studio-shell'
import { activeDoc, getEditState, refreshPreview } from '../../edit/index.ts'
import {
  paintRegionOverlay,
  type RegionBoxUi,
} from '../../edit/region-overlay'

export type PreviewFlight = {
  inflight: string | null
  failedPath: string | null
}

export function buildRegionBoxes(): RegionBoxUi[] {
  const es = getEditState()
  const doc = activeDoc()
  const shell = getShellState()
  const vw = shell.viewport || 1440
  const vh = Math.round((vw * 9) / 16)
  const selected = es.selectionNodeId
  const hover = es.hoverTip.visible ? es.hoverTip.nodeId : null
  return (doc?.preview.regions ?? [])
    .filter((r) => r.w > 0 && r.h > 0)
    .map((r) => ({
      id: r.nodeId,
      left: `${((r.x / vw) * 100).toFixed(2)}%`,
      top: `${((r.y / vh) * 100).toFixed(2)}%`,
      width: `${((r.w / vw) * 100).toFixed(2)}%`,
      height: `${((r.h / vh) * 100).toFixed(2)}%`,
      selected: r.nodeId === selected,
      hover: r.nodeId === hover,
      key: r.nodeId,
    }))
}

/** Region count label for the static preview chrome (empty when none). */
export function regionMetaLabel(count: number): string {
  if (count <= 0) return ''
  return count === 1 ? '1 region' : `${count} regions`
}

export function paintRegions(): RegionBoxUi[] {
  const boxes = buildRegionBoxes()
  queueMicrotask(() => paintRegionOverlay(boxes))
  return boxes
}

function shellViewport(): { width: number; height: number } {
  const w = getShellState().viewport
  return { width: w, height: Math.round((w * 9) / 16) }
}

/**
 * Bookkeep flight only when refresh actually applied (gen still current).
 * Superseded settles must not clear a newer inflight or sticky-fail early.
 */
function settleFlight(
  flight: PreviewFlight,
  path: string,
  applied: boolean,
  onDone: () => void,
): void {
  if (applied) {
    if (flight.inflight === path) flight.inflight = null
    const next = activeDoc()
    if (next?.preview.imagePng) flight.failedPath = null
    else if (getEditState().activePath === path) flight.failedPath = path
  } else if (flight.inflight === path) {
    // Superseded: keep inflight if a newer refresh still owns loading; else unstick
    // so openDocument / path change cannot leave single-flight stuck forever.
    const d = activeDoc()
    if (getEditState().activePath !== path || !d?.preview.loading) {
      flight.inflight = null
    }
  }
  onDone()
}

/**
 * Single-flight preview load; fails soft and will not storm.
 * Timeout / reject paths leave `doc.preview.error` set and `failedPath` sticky
 * until {@link retryPreviewOnce}. Uses `refreshPreview().applied` so superseded
 * settles do not clear a newer flight.
 */
export function ensurePreviewOnce(
  flight: PreviewFlight,
  onDone: () => void,
): void {
  const es = getEditState()
  const doc = activeDoc()
  if (!es.activePath || !doc) return
  if (es.view !== 'preview' && es.view !== 'layout') return
  if (doc.preview.imagePng) {
    flight.failedPath = null
    return
  }
  // Already failed for this path — UI shows error + retry; do not re-storm.
  if (flight.failedPath === es.activePath) return
  if (doc.preview.error && !doc.preview.loading) {
    flight.failedPath = es.activePath
    return
  }
  if (flight.inflight === es.activePath) return
  if (doc.preview.loading) {
    flight.inflight = es.activePath
    return
  }
  const path = es.activePath
  flight.inflight = path
  const shell = getShellState()
  void refreshPreview({
    theme: shell.themeId,
    mode: shell.modeId,
    viewport: shellViewport(),
  })
    .then((r) => settleFlight(flight, path, r.applied, onDone))
    .catch(() => settleFlight(flight, path, false, onDone))
}

/** Clear sticky failure and re-run preview (user Retry). */
export function retryPreviewOnce(
  flight: PreviewFlight,
  onDone: () => void,
): void {
  const path = getEditState().activePath
  if (!path) return
  flight.failedPath = null
  flight.inflight = null
  const doc = activeDoc()
  if (doc?.preview.error || doc?.preview.imagePng == null) {
    flight.inflight = path
    const shell = getShellState()
    void refreshPreview({
      theme: shell.themeId,
      mode: shell.modeId,
      viewport: shellViewport(),
    })
      .then((r) => settleFlight(flight, path, r.applied, onDone))
      .catch(() => settleFlight(flight, path, false, onDone))
  }
}
