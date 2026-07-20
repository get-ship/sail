//! Node-only walk-up resolver for `sail.config.json` ambient authority.
//!
//! Mirrors `sail_lint::config::resolve` + `Policy::network_for` /
//! `Policy::unconfined_for`. Browser CSR hosts must not import this module —
//! pass a bake-stamped or host-resolved {@link AmbientAuthority} into
//! `prepareRouteConfinement` instead.

import { existsSync, readFileSync, statSync } from 'node:fs'
import { dirname, isAbsolute, join, resolve as pathResolve } from 'node:path'

import {
  AMBIENT_DENY_ALL,
  SAIL_CONFIG_FILE,
  classifyAmbientJson,
  type AmbientAuthority,
} from './ambient-policy'

/**
 * Walk up from `fromDir` (inclusive) and return the first `sail.config.json`
 * path found, or `null`. Mirrors `sail_lint::config::resolve` / `owning_dir`.
 */
export function findSailConfigPath(fromDir: string): string | null {
  let cur: string | null = pathResolve(fromDir)
  while (cur) {
    const candidate = join(cur, SAIL_CONFIG_FILE)
    if (existsSync(candidate)) return candidate
    const parent = dirname(cur)
    if (parent === cur) break
    cur = parent
  }
  return null
}

/**
 * Resolve ambient authority for a source file path — JS mirror of
 * `Policy::network_for` + `Policy::unconfined_for` (same walk-up, same
 * fail-closed ambient classification). No config on the walk → deny-all.
 *
 * `sourceFile` may be a file or a directory; the walk starts at the file's
 * parent (matching Rust `source_file.parent()`), or at the directory itself
 * when a directory path is passed (host convenience for package roots).
 */
export function resolveAmbientAuthority(sourceFile: string): AmbientAuthority {
  const abs = isAbsolute(sourceFile) ? sourceFile : pathResolve(sourceFile)

  let startDir = dirname(abs)
  try {
    if (existsSync(abs) && statSync(abs).isDirectory()) {
      startDir = abs
    }
  } catch {
    // keep parent-of-path default
  }

  const configPath = findSailConfigPath(startDir)
  if (!configPath) return { ...AMBIENT_DENY_ALL, configPath: null }

  let text: string
  try {
    text = readFileSync(configPath, 'utf8')
  } catch {
    return { ...AMBIENT_DENY_ALL, configPath }
  }
  return classifyAmbientJson(text, configPath)
}
