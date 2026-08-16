// Regenerate gen/runtime-surface.ts — **tier B** author runtime surface
// of @ship/sail-runtime.
//
// Walks the package root barrel (`.`) with the TypeScript compiler API, then
// keeps only names listed in packages/sail/runtime/author-surface.json.
// That JSON is the documented allowlist for intentional human imports.
//
// Tier A (template / ambient / compiler tables) is gen/language-surface.ts
// from `cargo run -p sail-compiler -- prop-registry` — not this script.
// Tier C (/internal emit ABI, host subpaths like ./framework ./server ./dev)
// is never emitted here.
//
// Regen: `bun run gen` (apps/sail-site) or:
//   node apps/sail-site/scripts/gen-runtime-surface.ts
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import * as ts from 'typescript'

const here = dirname(fileURLToPath(import.meta.url))
const runtimeDir = resolve(here, '../../../packages/sail/runtime')
const authorSurface = JSON.parse(
  readFileSync(resolve(runtimeDir, 'author-surface.json'), 'utf8'),
)
const allowlist = authorSurface.exports
const categoryDocs = authorSurface.categories ?? {}
const categoryOrder = Object.keys(categoryDocs)

const pkg = JSON.parse(readFileSync(resolve(runtimeDir, 'package.json'), 'utf8'))
const rootExport = pkg.exports?.['.']
const rootRel =
  typeof rootExport === 'string'
    ? rootExport
    : rootExport?.bun ||
      rootExport?.development ||
      rootExport?.default ||
      rootExport?.types
if (typeof rootRel !== 'string') {
  throw new Error('@ship/sail-runtime package.json exports["."] must name a source path')
}
const rootFile = resolve(runtimeDir, rootRel)

const program = ts.createProgram({
  rootNames: [rootFile],
  options: {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    strict: true,
    skipLibCheck: true,
    noEmit: true,
  },
})
const checker = program.getTypeChecker()

function kindOf(symbol) {
  const flags = symbol.getFlags()
  if (flags & ts.SymbolFlags.Class) return 'class'
  if (flags & ts.SymbolFlags.Function) return 'function'
  if (flags & ts.SymbolFlags.Interface) return 'interface'
  if (flags & ts.SymbolFlags.TypeAlias) return 'type'
  if (flags & ts.SymbolFlags.Enum) return 'enum'
  if (flags & ts.SymbolFlags.Variable) {
    const decl = symbol.valueDeclaration
    if (decl) {
      const type = checker.getTypeOfSymbolAtLocation(symbol, decl)
      if (type.getCallSignatures().length > 0) return 'function'
    }
    return 'const'
  }
  return 'value'
}

// The checker falls back to `import("<absolute-path>").Name` for a type it
// can't resolve through a visible import at the print site. The absolute
// path is worktree-local (breaks `bun run gen`'s byte-stable-regen
// contract) and useless to a reader either way — strip to the bare name.
function stripAbsoluteImports(sig) {
  return sig.replace(/import\([^)]*\)\./g, '')
}

function signatureOf(symbol, kind) {
  const decl = symbol.valueDeclaration ?? symbol.declarations?.[0]
  if (!decl) return ''
  if (kind === 'function') {
    const type = checker.getTypeOfSymbolAtLocation(symbol, decl)
    const sigs = type.getCallSignatures()
    if (sigs.length > 0) {
      return stripAbsoluteImports(
        sigs
          .map((s) => checker.signatureToString(s, decl, ts.TypeFormatFlags.NoTruncation))
          .join(' | '),
      )
    }
  }
  if (kind === 'class' || kind === 'const' || kind === 'value') {
    const type = checker.getTypeOfSymbolAtLocation(symbol, decl)
    return stripAbsoluteImports(checker.typeToString(type, decl, ts.TypeFormatFlags.NoTruncation))
  }
  // interface / type / enum — print the declaration's own text, condensed:
  // strip JSDoc/line comments first so a heavily-documented interface
  // (e.g. spring options) doesn't drown its shape in prose before the
  // 200-char clamp.
  const noComments = decl
    .getText()
    .replace(/\/\*\*[\s\S]*?\*\//g, '')
    .replace(/\/\/[^\n]*/g, '')
  return stripAbsoluteImports(noComments.replace(/\s+/g, ' ').trim().slice(0, 200))
}

function docOf(symbol) {
  const parts = symbol.getDocumentationComment(checker)
  let full = ts.displayPartsToString(parts).trim()
  if (!full) return ''
  // JSDoc wraps long lines with hard line breaks; taking only the first
  // line cuts sentences mid-thought (e.g. `__invalidate` ended with
  // "`invalidate(ref)` (see"). Return the first paragraph instead,
  // collapsing intra-paragraph line breaks into spaces.
  return full
    .replace(/\r\n/g, '\n')
    .split(/\n\s*\n/)
    .map((p) => p.replace(/\n\s*/g, ' ').trim())
    .find((p) => p.length > 0) ?? ''
}

function fileLineOf(symbol) {
  const decl = symbol.valueDeclaration ?? symbol.declarations?.[0]
  if (!decl) return ''
  const sf = decl.getSourceFile()
  const { line } = sf.getLineAndCharacterOfPosition(decl.getStart())
  // A type re-exported from a dependency (e.g. SpringOptions from
  // motion-dom) resolves to a node_modules .d.ts — its absolute path is
  // both worktree-local (breaks byte-stable regen) and not useful to a
  // reader. Name the package instead of the path.
  if (!sf.fileName.startsWith(`${runtimeDir}/`)) {
    const m = sf.fileName.match(/node_modules\/(?:\.bun\/)?(?:@[^/]+\/)?([^/@]+)/)
    return m ? `(from ${m[1]})` : '(external)'
  }
  const relPath = sf.fileName.replace(`${runtimeDir}/`, '')
  return `${relPath}:${line + 1}`
}

const sourceFile = program.getSourceFile(rootFile)
if (!sourceFile) {
  throw new Error(`could not load root barrel: ${rootFile}`)
}
const moduleSymbol = checker.getSymbolAtLocation(sourceFile)
if (!moduleSymbol) {
  throw new Error(`no module symbol for root barrel: ${rootFile}`)
}

const rootNames = new Set()
const entries = []
for (const symbol of checker.getExportsOfModule(moduleSymbol)) {
  const name = symbol.getName()
  rootNames.add(name)
  const category = allowlist[name]
  if (!category) continue
  const resolved =
    symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol
  const kind = kindOf(resolved)
  entries.push({
    subpath: '.',
    category,
    name,
    kind,
    signature: signatureOf(resolved, kind),
    doc: docOf(resolved) || docOf(symbol),
    loc: fileLineOf(resolved),
  })
}

const missing = Object.keys(allowlist).filter((n) => !rootNames.has(n))
if (missing.length > 0) {
  throw new Error(
    `author-surface.json lists exports not on the root barrel (add to index.ts or drop from allowlist):\n  ${missing.join('\n  ')}`,
  )
}

const catRank = (c) => {
  const i = categoryOrder.indexOf(c)
  return i === -1 ? 999 : i
}
entries.sort(
  (a, b) => catRank(a.category) - catRank(b.category) || a.name.localeCompare(b.name),
)

let out = ''
out += '// AUTO-GENERATED by `node scripts/gen-runtime-surface.ts`. Do not edit.\n'
out += '// Tier B author runtime of @ship/sail-runtime — root barrel filtered by\n'
out += '// packages/sail/runtime/author-surface.json. Not the full package graph\n'
out += '// (framework/server/dev/internal are tier C / host). See gen-runtime-surface.mjs.\n\n'
out += 'export interface RuntimeExportDoc {\n'
out += '  subpath: string\n'
out += '  category: string\n'
out += '  name: string\n'
out += "  kind: 'function' | 'class' | 'interface' | 'type' | 'enum' | 'const' | 'value'\n"
out += '  signature: string\n'
out += '  doc: string\n'
out += '  loc: string\n'
out += '}\n\n'
out += 'export interface RuntimeCategoryDoc {\n'
out += '  id: string\n'
out += '  summary: string\n'
out += '  count: number\n'
out += '}\n\n'
out += '/** Documented tier-B allowlist size (author-surface.json). */\n'
out += `export const AUTHOR_RUNTIME_COUNT = ${entries.length}\n\n`
out += 'export const RUNTIME_CATEGORIES: RuntimeCategoryDoc[] = '
const categories = categoryOrder.map((id) => ({
  id,
  summary: categoryDocs[id] ?? id,
  count: entries.filter((e) => e.category === id).length,
}))
out += JSON.stringify(categories, null, 2)
out += '\n\n'
out += 'export const RUNTIME_EXPORTS: RuntimeExportDoc[] = '
out += JSON.stringify(entries, null, 2)
out += '\n'

const dest = resolve(here, '../gen/runtime-surface.ts')
const args = process.argv.slice(2)
const check = args.includes('--check')
const outFlag = args.indexOf('--out')
const destWrite =
  outFlag >= 0 && args[outFlag + 1]
    ? resolve(args[outFlag + 1]!)
    : dest

if (check) {
  // apps/sail-site/gen/runtime-surface.ts is gitignored. Absent dest is stale.
  const existing = existsSync(destWrite) ? readFileSync(destWrite, 'utf8') : null
  if (existing !== out) {
    const named =
      destWrite === dest ? 'apps/sail-site/gen/runtime-surface.ts' : destWrite
    console.error(
      `gen-runtime-surface --check: ${named} stale\n` +
        '  regenerate: bun apps/sail-site/scripts/gen-runtime-surface.ts',
    )
    process.exit(1)
  }
  console.log(`ok: runtime-surface (${entries.length} exports)`)
} else {
  mkdirSync(resolve(destWrite, '..'), { recursive: true })
  writeFileSync(destWrite, out)
  console.log(
    `wrote ${destWrite} (${entries.length} tier-B exports across ${categories.length} categories; root has ${rootNames.size} total names)`,
  )
}
