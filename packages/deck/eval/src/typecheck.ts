// Best-effort TypeScript type-check for a setup .ts file. We dynamic-
// import the compiler so the rig stays usable without it; when missing,
// we fall back to a coarse syntactic check (balanced braces/parens).

export interface CheckOutcome {
  ok: boolean
  error?: string
}

export async function typeCheckSetup(source: string): Promise<CheckOutcome> {
  let ts: TypeScriptModule
  try {
    ts = (await import('typescript')) as unknown as TypeScriptModule
  } catch {
    return syntaxCheckOnly(source)
  }
  const fileName = '__setup__.ts'
  const sourceFile = ts.createSourceFile(
    fileName,
    source,
    ts.ScriptTarget.ES2022,
    true,
  )
  const compilerHost: TSCompilerHost = {
    getSourceFile: (name) => (name === fileName ? sourceFile : undefined),
    writeFile: () => {},
    getDefaultLibFileName: () => 'lib.d.ts',
    useCaseSensitiveFileNames: () => true,
    getCanonicalFileName: (n) => n,
    getCurrentDirectory: () => '',
    getNewLine: () => '\n',
    fileExists: (name) => name === fileName,
    readFile: (name) => (name === fileName ? source : undefined),
    directoryExists: () => true,
    getDirectories: () => [],
  }
  const program = ts.createProgram(
    [fileName],
    {
      noEmit: true,
      strict: true,
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.ESNext,
      moduleResolution: ts.ModuleResolutionKind.Bundler,
      noResolve: true,
      types: [],
      skipLibCheck: true,
    },
    compilerHost,
  )
  const diagnostics = [
    ...program.getSyntacticDiagnostics(sourceFile),
    ...program.getSemanticDiagnostics(sourceFile),
  ]
  if (diagnostics.length === 0) return { ok: true }
  const msg = diagnostics
    .slice(0, 3)
    .map((d) =>
      typeof d.messageText === 'string'
        ? d.messageText
        : d.messageText.messageText,
    )
    .join('; ')
  return { ok: false, error: `typecheck: ${msg}` }
}

function syntaxCheckOnly(source: string): CheckOutcome {
  const stripped = source.replace(/\/\*[\s\S]*?\*\/|\/\/[^\n]*/g, '')
  if (countChar(stripped, '{') !== countChar(stripped, '}')) {
    return { ok: false, error: 'typecheck-fallback: unbalanced braces' }
  }
  if (countChar(stripped, '(') !== countChar(stripped, ')')) {
    return { ok: false, error: 'typecheck-fallback: unbalanced parens' }
  }
  return { ok: true }
}

function countChar(s: string, c: string): number {
  let n = 0
  for (const ch of s) if (ch === c) n += 1
  return n
}

interface TypeScriptModule {
  createSourceFile: (
    fileName: string,
    source: string,
    target: number,
    setParents: boolean,
  ) => unknown
  createProgram: (
    rootNames: string[],
    options: Record<string, unknown>,
    host: TSCompilerHost,
  ) => {
    getSyntacticDiagnostics: (sf: unknown) => readonly TSDiagnostic[]
    getSemanticDiagnostics: (sf: unknown) => readonly TSDiagnostic[]
  }
  ScriptTarget: { ES2022: number }
  ModuleKind: { ESNext: number }
  ModuleResolutionKind: { Bundler: number }
}

interface TSDiagnostic {
  messageText: string | { messageText: string }
}

interface TSCompilerHost {
  getSourceFile: (name: string) => unknown
  writeFile: () => void
  getDefaultLibFileName: () => string
  useCaseSensitiveFileNames: () => boolean
  getCanonicalFileName: (n: string) => string
  getCurrentDirectory: () => string
  getNewLine: () => string
  fileExists: (name: string) => boolean
  readFile: (name: string) => string | undefined
  directoryExists: () => boolean
  getDirectories: () => string[]
}
