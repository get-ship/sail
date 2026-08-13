// Build &amp; deploy examples for the deck Build page.

export const buildCmd = `$ sail build --project . --bundle-framework --base assets --out dist`

export const buildOut = `dist/
  index.html                      CSR SPA shell (importmap + #root)
  _deck/{runtime,dom,route}.js    framework singletons, materialized
  _deck/icons/{name}.svg          used glyphs (no CDN)
  assets/entry-<hash>.js          SPA entry + lazy route chunks
  sail_compiler_bg.wasm           playground / tutorial compiler
  _headers  _redirects            Cloudflare Pages MIME + SPA fallback`

export const baseCmd = `$ sail build --project . --bundle-framework --base assets --out dist
  ▸ asset URLs become /assets/<file> — matches on-disk layout`

export const cfDeploy = `$ bun run deploy:pages
  ▸ wrangler pages deploy dist --project-name sail-site`
