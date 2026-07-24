#!/usr/bin/env node
/**
 * DTCG (grouped) → tokens.css + themes.css + token-registry.json
 *
 * SSOT is packages/sail/tokens/model/**.json (Design Tokens Format).
 * Generated files under src/ must not be hand-edited.
 *
 * Emit strategy for aliases: DTCG `{token-name}` or `{group.path}` →
 * `var(--css-name)` so theme override friendliness (var chains) is preserved.
 *
 * Public CSS custom property names are stable via per-leaf
 * `$extensions.ship.cssName` (compat map path → --name).
 *
 * No GPL tooling required — pure Node, zero deps.
 */
import { readdirSync, readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const PKG = resolve(__dirname, '..')
const MODEL = join(PKG, 'model')
const THEMES_DIR = join(MODEL, 'themes')
const SRC = join(PKG, 'src')

const GENERATED_HEADER = `\
/*
 * GENERATED FILE — do not edit by hand.
 *
 * Source of truth: packages/sail/tokens/model/** (DTCG groups + themes).
 * Regenerate: pnpm --filter @ship/tokens generate
 *            (or: node packages/sail/tokens/scripts/generate.mjs)
 *
 * Token layers, in order of abstraction (highest first):
 *   1. Semantic surfaces  — --surface-0..3, --backdrop
 *   2. Tone ramps         — --tone-{name}-{fg|bg|bg-solid|border|ring}
 *   3. Brand palette      — --primary, --accent, --foreground, etc.
 *   4. Primitives         — spacing, radii, typography scale, motion curves
 *
 * Components should read the HIGHEST layer that fits.
 * Themes: model/themes/*.json → themes.css (plus model/themes-chrome.css).
 */
`

const THEMES_HEADER = `\
/*
 * GENERATED FILE — do not edit by hand.
 *
 * Theme token overrides from packages/sail/tokens/model/themes/*.json.
 * Structural flourishes: packages/sail/tokens/model/themes-chrome.css
 * Regenerate: pnpm --filter @ship/tokens generate
 */
`

// ─── DTCG walk ───────────────────────────────────────────────────────

function isTokenLeaf(node) {
  return node && typeof node === 'object' && !Array.isArray(node) && Object.prototype.hasOwnProperty.call(node, '$value')
}

function isGroup(node) {
  return node && typeof node === 'object' && !Array.isArray(node) && !Object.prototype.hasOwnProperty.call(node, '$value')
}

/**
 * Flatten a DTCG document (nested groups or flat leaves) into ordered tokens.
 * Each token: { path, cssName, type, value, description }
 */
function flattenTokens(doc, pathPrefix = [], inheritedType = undefined) {
  const out = []
  for (const [key, node] of Object.entries(doc)) {
    if (key.startsWith('$')) continue
    if (!node || typeof node !== 'object' || Array.isArray(node)) continue
    if (isTokenLeaf(node)) {
      const path = [...pathPrefix, key].join('.')
      const cssName =
        (typeof node.$extensions?.ship?.cssName === 'string' && node.$extensions.ship.cssName) ||
        [...pathPrefix, key].join('-')
      const type =
        (typeof node.$type === 'string' && node.$type) ||
        inheritedType ||
        'string'
      const value = node.$value
      out.push({
        path,
        cssName,
        type,
        value: typeof value === 'string' ? value : String(value),
        description: typeof node.$description === 'string' ? node.$description : undefined,
      })
    } else if (isGroup(node)) {
      const groupType = typeof node.$type === 'string' ? node.$type : inheritedType
      out.push(...flattenTokens(node, [...pathPrefix, key], groupType))
    }
  }
  return out
}

/** Build path → cssName and cssName → path indexes from a token list. */
function buildNameIndex(tokens) {
  const pathToCss = new Map()
  const cssToPath = new Map()
  for (const t of tokens) {
    pathToCss.set(t.path, t.cssName)
    cssToPath.set(t.cssName, t.path)
  }
  return { pathToCss, cssToPath }
}

/**
 * Resolve a DTCG $value to a CSS value.
 * Aliases: `{token-name}` or `{group.path}` → var(--cssName)
 */
function resolveValue(value, nameIndex) {
  if (typeof value !== 'string') {
    throw new Error(`Token $value must be a string, got ${typeof value}`)
  }
  const alias = value.match(/^\{([a-zA-Z0-9_.-]+)\}$/)
  if (alias) {
    const ref = alias[1]
    let cssName
    if (nameIndex.pathToCss.has(ref)) cssName = nameIndex.pathToCss.get(ref)
    else if (nameIndex.cssToPath.has(ref)) cssName = ref
    else cssName = ref.replace(/\./g, '-')
    return `var(--${cssName})`
  }
  return value
}

function loadJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'))
}

function layerMeta(doc, file, fallbackLayer) {
  const ext = doc.$extensions?.ship ?? {}
  return {
    file,
    description: doc.$description ?? file,
    selector: ext.selector ?? ':root',
    media: ext.media ?? null,
    layer: ext.layer ?? fallbackLayer ?? file.replace(/\.json$/, ''),
    themeId: ext.themeId ?? null,
    dark: !!ext.dark,
  }
}

function loadBaseLayers() {
  // Layer JSON files at model root (not themes/, not backup, not compat)
  const skip = new Set(['compat-aliases.json'])
  const files = readdirSync(MODEL)
    .filter((f) => f.endsWith('.json') && !skip.has(f))
    .sort()

  // First pass: load base for name index (aliases resolve against base)
  const baseFile = files.includes('base.json') ? 'base.json' : files[0]
  const baseDoc = loadJson(join(MODEL, baseFile))
  const baseTokensRaw = flattenTokens(baseDoc)
  const nameIndex = buildNameIndex(baseTokensRaw)

  const layers = []
  for (const file of files) {
    const doc = loadJson(join(MODEL, file))
    const meta = layerMeta(doc, file, file.replace(/\.json$/, ''))
    const raw = flattenTokens(doc)
    const tokens = raw.map((t) => ({
      ...t,
      cssValue: resolveValue(t.value, nameIndex),
    }))
    layers.push({ ...meta, tokens })
  }
  return { layers, nameIndex, baseTokens: layers.find((l) => l.layer === 'base')?.tokens ?? layers[0]?.tokens ?? [] }
}

function loadThemeLayers(nameIndex) {
  if (!existsSync(THEMES_DIR)) return []
  const files = readdirSync(THEMES_DIR)
    .filter((f) => f.endsWith('.json'))
    .sort()
  const themes = []
  for (const file of files) {
    const doc = loadJson(join(THEMES_DIR, file))
    const meta = layerMeta(doc, `themes/${file}`, file.replace(/\.json$/, ''))
    if (!meta.themeId) {
      meta.themeId = file.replace(/\.json$/, '')
    }
    if (!meta.selector || meta.selector === ':root') {
      meta.selector = `:root[data-theme='${meta.themeId}'],\n[data-theme-preview='${meta.themeId}']`
    }
    const raw = flattenTokens(doc)
    const tokens = raw.map((t) => ({
      ...t,
      cssValue: resolveValue(t.value, nameIndex),
    }))
    themes.push({ ...meta, layer: 'theme', tokens })
  }
  return themes
}

function loadCompatAliases() {
  const p = join(MODEL, 'compat-aliases.json')
  if (!existsSync(p)) return {}
  try {
    const doc = loadJson(p)
    return doc.aliases && typeof doc.aliases === 'object' ? doc.aliases : {}
  } catch {
    return {}
  }
}

// ─── CSS emit ────────────────────────────────────────────────────────

function emitBlock(selector, tokens, indent = '  ') {
  if (tokens.length === 0) return ''
  const lines = [`${selector} {`]
  for (const t of tokens) {
    lines.push(`${indent}--${t.cssName}: ${t.cssValue};`)
  }
  lines.push('}')
  return lines.join('\n')
}

function emitCompatAliases(selector, aliases, indent = '  ') {
  const entries = Object.entries(aliases)
  if (entries.length === 0) return ''
  const lines = [`${selector} {`, `${indent}/* Legacy aliases (PE compat) */`]
  for (const [oldName, newName] of entries.sort(([a], [b]) => a.localeCompare(b))) {
    lines.push(`${indent}--${oldName}: var(--${newName});`)
  }
  lines.push('}')
  return lines.join('\n')
}

function emitLayer(layer) {
  const parts = []
  parts.push(`/* ─── ${layer.layer}${layer.themeId ? `:${layer.themeId}` : ''} (${layer.file}) ─── */`)
  if (layer.description) {
    parts.push(`/* ${layer.description} */`)
  }
  const block = emitBlock(layer.selector, layer.tokens)
  if (layer.media) {
    parts.push(`@media ${layer.media} {`)
    parts.push(
      block
        .split('\n')
        .map((l) => (l ? `  ${l}` : l))
        .join('\n'),
    )
    parts.push('}')
  } else {
    parts.push(block)
  }
  return parts.join('\n')
}

// ─── Registry ────────────────────────────────────────────────────────

function buildRegistry(baseTokens, aliases) {
  const tokens = {}
  for (const t of baseTokens) {
    tokens[t.cssName] = {
      path: t.path,
      cssName: t.cssName,
      type: t.type,
      value: t.cssValue,
    }
  }

  // Color surface: explicit $type color + alias chains that resolve to color
  const colors = new Set()
  for (const [name, meta] of Object.entries(tokens)) {
    if (meta.type === 'color') colors.add(name)
  }
  for (let i = 0; i < 4; i++) {
    for (const [name, meta] of Object.entries(tokens)) {
      if (colors.has(name)) continue
      const m = String(meta.value).match(/^var\(--([a-zA-Z0-9_-]+)\)/)
      if (m && colors.has(m[1])) {
        colors.add(name)
        meta.type = 'color'
      }
    }
  }

  // Legacy alias names dual-emitted in CSS for PE stability (e.g. app-accent-*).
  // Include in names/colors so lint accepts var(--app-accent-…) and sx color surface.
  const aliasEntries = {}
  for (const [oldName, newName] of Object.entries(aliases)) {
    aliasEntries[oldName] = newName
    if (colors.has(newName)) colors.add(oldName)
    if (tokens[newName] && !tokens[oldName]) {
      tokens[oldName] = {
        path: tokens[newName].path,
        cssName: oldName,
        type: tokens[newName].type,
        value: `var(--${newName})`,
        aliasOf: newName,
      }
    }
  }

  const names = Object.keys(tokens).sort()
  // paths only for canonical (non-alias) tokens
  const paths = {}
  for (const n of names) {
    const meta = tokens[n]
    if (meta.aliasOf) continue
    paths[meta.path] = n
  }
  return {
    version: 2,
    source: 'packages/sail/tokens/model',
    generated: true,
    names,
    colors: [...colors].sort(),
    tokens,
    // path → cssName for J2 / tooling
    paths,
    aliases: aliasEntries,
  }
}

// ─── Main ────────────────────────────────────────────────────────────

function main() {
  const { layers, nameIndex, baseTokens } = loadBaseLayers()
  if (layers.length === 0) {
    console.error('No model/*.json files found')
    process.exit(1)
  }

  const aliases = loadCompatAliases()
  const themes = loadThemeLayers(nameIndex)

  const byLayer = Object.fromEntries(layers.map((l) => [l.layer, l]))
  const ordered = []
  if (byLayer.base) ordered.push(byLayer.base)
  for (const l of layers) {
    if (['base', 'reduced-motion', 'dark', 'density-compact', 'density-spacious'].includes(l.layer)) {
      continue
    }
    ordered.push(l)
  }

  const chromePath = join(MODEL, 'chrome.css')
  let chrome = ''
  try {
    chrome = readFileSync(chromePath, 'utf8')
  } catch {
    console.warn('warn: model/chrome.css missing — emitting tokens only')
  }

  const cssParts = [GENERATED_HEADER]
  for (const l of ordered) {
    cssParts.push(emitLayer(l))
    cssParts.push('')
  }
  // PE compat: dual-emit legacy --app-accent-* → var(--accent-*)
  if (Object.keys(aliases).length > 0) {
    cssParts.push('/* ─── compat aliases (legacy CSS names) ─── */')
    cssParts.push(emitCompatAliases(':root', aliases))
    cssParts.push('')
  }
  if (chrome) {
    cssParts.push(chrome.trimEnd())
    cssParts.push('')
  }
  if (byLayer['reduced-motion']) {
    cssParts.push(emitLayer(byLayer['reduced-motion']))
    cssParts.push('')
  }
  for (const name of ['density-compact', 'density-spacious', 'dark']) {
    if (byLayer[name]) {
      cssParts.push(emitLayer(byLayer[name]))
      cssParts.push('')
    }
  }
  // dual-emit dark-layer accent renames if present
  if (byLayer.dark && Object.keys(aliases).length > 0) {
    // only re-alias; values already flow via var() to accent-* overrides in .dark
    cssParts.push('/* ─── compat aliases under .dark ─── */')
    cssParts.push(emitCompatAliases('.dark', aliases))
    cssParts.push('')
  }

  const css = cssParts.join('\n').replace(/\n{3,}/g, '\n\n')
  mkdirSync(SRC, { recursive: true })
  const cssOut = join(SRC, 'tokens.css')
  writeFileSync(cssOut, css.endsWith('\n') ? css : css + '\n')

  // themes.css
  const themeParts = [THEMES_HEADER]
  // Stable order: catalog order if present, else alpha by themeId
  const themeOrder = [
    'ship-dark', 'paper', 'halo', 'console', 'contrast',
    'neutral-light', 'neutral-dark', 'glass', 'brut', '8bit', 'synthwave',
    'ship-light',
  ]
  const themeById = Object.fromEntries(themes.map((t) => [t.themeId, t]))
  const orderedThemes = []
  for (const id of themeOrder) {
    if (themeById[id]) orderedThemes.push(themeById[id])
  }
  for (const t of themes) {
    if (!themeOrder.includes(t.themeId)) orderedThemes.push(t)
  }
  for (const t of orderedThemes) {
    themeParts.push(emitLayer(t))
    themeParts.push('')
  }
  const themesChromePath = join(MODEL, 'themes-chrome.css')
  if (existsSync(themesChromePath)) {
    themeParts.push(readFileSync(themesChromePath, 'utf8').trimEnd())
    themeParts.push('')
  }
  const themesCss = themeParts.join('\n').replace(/\n{3,}/g, '\n\n')
  const themesOut = join(SRC, 'themes.css')
  writeFileSync(themesOut, themesCss.endsWith('\n') ? themesCss : themesCss + '\n')

  const registry = buildRegistry(baseTokens, aliases)
  const regOut = join(SRC, 'token-registry.json')
  writeFileSync(regOut, JSON.stringify(registry, null, 2) + '\n')

  console.log(
    `generate: ${registry.names.length} tokens → ${cssOut.replace(PKG + '/', '')}`,
  )
  console.log(
    `  themes: ${orderedThemes.length} → ${themesOut.replace(PKG + '/', '')}`,
  )
  console.log(
    `  registry: ${regOut.replace(PKG + '/', '')} (colors: ${registry.colors.length}; paths: ${Object.keys(registry.paths).length})`,
  )
  console.log(`  layers: ${layers.map((l) => l.layer).join(', ')}`)
}

main()
