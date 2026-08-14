import { type ExtensionContext, window, workspace } from 'vscode'
import {
  LanguageClient,
  type LanguageClientOptions,
  RevealOutputChannelOn,
  type ServerOptions,
} from 'vscode-languageclient/node'

/**
 * VS Code / Cursor client for `.sail`.
 *
 * Spawns **`sail lsp`** on stdio (same protocol as the `sail-lsp` crate).
 * Optional `sail.lsp.path` replaces the executable; the `lsp` argument is
 * always appended. The spawned argv is exactly `[<sail>, "lsp"]`.
 *
 * Highlighting is not this package — tree-sitter queries and the Studio
 * overlay own colorization.
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

  try {
    await next.start()
    client = next
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
