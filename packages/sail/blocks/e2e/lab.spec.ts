import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { test, expect, type Page } from '@playwright/test'
import { BLOCKS } from '../src/catalog'

const authored = BLOCKS.filter((b) => b.kind === 'block' && b.status === 'authored')

const pageErrors = (page: Page): string[] => {
  const errs: string[] = []
  page.on('pageerror', (e) => errs.push(e.message))
  return errs
}

test.describe('authored lab canvases', () => {
  for (const row of authored) {
    test(`${row.id} mounts and stays on a measure`, async ({ page }) => {
      const errs = pageErrors(page)
      const res = await page.goto(`/lab/${row.id}`, { waitUntil: 'domcontentloaded' })
      expect(res?.ok(), row.id).toBeTruthy()
      await expect(page.getByText(row.id, { exact: true }).first()).toBeVisible()

      const vp = page.viewportSize()?.width ?? 1440
      const heading = page.getByRole('heading').first()
      if ((await heading.count()) > 0) {
        const box = await heading.boundingBox()
        expect(box, row.id).toBeTruthy()
        expect(box!.width, `${row.id} heading must not span the viewport`).toBeLessThan(vp * 0.8)
      }

      expect(errs, row.id).toEqual([])
    })
  }
})

/** Catalog ids are PascalCase; authored files live at src/blocks/<kebab>/. */
const kebab = (id: string): string =>
  id.replace(/[A-Z]/g, (ch, i) => (i === 0 ? ch.toLowerCase() : `-${ch.toLowerCase()}`))

test('lab canvas mounts the block host, not a second copy', async () => {
  const root = resolve(process.cwd())
  for (const row of authored) {
    const stem = kebab(row.id)
    const blockPath = resolve(root, 'src/blocks', stem, `${stem}.sail`)
    readFileSync(blockPath, 'utf8')
    const lab = readFileSync(resolve(root, 'routes/lab', row.id, 'page.sail'), 'utf8')
    expect(lab, row.id).toContain(`<${stem}>`)
  }
})
