// Regenerate src/runtime-surface.gen.ts — the COMPLETE public export surface
// of @ship/sail-runtime, walked mechanically off the TypeScript compiler API
// against the package's own `exports` map (packages/sail/runtime/package.json).
// One entry point's module symbol, `checker.getExportsOfModule`, per subpath —
// so a new export (or a new subpath) shows up here on the next `bun run gen`
// with no hand-editing. This is the runtime's sibling to
// `cargo run -p sail-compiler -- prop-registry` (the compiler's own surface):
// same law (generated-from-tables, never hand-listed), different source
// language, because the runtime lives in TS and the compiler lives in Rust.
//
//   node apps/sail-site/scripts/gen-runtime-surface.mjs
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import * as ts from 'typescript'

const here = dirname(fileURLToPath(import.meta.url))
const runtimeDir = resolve(here, '../../../packages/sail/runtime')
const pkg = JSON.parse(readFileSync(resolve(runtimeDir, 'package.json'), 'utf8'))
const exportsMap = pkg.exports

const entryFiles = Object.entries(exportsMap).map(([subpath, rel]) => [
  subpath,
  resolve(runtimeDir, rel),
])

const program = ts.createProgram({
  rootNames: entryFiles.map(([, file]) => file),
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

// `./internal` is a total re-export of the root barrel (`export * from
// './index'`) — the compiler's ABI door, not a distinct surface. Documenting
// it would duplicate every root-barrel row under a second subpath for zero
// new information; skip its rows, note its existence separately in the page.
const entries = []
for (const [subpath, file] of entryFiles) {
  if (subpath === './internal') continue
  const sourceFile = program.getSourceFile(file)
  if (!sourceFile) continue
  const moduleSymbol = checker.getSymbolAtLocation(sourceFile)
  if (!moduleSymbol) continue
  const exported = checker.getExportsOfModule(moduleSymbol)
  for (const symbol of exported) {
    const resolved =
      symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol
    const kind = kindOf(resolved)
    entries.push({
      subpath,
      name: symbol.getName(),
      kind,
      signature: signatureOf(resolved, kind),
      doc: docOf(resolved) || docOf(symbol),
      loc: fileLineOf(resolved),
    })
  }
}

entries.sort((a, b) => a.subpath.localeCompare(b.subpath) || a.name.localeCompare(b.name))

let out = ''
out += '// AUTO-GENERATED by `node scripts/gen-runtime-surface.mjs`. Do not edit.\n'
out +=
  '// The complete public export surface of @ship/sail-runtime, walked off the\n'
out += '// TypeScript compiler API against the package.json `exports` map — every\n'
out += '// subpath, every export, mechanically. See scripts/gen-runtime-surface.mjs.\n\n'
out += 'export interface RuntimeExportDoc {\n'
out += '  subpath: string\n'
out += "  name: string\n"
out += "  kind: 'function' | 'class' | 'interface' | 'type' | 'enum' | 'const' | 'value'\n"
out += '  signature: string\n'
out += '  doc: string\n'
out += '  loc: string\n'
out += '}\n\n'
out += 'export const RUNTIME_EXPORTS: RuntimeExportDoc[] = '
out += JSON.stringify(entries, null, 2)
out += '\n'

const dest = resolve(here, '../src/runtime-surface.gen.ts')
writeFileSync(dest, out)
console.log(
  `wrote ${dest} (${entries.length} exports across ${entryFiles.length} entry points)`,
)
