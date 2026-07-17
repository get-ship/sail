#!/usr/bin/env node
// Lit ↔ SAIL button parity — disabled.
//
// Mixing Lit (Ship product) into the framework fixtures workspace breaks
// the layering this directory was created to enforce. The right home for
// parity tests is at the Ship-host level where both component sets
// legitimately coexist — `apps/shell/public/parity-probe.html` (which
// requires shell's vite dev server) already provides an importmap-
// bearing host page and an existing `.artifacts/probes/probe-parity-*`
// pattern.
//
// Move parity tests there. This stub exists so the framework E2E dir
// doesn't claim parity coverage it doesn't have.

console.log('parity-button: skipped (see file header).')
process.exit(0)
