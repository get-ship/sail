import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const NOTE =
  'Structure intelligence only (parse, layout, vocabulary). TypeScript checking for controller and expressions runs in the sail CLI language server, not in this tab.'

test('structure-only TypeScript chrome is playground-only', () => {
  const playground = readFileSync(
    resolve(
      import.meta.dir,
      '../../apps/sail-site/src/sail-playground-shell/sail-playground-shell.sail',
    ),
    'utf8',
  )
  const fill = readFileSync(
    resolve(import.meta.dir, '../../apps/sail-site/src/sail-fill-shell/sail-fill-shell.sail'),
    'utf8',
  )
  const layout = readFileSync(
    resolve(import.meta.dir, '../../apps/sail-site/routes/playground/_layout.sail'),
    'utf8',
  )
  expect(playground).toContain(NOTE)
  expect(layout).toContain('sail-playground-shell')
  expect(fill).not.toContain('Structure intelligence only')
})
