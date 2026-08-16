import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

test('fill-shell workbench chrome names structure-only TypeScript', () => {
  const src = readFileSync(
    resolve(import.meta.dir, '../../apps/sail-site/src/sail-fill-shell/sail-fill-shell.sail'),
    'utf8',
  )
  expect(src).toContain(
    'Structure intelligence only (parse, layout, vocabulary). TypeScript checking for controller and expressions runs in the sail CLI language server, not in this tab.',
  )
})
