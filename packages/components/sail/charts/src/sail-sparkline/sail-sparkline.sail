<!--
  <sail-sparkline> — compact inline trend line, SVG-rendered.

  MIGRATED onto the createChartData engine + createChartViewport adapter: the
  comma-string `data` decodes through `normalizeSeries`, geometry is a pure
  `derived` (present at first paint), and a null comma slot splits the stroke per
  the gap contract. Sparkline is the ONE lightweight presentation exception —
  loading composes `sail-spinner` (skeleton is too heavy inline) and empty stays
  the `aria-hidden` empty SVG (no full-size `sail-empty-state`).

  Props
  ─────────────────────────────────────────────────────────────────────────────
  data     string  reflect   comma-separated numbers (normalizeSeries); series
                             identity supersedes (normalizeSeries handles both).
  tone     union   reflect   line color (toneColor role-token map).
  width    number  80        initial estimate — measured box drives the render.
  height   number  20        same.
  fill     bool    reflect   fill the area under the line at 18% alpha.
  label    string  "sparkline"  aria-label kind (prefixes the data summary).
  loading  bool    reflect   loading branch (engine isLoading) → sail-spinner.
  ─────────────────────────────────────────────────────────────────────────────
  Slots: none

  The SVG is the component's box: `width`/`height` size it, `display:inline-block`
  keeps it inline with the text it trends alongside, and `overflow:visible` keeps
  the round stroke caps at the extremes from clipping on the viewBox edge.
-->

<scope>
  props: {
    data = ''
    tone: 'primary' | 'default' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    width = 80
    height = 20
    fill = false
    label = 'sparkline'
    loading = false
  }
</scope>

<controller>

import type { Action } from '@ship/sail-runtime'
import { createChartData } from '../engine/create-chart-data'
import { createChartViewport } from '../engine/create-chart-viewport'
import { normalizeSeries } from '../engine/normalize'
import { areaGeometry } from '../engine/geometry-cartesian'
import { toneColor } from '../chart-util'
import type { Scope } from './sail-sparkline.sail'

// `error` arrives via `error={…}` identity (no decodable <prop> for
// `unknown`); read off the host in `mount` and patch the engine.
interface SparkHost extends Element {
  error?: unknown
}

export default controller(({ data, tone, width, height, label, loading }: Scope) => {
    const viewport = createChartViewport({
      initialWidth: width(),
      initialHeight: height(),
    })
    const chart = createChartData({
      series: derived(() => normalizeSeries(data())),
      isLoading: loading,
    })

    // GEOMETRY IS A PURE DERIVED — areaGeometry yields both fillD + strokeD so
    // one derivation serves the fill-on and fill-off template branches. The
    // single series' tone overrides the comma-string default (toneColor map).
    const seriesPaths = derived(() => {
      const t = tone()
      return areaGeometry(chart.normalized(), {
        w: viewport.width(),
        h: viewport.height(),
      }).map((p) => ({ ...p, color: toneColor(t) }))
    })
    const ariaLabel = derived(() => `${label()}: ${chart.ariaSummary()}`)

    // The <svg> lives behind {#if}, so it mounts and unmounts as the chart
    // moves between loading/error/empty and data. The action runs on each
    // mount and disposes on each unmount — `observe()` builds a fresh
    // observer per call, so the pair is re-entrant.
    const observeViewport: Action<void> = (node) => {
      viewport.observe(node)
      return { destroy: () => viewport.dispose() }
    }

    mount((host) => {
      const err = (host as SparkHost).error
      if (err !== undefined) chart.setOptions({ error: err })
    })

    return { chart, viewport, observeViewport, seriesPaths, ariaLabel }
  })

</controller>
<if {loading()}>
  <spinner></spinner>
<else-if {chart.isEmpty()}>
  <svg
    viewBox={'0 0 ' + viewport.width() + ' ' + viewport.height()}
    width={viewport.width()}
    height={viewport.height()}
    aria-hidden="true"
>
    style.display(inline-block).overflow(visible)

  </svg>
</else-if><else>
  <svg
    viewBox={'0 0 ' + viewport.width() + ' ' + viewport.height()}
    width={viewport.width()}
    height={viewport.height()}
    role="img"
    aria-label={ariaLabel()}

    use:observeViewport>
    style.display(inline-block).overflow(visible)

    <for {seriesPaths()} as="s" key={s.name}>
      <if {fill()}>
        <path d={s.fillD} fill={s.color} fill-opacity="0.18" stroke="none"></path>
      </if>
      <path d={s.strokeD} fill="none" stroke={s.color} stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      </path>
    </for>
  </svg>
</else></if>
