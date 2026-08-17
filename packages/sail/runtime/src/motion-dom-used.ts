// Deep ESM entries only. `import { x } from 'motion-dom'` resolves the
// package barrel; Rolldown then keeps VisualElement / SVG render.
// motion-dom's package.json exports only `.`, so these are filesystem
// paths the bundler follows. Do not add a barrel import here.

// @ts-expect-error subpath is not on the package exports map
export { interpolate } from '../../../../node_modules/motion-dom/dist/es/utils/interpolate.mjs'
// @ts-expect-error subpath is not on the package exports map
export { spring } from '../../../../node_modules/motion-dom/dist/es/animation/generators/spring.mjs'
// @ts-expect-error subpath is not on the package exports map
export { generateLinearEasing } from '../../../../node_modules/motion-dom/dist/es/animation/waapi/utils/linear.mjs'
// @ts-expect-error subpath is not on the package exports map
export { calcGeneratorDuration } from '../../../../node_modules/motion-dom/dist/es/animation/generators/utils/calc-duration.mjs'
// @ts-expect-error subpath is not on the package exports map
export { frame, cancelFrame } from '../../../../node_modules/motion-dom/dist/es/frameloop/frame.mjs'
// @ts-expect-error subpath is not on the package exports map
export { motionValue } from '../../../../node_modules/motion-dom/dist/es/value/index.mjs'
// @ts-expect-error subpath is not on the package exports map
export { springValue } from '../../../../node_modules/motion-dom/dist/es/value/spring-value.mjs'

export type { SpringOptions } from 'motion-dom'
