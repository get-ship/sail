#!/usr/bin/env node
// Dumb static host for a `sail build --base assets --bundle-framework` dir.
// Same contract as Cloudflare Pages + packages/deck/e2e/serve-built.mjs,
// with MIME types Pages needs (wasm / woff2 / webp) so playground compile
// and font faces work offline. SPA fallback: no-extension paths → index.html.
import { createServer } from 'node:http'
import { readFile, stat } from 'node:fs/promises'
import { join, extname, normalize, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

export const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.mjs': 'application/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.map': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.webp': 'image/webp',
  '.gif': 'image/gif',
  '.wasm': 'application/wasm',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
  '.ttf': 'font/ttf',
  '.txt': 'text/plain; charset=utf-8',
}

export function createStaticServer(dir, { host = '127.0.0.1', port } = {}) {
  const root = resolve(dir)

  const sendIndex = async (res) => {
    try {
      const body = await readFile(join(root, 'index.html'))
      res.writeHead(200, { 'content-type': TYPES['.html'] }).end(body)
    } catch {
      res.writeHead(404).end('no index.html in build output')
    }
  }

  const server = createServer(async (req, res) => {
    const url = new URL(req.url ?? '/', `http://${host}`)
    const path = decodeURIComponent(url.pathname)
    const ext = extname(path)
    if (path === '/' || ext === '') return sendIndex(res)
    const rel = normalize(path).replace(/^(\.\.([/\\]|$))+/, '').replace(/^[/\\]+/, '')
    const file = join(root, rel)
    if (!file.startsWith(root)) {
      res.writeHead(403).end('forbidden')
      return
    }
    try {
      const st = await stat(file)
      if (st.isDirectory()) return sendIndex(res)
      const body = await readFile(file)
      res.writeHead(200, { 'content-type': TYPES[ext] ?? 'application/octet-stream' }).end(body)
    } catch {
      res.writeHead(404).end(`not found: ${path}`)
    }
  })

  return new Promise((resolveStart) => {
    server.listen(port, host, () => {
      const addr = server.address()
      const actual = typeof addr === 'object' && addr ? addr.port : port
      resolveStart({
        server,
        port: actual,
        url: `http://${host}:${actual}`,
        close: () =>
          new Promise((r, j) => server.close((err) => (err ? j(err) : r()))),
      })
    })
  })
}

const isMain = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)
if (isMain) {
  const dir = process.argv[2]
  const port = Number(process.argv[3] ?? 4173)
  if (!dir || !port) {
    console.error('usage: node static-serve.mjs <build-dir> [port]')
    process.exit(1)
  }
  const handle = await createStaticServer(dir, { port })
  console.log(`sail-site static serve: ${dir} on ${handle.url}`)
}
