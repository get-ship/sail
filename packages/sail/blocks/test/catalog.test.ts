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
      expect(row?.mantineUrl, id).toBeUndefined()
      expect(existsSync(resolve(ROOT, 'routes', id, 'page.sail')), id).toBe(true)
    }
  })

  it('gives every block a mantine URL and a PascalCase id', () => {
    for (const row of blocks()) {
      expect(row.kind).toBe('block')
      expect(row.mantineUrl, row.id).toMatch(/^https:\/\/ui\.mantine\.dev\/component\//)
      expect(row.id, row.id).toMatch(/^[A-Z][A-Za-z]+$/)
    }
    expect(templates().every((t) => t.kind === 'template')).toBe(true)
  })

  it('puts authored blocks at src/blocks/<Name>/<Name>.{sail,test.ts}', () => {
    const authored = blocks().filter((b) => b.status === 'authored')
    expect(authored.length).toBeGreaterThan(0)
    for (const row of authored) {
      const dir = resolve(ROOT, 'src/blocks', row.id)
      expect(existsSync(resolve(dir, `${row.id}.sail`)), row.id).toBe(true)
      expect(existsSync(resolve(dir, `${row.id}.test.ts`)), row.id).toBe(true)
      expect(blockLabPath(row.id)).toBe(`/lab/${row.id}`)
    }
  })

  it('does not put blocks under routes/', () => {
    for (const row of blocks()) {
      expect(existsSync(resolve(ROOT, 'routes', row.id, 'page.sail')), row.id).toBe(false)
    }
  })
})
