import { existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { BLOCKS, blockLabPath, blocks, templates } from '../src/catalog'

const ROOT = process.cwd()

describe('catalog contract', () => {
  it('marks the three pages as templates, not blocks', () => {
    for (const id of ['dashboard', 'pricing', 'settings'] as const) {
      const row = BLOCKS.find((b) => b.id === id)
      expect(row?.kind, id).toBe('template')
      expect(row?.status, id).toBe('authored')
      expect(existsSync(resolve(ROOT, 'routes', id, 'page.sail')), id).toBe(true)
    }
  })

  it('gives every block a PascalCase id', () => {
    for (const row of blocks()) {
      expect(row.kind).toBe('block')
      expect(row.id, row.id).toMatch(/^[A-Z][A-Za-z]+$/)
    }
    expect(templates().every((t) => t.kind === 'template')).toBe(true)
  })

  it('puts authored blocks at src/blocks/<kebab>/<kebab>.{sail,test.ts}', () => {
    const authored = blocks().filter((b) => b.status === 'authored')
    expect(authored.length).toBeGreaterThan(0)
    const kebab = (id: string) =>
      id
        .replace(/([a-z0-9])([A-Z])/g, '$1-$2')
        .replace(/([A-Z]+)([A-Z][a-z])/g, '$1-$2')
        .toLowerCase()
    for (const row of authored) {
      const stem = kebab(row.id)
      const dir = resolve(ROOT, 'src/blocks', stem)
      expect(existsSync(resolve(dir, `${stem}.sail`)), stem).toBe(true)
      expect(existsSync(resolve(dir, `${stem}.test.ts`)), stem).toBe(true)
      expect(existsSync(resolve(ROOT, 'routes/lab', row.id, 'page.sail')), row.id).toBe(true)
      expect(blockLabPath(row.id)).toBe(`/lab/${row.id}`)
    }
  })

  it('does not put blocks under routes/', () => {
    for (const row of blocks()) {
      expect(existsSync(resolve(ROOT, 'routes', row.id, 'page.sail')), row.id).toBe(false)
    }
  })
})
