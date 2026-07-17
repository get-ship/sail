export { initJsdomPolyfills, type JsdomPolyfillOpts } from './jsdom-polyfills'
export {
  createContainerRegistry,
  mount,
  flush,
  type TestContainerRegistry,
} from './dom-harness'
export { loadLucideIconMap, primeLucideIcons, defaultLucideIconsPath } from './lucide-icons'
export {
  renderView,
  renderComponent,
  type Rendered,
  type RenderOpts,
  type ControllerInput,
  type RenderComponentOpts,
} from './render'
export {
  runScenarios,
  type BehaviorSpec,
  type Scenario,
  type Step,
  type ScenarioResult,
} from './scenario-runner'
export {
  measureMountLeak,
  assertNoMountLeak,
  type LeakBudgetOpts,
  type LeakReport,
} from './leak-budget'
export {
  measureRetention,
  assertNoRetention,
  measureMountLeakAsync,
  forceGc,
  gcAvailable,
  type RetentionOpts,
  type RetentionFields,
  type RetentionReport,
  type RetentionCycle,
  type LeakReportWithRetention,
} from './leak-retention'
