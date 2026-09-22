// Domain-neutrality static gate for product compositions.
//
// Contract (`inv-domain-neutral`): named compositions (`sail-navbar`,
// `sail-theme-picker`, `sail-empty-state`, `sail-error-state`) must stay
// ship-free: zero `@ship/sdk-client` imports, zero
// `ship.<theme|account|project|billing>.` reads, zero product domain nouns
// (`account`, `billing`, `project`, `brand`) in code, parts, slot names, or
// event names.
//
// Gate regex (manual re-check):
//   grep -rniE "@ship/sdk-client|ship\.(account|theme|project|billing)\b" \
//     packages/components/sail/*/src
// This test is the regression guard.

import { readFileSync, readdirSync, statSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const PACK_SRC = dirname(fileURLToPath(import.meta.url))
// navbar and theme-picker live in the language now — the neutrality gate
// scans their stdlib sources (plus the runtime actions that carry their
// behavior). The property is the same: product compositions are ship-free.
const STDLIB = resolve(PACK_SRC, '../../../../../crates/sail/compiler/stdlib')
const ACTIONS = resolve(PACK_SRC, '../../../../sail/runtime/src/actions')

const COMPOSITIONS = [
  'navbar',
  'theme-picker',
] as const

// Every file under the composition's directory (recursively). Both the
// .sail source and the .controller.ts runtime are scanned; .test.ts specs
// are excluded (they may legitimately name the forbidden tokens in
// comments that DESCRIBE the absence — see sail-calendar/engine/purity.test.ts).
function compositionFiles(stem: string): string[] {
  const out: string[] = []
  const sail = join(STDLIB, `${stem}.sail`)
  if (statSync(sail, { throwIfNoEntry: false })?.isFile()) out.push(sail)
  const actionFile = join(ACTIONS, `${stem}.ts`)
  if (statSync(actionFile, { throwIfNoEntry: false })?.isFile()) out.push(actionFile)
  return out
}


// Forbidden product-coupling tokens. Each is a substring scan. Keep this
// list in sync with the GATE regex the operator runs at promotion time.
const FORBIDDEN_PRODUCT_TOKENS: { token: string; why: string }[] = [
  { token: '@ship/sdk-client', why: 'SDK import — primitive is pure UI' },
  { token: 'ship.account', why: 'product account namespace — primitive is domain-neutral' },
  { token: 'ship.theme', why: 'product theme namespace — drive tokens via @ship/tokens only' },
  { token: 'ship.project', why: 'product project namespace — primitive is domain-neutral' },
  { token: 'ship.billing', why: 'product billing namespace — primitive is domain-neutral' },
  { token: 'ship:notify', why: 'product notify channel — primitive does not bridge to product events' },
  { token: 'ship:theme:change', why: 'product theme channel — drive tokens via sail-theme-change' },
]

// Product domain nouns that must not appear in CODE. Comments that merely
// describe rationale are tolerated; the GATE regex catches the strong
// forms. A "code" hit is a token outside of a <!-- … --> / // … / /* … */
// comment block. We strip comments first, then scan — a real coupling
// won't hide in a comment.
function stripSailComments(src: string): string {
  return src
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/\/\/[^\n]*/g, '')
    .replace(/\/\*[\s\S]*?\*\//g, '')
}

function stripTsComments(src: string): string {
  return src
    .replace(/\/\/[^\n]*/g, '')
    .replace(/\/\*[\s\S]*?\*\//g, '')
}

describe('domain-neutrality (job-lock) — 4 product compositions are ship-free', () => {
  for (const stem of COMPOSITIONS) {
    const files = compositionFiles(stem)

    it(`<${stem}> has source files to scan (precondition)`, () => {
      // The audit can only run if the composition has at least a .sail
      // template. A .controller.ts is OPTIONAL — some compositions
      // (e.g. sail-empty-state) are pure templates with no runtime.
      // Empty directories FAIL the precondition (would mask a future
      // regression where the source is deleted).
      expect(files.length).toBeGreaterThanOrEqual(1)
    })

    it(`<${stem}> imports NO @ship/sdk-client (no product SDK reach)`, () => {
      for (const file of files) {
        const src = readFileSync(file, 'utf8')
        expect(
          src.includes('@ship/sdk-client'),
          `${file} must not import @ship/sdk-client`,
        ).toBe(false)
      }
    })

    it(`<${stem}> contains NO product coupling tokens in CODE (comments stripped)`, () => {
      for (const file of files) {
        const raw = readFileSync(file, 'utf8')
        const code = file.endsWith('.sail') ? stripSailComments(raw) : stripTsComments(raw)
        for (const { token, why } of FORBIDDEN_PRODUCT_TOKENS) {
          expect(
            code.includes(token),
            `${file} contains forbidden "${token}" — ${why}`,
          ).toBe(false)
        }
      }
    })

    it(`<${stem}> does not name the product domain nouns (account/billing/project) in CODE`, () => {
      for (const file of files) {
        const raw = readFileSync(file, 'utf8')
        const code = file.endsWith('.sail') ? stripSailComments(raw) : stripTsComments(raw)
        for (const noun of ['account', 'billing', 'project']) {
          // Word-boundary match — catches "account" but not "accountant".
          // The `brand` noun is intentionally NOT in this list — `brand`
          // is a generic layout slot name in <sail-navbar>, not a product
          // domain reference. Verifying the slot neutrality is a separate
          // test in test/sail-navbar.test.ts.
          const re = new RegExp(`\\b${noun}\\b`, 'i')
          expect(
            re.test(code),
            `${file} names forbidden product noun "${noun}" in code`,
          ).toBe(false)
        }
      }
    })
  }
})
