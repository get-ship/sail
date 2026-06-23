// sui-sheet engine — the NOOP FormulaCore (literal + cell-ref arithmetic only).
//
// The deterministic, vendor-free implementation of OUR `FormulaCore` contract —
// the seam the DAG slice binds to so it is testable WITHOUT the vendored parser.
// It parses a formula's cell/range references into the dep set the DAG needs and
// evaluates arithmetic (`+ - * / ( )`) over numeric literals + plain cell refs,
// but has NO function library — any function call returns `#NAME?`.
//
// PURE TypeScript — imports OUR `a1` algebra + contract types ONLY (no runtime,
// no vendor). The engine binds to `FormulaCore`, never to this concretely.
//
// Authority: build-spec §"The formula-core vendor boundary" (the FormulaCore
// interface), §Slice 2 (the literal-only stub core), adapter-noop.test.ts.

import { fromA1, parseRange } from '../a1'
import type { FormulaCore } from '../contract'
import type { CellAddress, ScalarValue } from '../types'

type Dep = CellAddress | { range: [CellAddress, CellAddress] }

const DIV0: ScalarValue = { error: '#DIV/0!' }
const NAME: ScalarValue = { error: '#NAME?' }

/** Build the literal-only core. Stateless — one instance is reusable. */
export function createNoopFormulaCore(): FormulaCore {
  return {
    parse(formula: string, _origin: CellAddress) {
      const body = formula.startsWith('=') ? formula.slice(1) : formula
      let tokens: Token[]
      try {
        tokens = tokenize(body)
      } catch {
        tokens = []
      }
      const deps = collectDeps(tokens)
      return {
        deps,
        evaluate(getCell: (a: CellAddress) => ScalarValue): ScalarValue {
          try {
            return evalTokens(tokens, getCell)
          } catch (e) {
            if (e instanceof FormulaEvalError) return { error: e.code }
            return NAME
          }
        },
      }
    },
    supportedFunctions() {
      return []
    },
  }
}

// ── tokenizer ────────────────────────────────────────────────────────────────

type Token =
  | { kind: 'num'; value: number }
  | { kind: 'op'; value: '+' | '-' | '*' | '/' }
  | { kind: 'lparen' }
  | { kind: 'rparen' }
  | { kind: 'ref'; a1: string }
  | { kind: 'range'; ref: string }
  | { kind: 'func'; name: string } // an identifier directly followed by '('

const CELL = /^\$?[A-Za-z]+\$?[0-9]+/
const IDENT = /^[A-Za-z_][A-Za-z0-9_.]*/
const NUMBER = /^[0-9]+(\.[0-9]+)?/

function tokenize(src: string): Token[] {
  const out: Token[] = []
  let i = 0
  while (i < src.length) {
    const ch = src[i]!
    if (ch === ' ' || ch === '\t') {
      i++
      continue
    }
    if (ch === '+' || ch === '-' || ch === '*' || ch === '/') {
      out.push({ kind: 'op', value: ch })
      i++
      continue
    }
    if (ch === '(') {
      out.push({ kind: 'lparen' })
      i++
      continue
    }
    if (ch === ')') {
      out.push({ kind: 'rparen' })
      i++
      continue
    }
    const rest = src.slice(i)
    // A range (`A1:A3`) — must be tried before a bare cell so the colon is kept.
    const range = matchRange(rest)
    if (range) {
      out.push({ kind: 'range', ref: range })
      i += range.length
      continue
    }
    const cell = CELL.exec(rest)
    if (cell) {
      out.push({ kind: 'ref', a1: cell[0] })
      i += cell[0].length
      continue
    }
    const ident = IDENT.exec(rest)
    if (ident) {
      // An identifier followed by '(' is a function call (unsupported here).
      const after = src.slice(i + ident[0].length).trimStart()
      if (after.startsWith('(')) {
        out.push({ kind: 'func', name: ident[0] })
        i += ident[0].length
        continue
      }
      // A bare identifier (e.g. a name) the noop core cannot resolve.
      throw new Error(`noop-core: unsupported identifier ${ident[0]}`)
    }
    const num = NUMBER.exec(rest)
    if (num) {
      out.push({ kind: 'num', value: Number(num[0]) })
      i += num[0].length
      continue
    }
    throw new Error(`noop-core: unexpected char ${JSON.stringify(ch)}`)
  }
  return out
}

/** Match a leading range reference (`A1:A3`, `A:A`, `1:1`) — both sides bounded
 *  by the cell/col/row grammar around a single colon. */
function matchRange(rest: string): string | null {
  const m = /^(\$?[A-Za-z]+\$?[0-9]+|\$?[A-Za-z]+|\$?[0-9]+):(\$?[A-Za-z]+\$?[0-9]+|\$?[A-Za-z]+|\$?[0-9]+)/.exec(
    rest,
  )
  if (!m) return null
  // Reject a single-cell match masquerading (the regex requires the colon).
  return m[0]
}

// ── dependency collection ──────────────────────────────────────────────────

function collectDeps(tokens: Token[]): Dep[] {
  const deps: Dep[] = []
  for (const t of tokens) {
    if (t.kind === 'ref') deps.push(fromA1(t.a1))
    else if (t.kind === 'range') {
      const parsed = parseRange(t.ref)
      deps.push({ range: [parsed.start, parsed.end] })
    }
  }
  return deps
}

// ── evaluator (recursive-descent over the operator precedence grammar) ───────

class FormulaEvalError extends Error {
  constructor(public code: '#DIV/0!' | '#NAME?' | '#VALUE!') {
    super(code)
  }
}

function evalTokens(tokens: Token[], getCell: (a: CellAddress) => ScalarValue): ScalarValue {
  if (tokens.length === 0) return null
  const p = new Parser(tokens, getCell)
  const v = p.parseExpr()
  if (!p.atEnd()) throw new FormulaEvalError('#NAME?')
  return v
}

class Parser {
  private pos = 0
  constructor(
    private readonly toks: Token[],
    private readonly getCell: (a: CellAddress) => ScalarValue,
  ) {}

  atEnd(): boolean {
    return this.pos >= this.toks.length
  }
  private peek(): Token | undefined {
    return this.toks[this.pos]
  }
  private next(): Token | undefined {
    return this.toks[this.pos++]
  }

  parseExpr(): number {
    let left = this.parseTerm()
    for (;;) {
      const t = this.peek()
      if (t?.kind === 'op' && (t.value === '+' || t.value === '-')) {
        this.next()
        const right = this.parseTerm()
        left = t.value === '+' ? left + right : left - right
      } else break
    }
    return left
  }

  private parseTerm(): number {
    let left = this.parseFactor()
    for (;;) {
      const t = this.peek()
      if (t?.kind === 'op' && (t.value === '*' || t.value === '/')) {
        this.next()
        const right = this.parseFactor()
        if (t.value === '*') left = left * right
        else {
          if (right === 0) throw new FormulaEvalError('#DIV/0!')
          left = left / right
        }
      } else break
    }
    return left
  }

  private parseFactor(): number {
    const t = this.next()
    if (!t) throw new FormulaEvalError('#NAME?')
    if (t.kind === 'op' && t.value === '-') return -this.parseFactor()
    if (t.kind === 'op' && t.value === '+') return this.parseFactor()
    if (t.kind === 'num') return t.value
    if (t.kind === 'lparen') {
      const v = this.parseExpr()
      const close = this.next()
      if (close?.kind !== 'rparen') throw new FormulaEvalError('#NAME?')
      return v
    }
    if (t.kind === 'ref') return this.coerceNumber(this.getCell(fromA1(t.a1)))
    // A function call OR a range used as a scalar — the noop core has no library.
    if (t.kind === 'func' || t.kind === 'range') throw new FormulaEvalError('#NAME?')
    throw new FormulaEvalError('#NAME?')
  }

  /** Coerce a cell value to a number for arithmetic: empty → 0, an error
   *  propagates, a non-numeric string → `#VALUE!`. */
  private coerceNumber(v: ScalarValue): number {
    if (v == null) return 0
    if (typeof v === 'number') return v
    if (typeof v === 'boolean') return v ? 1 : 0
    if (typeof v === 'object' && 'error' in v) throw new FormulaEvalError('#NAME?')
    const n = Number(v)
    if (Number.isNaN(n)) throw new FormulaEvalError('#VALUE!')
    return n
  }
}
