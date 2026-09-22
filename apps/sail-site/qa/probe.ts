#!/usr/bin/env bun
// Moved. L1 × theme matrix is not a docs-site test.
// SSOT: crates/sail/compiler/tests/l1-matrix/probe.ts
// Fixture: crates/sail/compiler/tests/fixtures/l1-matrix
console.error(`sail-site/qa/probe.ts is retired.
L1 theme matrix lives on an owned fixture, not /components/:tag.

  bun crates/sail/compiler/tests/l1-matrix/probe.ts
  SAIL_PROBE_BASE=http://127.0.0.1:7892 …

Theme cells come from generated THEME_META. The site is a product gauge.`)
process.exit(2)
