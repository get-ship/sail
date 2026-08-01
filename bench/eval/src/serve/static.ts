/**
 * One scoped static server over surfaces + run tree.
 * No raw dump of the entire home dir — only $SAIL_LAB.
 */

import { createServer, type IncomingMessage, type ServerResponse } from 'node:http'
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs'
import { extname, join, resolve } from 'node:path'
import { labRoot } from '../naming/paths.ts'

const MIME: Record<string, string> = {
  '.html': 'text/html; charset=utf-8',
  '.md': 'text/markdown; charset=utf-8',
  '.json': 'application/json',
  '.jsonl': 'text/plain; charset=utf-8',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.svg': 'image/svg+xml',
  '.css': 'text/css',
  '.js': 'text/javascript',
  '.sail': 'text/plain; charset=utf-8',
  '.txt': 'text/plain; charset=utf-8',
}

export interface ServeOptions {
  lab?: string
  port?: number
  host?: string
}

export async function serveLab(opts: ServeOptions = {}): Promise<{ port: number; close: () => Promise<void> }> {
  const lab = resolve(opts.lab ?? labRoot())
  const host = opts.host ?? '127.0.0.1'
  const port = opts.port ?? 0

  const server = createServer((req, res) => {
    try {
      handle(lab, req, res)
    } catch (e) {
      res.writeHead(500)
      res.end(e instanceof Error ? e.message : String(e))
    }
  })

  await new Promise<void>((resolvePromise, reject) => {
    server.listen(port, host, () => resolvePromise())
    server.on('error', reject)
  })

  const addr = server.address()
  const bound = addr && typeof addr === 'object' ? addr.port : port

  return {
    port: bound,
    close: () =>
      new Promise((resolvePromise) => {
        server.close(() => resolvePromise())
      }),
  }
}

function handle(lab: string, req: IncomingMessage, res: ServerResponse): void {
  const url = (req.url ?? '/').split('?')[0] || '/'
  if (url === '/' || url === '') {
    res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' })
    res.end(homePage(lab))
    return
  }

  const rel = decodeURIComponent(url).replace(/^\/+/, '')
  const abs = resolve(join(lab, rel))
  // Path traversal guard
  if (!abs.startsWith(lab)) {
    res.writeHead(403)
    res.end('forbidden')
    return
  }
  if (!existsSync(abs)) {
    res.writeHead(404)
    res.end('not found')
    return
  }
  const st = statSync(abs)
  if (st.isDirectory()) {
    res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' })
    res.end(dirListing(rel, abs))
    return
  }
  const ext = extname(abs)
  res.writeHead(200, { 'Content-Type': MIME[ext] ?? 'application/octet-stream' })
  res.end(readFileSync(abs))
}

function homePage(lab: string): string {
  return `<!doctype html><html><head><meta charset="utf-8"><title>sail-bench lab</title></head>
<body style="font-family:system-ui;max-width:40rem;margin:2rem auto">
<h1>sail-bench lab server</h1>
<p>Scoped to <code>${lab}</code></p>
<ul>
  <li><a href="/reports/">reports/</a></li>
  <li><a href="/runs/">runs/</a></li>
</ul>
</body></html>`
}

function dirListing(rel: string, abs: string): string {
  const entries = readdirSync(abs)
  const items = entries
    .map((e: string) => {
      const href = '/' + (rel ? rel.replace(/\/?$/, '/') : '') + e
      return `<li><a href="${href}">${e}</a></li>`
    })
    .join('\n')
  return `<!doctype html><html><body><h1>/${rel}</h1><ul>${items}</ul></body></html>`
}
