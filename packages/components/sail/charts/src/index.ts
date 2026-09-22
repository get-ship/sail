// @ship/sail-components/charts — SAIL chart primitives registry.
//
// Side-effect imports for the chart components. Each `<tag>.compiled.js`
// calls `customElements.define()` on import, so a single
// `import '@ship/sail-components/charts'` at the consumer registers every tag in this
// package globally.

import './sail-area-chart/sail-area-chart.compiled.js'
import './sail-bar-chart/sail-bar-chart.compiled.js'
import './sail-line-chart/sail-line-chart.compiled.js'
import './sail-sparkline/sail-sparkline.compiled.js'
import './sail-donut/sail-donut.compiled.js'
import './sail-gauge/sail-gauge.compiled.js'
import './sail-heatmap/sail-heatmap.compiled.js'
