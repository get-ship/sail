// Minimal static file server for a `deck build` output dir — the PROD path the
// harness asserts (no `deck dev`, no host: just the built bytes over HTTP, the
// way a user's static host would serve them). Zero deps (Node stdlib only).
//
// Usage: node serve-built.mjs <build-dir> <port>
//
// SPA fallback: a request path with no file extension is a client route, so it
// serves index.html and lets the deck router resolve it client-side (mirrors a
// static host configured for an SPA). Real assets (/_deck/*, /assets/*, *.css,
// *.js) are served by path with the right content-type so ESM module loading
// and the importmap resolve exactly as in production.
import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { join, extname, normalize } from 'node:path'

const dir = process.argv[2]
const port = Number(process.argv[3])
if (!dir || !port) {
  console.error('usage: node serve-built.mjs <build-dir> <port>')
  process.exit(1)
}

const TYPES = {
  '.js': 'application/javascript; charset=utf-8',
  '.mjs': 'application/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
  '.map': 'application/json; charset=utf-8',
}

const sendIndex = async (res) => {
  try {
    const body = await readFile(join(dir, 'index.html'))
    res.writeHead(200, { 'content-type': TYPES['.html'] }).end(body)
  } catch {
    res.writeHead(404).end('no index.html in build output')
  }
}

createServer(async (req, res) => {
  const url = new URL(req.url, 'http://localhost')
  const path = decodeURIComponent(url.pathname)
  const ext = extname(path)
  // Bare path (no extension) → SPA client route → index.html.
  if (path === '/' || ext === '') return sendIndex(res)
  // Traversal-guard then serve the file by path. With `deck build --base assets`
  // the emitted asset URLs (`/assets/<file>`, `/_deck/<file>`) MATCH the on-disk
  // layout, so a plain by-path lookup serves the whole build — no URL rewriting,
  // exactly as a dumb static host (Netlify/S3/nginx) would.
  const rel = normalize(path).replace(/^(\.\.([/\\]|$))+/, '')
  const ct = TYPES[ext] ?? 'application/octet-stream'
  try {
    const body = await readFile(join(dir, rel))
    res.writeHead(200, { 'content-type': ct }).end(body)
  } catch {
    res.writeHead(404).end(`not found: ${path}`)
  }
}).listen(port, '127.0.0.1', () => {
  console.log(`deck prod static serve: ${dir} on http://127.0.0.1:${port}`)
})
