import { describe, expect, test } from 'bun:test'
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

const root = join(import.meta.dir, '..')

function read(rel: string): string {
  return readFileSync(join(root, rel), 'utf8')
}

function walk(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    if (name === 'node_modules' || name === 'out' || name === '.git' || name === 'vendor') {
      continue
    }
    const p = join(dir, name)
    if (statSync(p).isDirectory()) walk(p, out)
    else out.push(p)
  }
  return out
}

describe('packages/sail/vscode', () => {
  const pkg = JSON.parse(read('package.json')) as {
    private?: boolean
    main?: string
    activationEvents?: string[]
    scripts?: Record<string, string>
    dependencies?: Record<string, string>
    contributes?: {
      languages?: Array<{ id?: string; extensions?: string[] }>
      configuration?: { properties?: Record<string, unknown> }
      grammars?: unknown
    }
  }
  const readme = read('README.md')
  const src = read('src/extension.ts')

  test('private local package — no marketplace publish script', () => {
    expect(pkg.private).toBe(true)
    const scripts = JSON.stringify(pkg.scripts ?? {})
    expect(scripts).not.toContain('vsce publish')
    expect(scripts).not.toContain('ovsx publish')
  })

  test('contributes language sail for *.sail', () => {
    const lang = pkg.contributes?.languages?.[0]
    expect(lang?.id).toBe('sail')
    expect(lang?.extensions).toContain('.sail')
  })

  test('contributes sail.lsp.path and activation on language sail', () => {
    expect(pkg.activationEvents).toContain('onLanguage:sail')
    expect(pkg.contributes?.configuration?.properties?.['sail.lsp.path']).toBeTruthy()
    expect(pkg.main).toBe('./out/extension.js')
  })

  test('README names the exact sail lsp command', () => {
    expect(readme).toContain('sail lsp')
    expect(readme).toContain('sail-ts-native/')
    expect(pkg.description ?? '').toContain('sail-ts-native/')
    expect(readme).toContain('Install from VSIX')
    expect(readme).toContain('Install from Location')
  })

  test('client inspects initializeResult.sail.typescript', () => {
    expect(src).toContain('initializeResult')
    expect(src).toContain('sail.typescript')
    expect(src).toContain('showWarningMessage')
  })

  test('client starts sail lsp over stdio — never cargo run', () => {
    expect(src).toContain("['lsp']")
    expect(src).toContain('sail lsp')
    expect(src).toContain("return 'sail'")
    expect(src).not.toMatch(/['"]cargo['"]/)
    expect(src).not.toMatch(/\bcargo\b/)
  })

  test('colorization is @ship/sail-highlight semantic tokens — no TextMate', () => {
    expect(pkg.contributes?.grammars).toBeUndefined()
    expect(existsSync(join(root, 'syntaxes'))).toBe(false)
    expect(pkg.dependencies?.['@ship/sail-highlight']).toBeTruthy()
    expect(src).toContain('@ship/sail-highlight')
    expect(src).toContain('registerDocumentSemanticTokensProvider')
    expect(readme).toContain('@ship/sail-highlight')
    expect(readme.toLowerCase()).toContain('tree-sitter')
    expect(readme.toLowerCase()).toContain('semantic')
    expect(pkg.scripts?.compile ?? '').toContain('vendor')
  })

  test('vendored highlights.scm bytes equal grammar queries', () => {
    const vendored = join(root, 'vendor/highlights.scm')
    const source = join(root, '../grammar/queries/highlights.scm')
    expect(existsSync(vendored), 'vendor/highlights.scm — bun run vendor').toBe(true)
    expect(readFileSync(vendored)).toEqual(readFileSync(source))
  })

  test('no retired prefixes in client/config (style: / {#if} / let:)', () => {
    const retired = [
      'style:',
      'let:',
      'class:',
      'prop:',
      'attr:',
      'transition:',
      '{#if}',
      '{/if}',
      '{:else}',
    ]
    const skip = new Set(['README.md', 'package.test.ts'])
    for (const file of walk(root)) {
      if (skip.has(file.split('/').pop() ?? '')) continue
      const rel = relative(root, file)
      const text = readFileSync(file, 'utf8')
      for (const token of retired) {
        expect(text.includes(token), `${rel} must not teach ${token}`).toBe(false)
      }
    }
  })
})
