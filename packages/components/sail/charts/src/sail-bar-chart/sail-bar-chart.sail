<!--
  <sail-bar-chart> — vertical or horizontal categorical bar chart, SVG-rendered.

  MIGRATED onto the createChartData engine + createChartViewport adapter: the
  comma-string `data` decodes through `normalizeSeries`, bars are a pure
  `derived` (present at first paint), the ad-hoc `isEmpty` signal is replaced by
  the engine discriminator composing `sail-empty-state`, loading composes
  `sail-skeleton`, and the `max=0`→auto domain convention is preserved by the
  engine extent. The viewBox tracks the measured box.

  Props
  ─────────────────────────────────────────────────────────────────────────────
  data         string  reflect    comma-separated numbers (normalizeSeries).
  labels       string             used as <title> per bar.
  tone         union   reflect    series color (toneColor role-token map).
  orientation  union   reflect    vertical | horizontal.
  width        number  300        initial estimate — measured box drives render.
  height       number  200        same.
  max          number  0          0 = auto (engine extent yMax-null semantics).
  label        string  "bar chart"  aria-label kind (prefixes the data summary).
  loading      bool    reflect    loading branch (engine isLoading) → sail-skeleton.
  error        —       error identity-only → sail-error-state.
  ─────────────────────────────────────────────────────────────────────────────
  Slots: description (described-data override)

  The SVG is the component's box: full-bleed width, `height` px tall. Bars rest
  at 85% fill so a dense chart reads as one mass rather than a row of shouting
  blocks.
-->

<scope>
  props: {
    data = ''
    labels = ''
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    orientation: 'vertical' | 'horizontal' = 'vertical'
    width = 300
    height = 200
    max = 0
    label = 'bar chart'
    loading = false
  }
</scope>

<controller>

import type { Action } from '@ship/sail-runtime'
import { createChartData } from '../engine/create-chart-data'
import { createChartViewport } from '../engine/create-chart-viewport'
import { normalizeSeries } from '../engine/normalize'
import { barGeometry } from '../engine/geometry-cartesian'
import { parseLabels } from '../chart-util'
import type { Scope } from './sail-bar-chart.sail'

// `error` arrives via `error={…}` identity (no decodable <prop> for
// `unknown`); read off the host in `mount` and patch the engine.
interface BarHost extends Element {
  error?: unknown
}

export default controller(({ data, labels, tone, orientation, max, width, height, label, loading }: Scope) => {
    const viewport = createChartViewport({
      initialWidth: width(),
      initialHeight: height(),
    })
    const chart = createChartData({
      series: derived(() => normalizeSeries(data())),
      // Bars grow from a zero baseline; `max>0` caps the y-domain, `0` = auto
      // (engine yMax-null) — reactive so changing `max` re-resolves the domain.
      yMin: 0,
      yMax: derived(() => (max() > 0 ? max() : null)),
      isLoading: loading,
    })

    // GEOMETRY IS A PURE DERIVED — present at first paint, no effect-into-signal.
    const bars = derived(() =>
      barGeometry(chart.normalized(), { w: viewport.width(), h: viewport.height() }, {
        horizontal: orientation() === 'horizontal',
        tone: tone(),
        labels: parseLabels(labels()),
      }),
    )
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
      const err = (host as BarHost).error
      if (err !== undefined) chart.setOptions({ error: err })
    })

    return { chart, viewport, observeViewport, bars, ariaLabel }
  })

</controller>
<if {loading()}>
  <skeleton shape="block" size="lg" role="status" aria-label="Loading" aria-live="polite" aria-busy="true"></skeleton>
{else-if {chart.error()} => <error-state></error-state>}{else-if {chart.isEmpty()} => <empty-state></empty-state>}{else => <svg
    viewBox={'0 0 ' + viewport.width() + ' ' + viewport.height()}
    height={height()}
    role="img"
    aria-label={ariaLabel()}
    aria-describedby="chart-data"

    use:observeViewport style({ box: display(block).w(full).overflow(hidden) })>
    <for {bars() as b} key={b.id}>
      <rect x={b.x} y={b.y} width={b.w} height={b.h} rx={b.rx} ry={b.ry} fill={b.color} fill-opacity="0.85">
        <title>{b.title}</title>
      </rect>
    </for>
  </svg>
  <column id="chart-data" style({ paint: srOnly() })>
    <slot name="description">
      <table>
        <for {chart.ariaTable() as row} key={row[0].id}>
          <tr>
            <for {row as cell} key={cell.id}>
              <td>{cell.text}</td>
            </for>
          </tr>
        </for>
      </table>
    </slot>
  </column>}</if>
