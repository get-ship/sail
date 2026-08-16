import {
  type ExtensionContext,
  languages,
  SemanticTokens,
  SemanticTokensLegend,
  Uri,
  window,
  workspace,
} from 'vscode'
import {
  LanguageClient,
  type LanguageClientOptions,
  RevealOutputChannelOn,
  type ServerOptions,
} from 'vscode-languageclient/node'
import { highlight, initHighlight } from '@ship/sail-highlight'
import {
  buildSemanticTokens,
  encodeTokenData,
  LEGEND_TOKEN_MODIFIERS,
  LEGEND_TOKEN_TYPES,
} from './tokens'

/**
 * VS Code / Cursor client for `.sail`.
 *
 * Colorization: `@ship/sail-highlight` (tree-sitter-sail.wasm + highlights.scm)
 * as a DocumentSemanticTokensProvider. Not TextMate. Not the Studio overlay.
 *
 * Intelligence: **`sail lsp`** on stdio. Optional `sail.lsp.path` replaces the
 * executable; the `lsp` argument is always appended. argv is `[<sail>, "lsp"]`.
 */

let client: LanguageClient | undefined

const LSP_ARGS = ['lsp'] as const

export async function activate(context: ExtensionContext): Promise<void> {
  context.subscriptions.push(
    workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('sail.lsp.path')) {
        void restart()
      }
    }),
  )
  await registerHighlight(context)
  await start()
}

export async function deactivate(): Promise<void> {
  if (!client) return
  const running = client
  client = undefined
  await running.stop()
}

async function restart(): Promise<void> {
  await deactivate()
  await start()
}

function sailExecutable(): string {
  const raw = workspace.getConfiguration('sail').get<string>('lsp.path')
  if (typeof raw === 'string' && raw.trim() !== '') {
    return raw.trim()
  }
  return 'sail'
}

function vendorUri(context: ExtensionContext, name: string): Uri {
  return Uri.joinPath(context.extensionUri, 'vendor', name)
}

async function registerHighlight(context: ExtensionContext): Promise<void> {
  const scmUri = vendorUri(context, 'highlights.scm')
  const langUri = vendorUri(context, 'tree-sitter-sail.wasm')
  const runtimeUri = vendorUri(context, 'web-tree-sitter.wasm')
  try {
    const scm = new TextDecoder().decode(await workspace.fs.readFile(scmUri))
    const wasm = await workspace.fs.readFile(langUri)
    const runtimeWasm = await workspace.fs.readFile(runtimeUri)
    await initHighlight({ wasm, highlightsScm: scm, runtimeWasm })
  } catch (err) {
    const detail = err instanceof Error ? err.message : String(err)
    void window.showErrorMessage(
      `Sail: failed to load tree-sitter-sail highlighter (${detail}). Colors need vendor/ from bun run compile.`,
    )
    return
  }

  const legend = new SemanticTokensLegend(LEGEND_TOKEN_TYPES, LEGEND_TOKEN_MODIFIERS)
  context.subscriptions.push(
    languages.registerDocumentSemanticTokensProvider(
      { language: 'sail' },
      {
        provideDocumentSemanticTokens(document) {
          try {
            const src = document.getText()
            const spans = highlight(src)
            const data = encodeTokenData(buildSemanticTokens(src, spans))
            return new SemanticTokens(new Uint32Array(data))
          } catch {
            return new SemanticTokens(new Uint32Array())
          }
        },
      },
      legend,
    ),
  )
}

async function start(): Promise<void> {
  const command = sailExecutable()
  const args = [...LSP_ARGS]
  const cwd = workspace.workspaceFolders?.[0]?.uri.fsPath

  const serverOptions: ServerOptions = {
    command,
    args,
    options: cwd ? { cwd } : undefined,
  }

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: 'file', language: 'sail' },
      { scheme: 'untitled', language: 'sail' },
    ],
    outputChannelName: 'Sail Language Server',
    revealOutputChannelOn: RevealOutputChannelOn.Error,
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.sail'),
    },
  }

  const next = new LanguageClient(
    'sail',
    'Sail Language Server',
    serverOptions,
    clientOptions,
  )

  const missingTs =
    'Sail: TypeScript 7 native compiler not found. Controller and {expr} types are off (structure intelligence stays on). Run `sail ts-native setup`, or set SAIL_TSC to the tsc ELF (must sit next to lib.d.ts). Product path is dirname(sail)/sail-ts-native/.'
  try {
    await next.start()
    client = next
    // initializeResult.sail.typescript — hosts that swallow window/showMessage.
    const init = next.initializeResult as
      | { sail?: { typescript?: { status?: string } } }
      | undefined
    if (init?.sail?.typescript?.status !== 'live') {
      void window.showWarningMessage(missingTs)
    }
  } catch (err) {
    const detail = err instanceof Error ? err.message : String(err)
    void window.showErrorMessage(
      `Sail: failed to start \`sail lsp\` via "${command}". Put the sail CLI on PATH, or set sail.lsp.path to that binary. ${detail}`,
    )
    try {
      await next.stop()
    } catch {
      // start() failed before the process was usable
    }
  }
}
