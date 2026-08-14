// SPEC-TEST-2: compile → DOM, query only through @ship/sail-test (no data-* / class).

import { test, expect } from '@playwright/test'
import { getByContract } from '@ship/sail-test/contract'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { compileAndBundle, pageHtml } from './compile-and-bundle'

const HERE = dirname(fileURLToPath(import.meta.url))
const FIXTURE = resolve(HERE, 'fixtures/sail-test-semantic.sail')

test.describe('sail-test semantic contract', () => {
  test('button is found by role and name only', async ({ page }) => {
    const bundle = compileAndBundle(FIXTURE)
    await page.setContent(pageHtml('sail-test-semantic'))
    await page.addScriptTag({ content: bundle })

    const save = getByContract(page, { role: 'button', name: 'Save' })
    await expect(save).toBeVisible()
    await save.click()
    await expect(page.getByText('count 1')).toBeVisible()
  })
})
