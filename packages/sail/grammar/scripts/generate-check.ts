#!/usr/bin/env bun
/**
 * Fail if checked-in `grammar.js` or `queries/highlights.scm` drift
 * from a fresh compiler vocabulary dump + `generate-grammar.ts`.
 * Same inputs as `bun run generate`.
 *
 * Usage (from packages/sail/grammar):
 *   bun run generate-check
 */

import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const GRAMMAR_DIR = resolve(SCRIPT_DIR, '..');
const WORKSPACE = resolve(GRAMMAR_DIR, '../../..');
const GENERATOR = join(SCRIPT_DIR, 'generate-grammar.ts');

const CHECKS: Array<{ generated: string; committed: string; label: string }> = [
  {
    generated: 'grammar.js',
    committed: join(GRAMMAR_DIR, 'grammar.js'),
    label: 'grammar.js',
  },
  {
    generated: 'queries/highlights.scm',
    committed: join(GRAMMAR_DIR, 'queries/highlights.scm'),
    label: 'queries/highlights.scm',
  },
];

function run(cmd: string, args: string[], cwd: string): void {
  const r = spawnSync(cmd, args, {
    cwd,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  if (r.status !== 0) {
    if (r.stdout) process.stderr.write(r.stdout);
    if (r.stderr) process.stderr.write(r.stderr);
    process.exit(r.status ?? 1);
  }
}

function readUtf8(path: string): string | null {
  try {
    return readFileSync(path, 'utf8');
  } catch {
    return null;
  }
}

function printDiff(generatedPath: string, committedPath: string, label: string): void {
  const r = spawnSync(
    'diff',
    ['-u', '--label', `generated/${label}`, '--label', `committed/${label}`, generatedPath, committedPath],
    { encoding: 'utf8' },
  );
  if (r.stdout) process.stderr.write(r.stdout);
  if (r.stderr) process.stderr.write(r.stderr);
}

function main(): void {
  const tmp = mkdtempSync(join(tmpdir(), 'sail-generate-check-'));
  try {
    const vocabPath = join(tmp, 'vocab.json');

    run(
      'cargo',
      ['run', '-q', '-p', 'sail-compiler', '--', 'vocabulary', '--out', vocabPath],
      WORKSPACE,
    );

    run(
      'bun',
      [
        'run',
        GENERATOR,
        '--vocab',
        vocabPath,
        '--out-dir',
        tmp,
      ],
      GRAMMAR_DIR,
    );

    let drifted = false;
    for (const check of CHECKS) {
      const generatedPath = join(tmp, check.generated);
      const generated = readUtf8(generatedPath);
      const committed = readUtf8(check.committed);
      if (generated === null) {
        console.error(`generate-check: missing generated ${check.label} at ${generatedPath}`);
        drifted = true;
        continue;
      }
      if (committed === null) {
        console.error(
          `generate-check: missing committed ${check.label} at ${check.committed}\n` +
            `  regenerate with: bun run generate`,
        );
        drifted = true;
        continue;
      }
      if (generated !== committed) {
        console.error(`generate-check: ${check.label} drifted from compiler vocabulary`);
        printDiff(generatedPath, check.committed, check.label);
        drifted = true;
      } else {
        console.log(`ok: ${check.label}`);
      }
    }

    if (drifted) {
      console.error(
        'generate-check: regenerate with `bun run generate` in packages/sail/grammar',
      );
      process.exit(1);
    }
  } finally {
    rmSync(tmp, { recursive: true, force: true });
  }
}

main();
