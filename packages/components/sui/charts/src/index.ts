// @ship/sail-charts — SUI chart primitives registry.
//
// Side-effect imports for the chart components. Each `<tag>.compiled.js`
// calls `customElements.define()` on import, so a single
// `import '@ship/sail-charts'` at the consumer registers every tag in this
// pack globally.

import './sui-area-chart/sui-area-chart.compiled.js'
import './sui-bar-chart/sui-bar-chart.compiled.js'
import './sui-line-chart/sui-line-chart.compiled.js'
import './sui-sparkline/sui-sparkline.compiled.js'
import './sui-donut/sui-donut.compiled.js'
import './sui-gauge/sui-gauge.compiled.js'
import './sui-heatmap/sui-heatmap.compiled.js'
