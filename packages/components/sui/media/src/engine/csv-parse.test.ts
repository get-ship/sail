// The CSV/TSV parser (engine/csv-parse.ts) — a sui-media engine spec.
//
// `sui-media-csv` composes the committed `sui-table`, so the parser's job is
// purely text → `{ rows, columns }`: header-row → column field/label when
// `header=true`, else synthetic `col0..N`; quoted fields with embedded
// delimiters/newlines; comma-vs-tab auto-detect.
//
// Authority: build-spec Slice 4 (sui-media-csv parse contract), §Props
// (`delimiter` comma|tab|auto, `header` bool), behavioral spec #6.

import { describe, expect, it } from 'vitest'
// RED: greenfield engine module.
import { parseCsv } from './csv-parse'

describe('parseCsv — header row drives columns', () => {
  it('first row becomes columns; remaining rows become records (header=true)', () => {
    const { rows, columns } = parseCsv('name,age\nAlice,30\nBob,25', { header: true })
    expect(columns.map((c) => c.field)).toEqual(['name', 'age'])
    expect(rows.length).toBe(2)
    expect(rows[0]).toMatchObject({ name: 'Alice', age: '30' })
    expect(rows[1]).toMatchObject({ name: 'Bob', age: '25' })
  })

  it('synthesizes col0..N labels when header=false', () => {
    const { rows, columns } = parseCsv('Alice,30\nBob,25', { header: false })
    expect(columns.map((c) => c.field)).toEqual(['col0', 'col1'])
    expect(rows.length).toBe(2)
    expect(rows[0]).toMatchObject({ col0: 'Alice', col1: '30' })
  })
})

describe('parseCsv — RFC-4180-ish quoting', () => {
  it('honors quoted fields containing the delimiter', () => {
    const { rows } = parseCsv('name,note\n"Doe, John",hi', { header: true })
    expect(rows[0]).toMatchObject({ name: 'Doe, John', note: 'hi' })
  })

  it('honors quoted fields containing an embedded newline', () => {
    const { rows } = parseCsv('name,note\n"line1\nline2",ok', { header: true })
    expect(rows[0]!.note).toBe('ok')
    expect(rows[0]!.name).toBe('line1\nline2')
  })

  it('unescapes doubled quotes inside a quoted field', () => {
    const { rows } = parseCsv('q\n"she said ""hi"""', { header: true })
    expect(rows[0]!.q).toBe('she said "hi"')
  })
})

describe('parseCsv — delimiter auto-detect (comma vs tab)', () => {
  it('auto-detects a tab-delimited body', () => {
    const { rows, columns } = parseCsv('a\tb\n1\t2', { header: true, delimiter: 'auto' })
    expect(columns.map((c) => c.field)).toEqual(['a', 'b'])
    expect(rows[0]).toMatchObject({ a: '1', b: '2' })
  })

  it('respects an explicit tab delimiter', () => {
    const { rows } = parseCsv('a\tb\n1\t2', { header: true, delimiter: 'tab' })
    expect(rows[0]).toMatchObject({ a: '1', b: '2' })
  })
})

describe('parseCsv — every row carries a stable id (table {#for} key)', () => {
  it('assigns a unique id per data row', () => {
    const { rows } = parseCsv('name\nAlice\nBob\nCara', { header: true })
    const ids = rows.map((r) => r.id)
    expect(ids.length).toBe(3)
    expect(new Set(ids).size).toBe(3)
  })
})
