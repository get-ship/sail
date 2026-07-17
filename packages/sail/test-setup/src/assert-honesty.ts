// Shared honesty guard for every SUI component vitest run.
//
// `expect.hasAssertions()` makes any `it()` whose executed path runs ZERO
// assertions FAIL instead of passing green. A test that mounts a component,
// drives an interaction, and then asserts nothing (or early-returns past its
// only `expect`) is not a passing test — it is a hole. This converts those
// holes to red.
//
// Wired through `suiComponentVitestConfig()` as the first setup file so it
// applies to every component package uniformly, regardless of whether the
// package authored its own `test/setup.ts`.

import { beforeEach, expect } from 'vitest'

beforeEach(() => {
  expect.hasAssertions()
})
