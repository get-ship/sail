// Template `<match>`: a real compiled page remounts only the active arm.
// Inactive arms must be absent from the tree, not merely hidden.

import { test, expect } from '@playwright/test'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { compileAndBundle, pageHtml } from './compile-and-bundle'

const HERE = dirname(fileURLToPath(import.meta.url))
const FIXTURE = resolve(HERE, 'fixtures/template-match.sail')

test.describe('template match — Playwright', () => {
  test('clicking a tone remounts only that arm', async ({ page }) => {
    const bundle = compileAndBundle(FIXTURE)
    await page.setContent(pageHtml('template-match'))
    await page.addScriptTag({ content: bundle })

    const pageRoot = page.getByTestId('match-page')
    await expect(pageRoot).toBeVisible()

    await expect(page.getByTestId('arm-info')).toHaveText('Heads up.')
    await expect(page.getByTestId('arm-success')).toHaveCount(0)
    await expect(page.getByTestId('arm-else')).toHaveCount(0)

    await page.getByTestId('to-success').click()
    await expect(page.getByTestId('arm-success')).toHaveText('All good.')
    await expect(page.getByTestId('arm-info')).toHaveCount(0)
    await expect(page.getByTestId('arm-else')).toHaveCount(0)

    await page.getByTestId('to-other').click()
    await expect(page.getByTestId('arm-else')).toHaveText('Pick a tone.')
    await expect(page.getByTestId('arm-info')).toHaveCount(0)
    await expect(page.getByTestId('arm-success')).toHaveCount(0)

    await page.getByTestId('to-info').click()
    await expect(page.getByTestId('arm-info')).toHaveText('Heads up.')
    await expect(page.getByTestId('arm-success')).toHaveCount(0)
    await expect(page.getByTestId('arm-else')).toHaveCount(0)
  })
})
