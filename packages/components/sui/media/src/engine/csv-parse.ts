// sui-media engine — RFC-4180-ish CSV/TSV parser. PURE: text → { rows,
// columns }. No I/O, no `@ship/sui-runtime`, no DOM. The CSV renderer composes
// the committed `sui-table`, so the parser's only job is the shape the table
// consumes: a header row drives column field/label when `header=true`, else
// synthetic `col0..N`; quoted fields with embedded delimiters/newlines; comma-
// vs-tab auto-detect.
//
// Authority: build-spec Slice 4 (sui-media-csv parse contract), §Props
// (`delimiter` comma|tab|auto, `header` bool), behavioral spec #6.

export type CsvDelimiter = 'comma' | 'tab' | 'auto'

export interface CsvParseOptions {
  /** First row is the header (drives columns). Default true. */
  header?: boolean
  /** Field delimiter. `auto` sniffs comma-vs-tab from the first line. */
  delimiter?: CsvDelimiter
}

export interface CsvColumn {
  field: string
  label: string
}

export type CsvRow = Record<string, string> & { id: string }

export interface CsvParseResult {
  columns: CsvColumn[]
  rows: CsvRow[]
}

/** Thrown when the input cannot be parsed (e.g. an unterminated quoted field).
 *  The renderer catches this to fire `sui-viewer-error {message}`. */
export class CsvParseError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'CsvParseError'
  }
}

/** Sniff the delimiter from the first physical line: tab wins when the first
 *  line has more tabs than commas, else comma. */
function detectDelimiter(text: string): string {
  const firstLine = text.split(/\r?\n/, 1)[0] ?? ''
  let tabs = 0
  let commas = 0
  for (const ch of firstLine) {
    if (ch === '\t') tabs++
    else if (ch === ',') commas++
  }
  return tabs > commas ? '\t' : ','
}

function resolveDelimiter(text: string, delimiter: CsvDelimiter): string {
  if (delimiter === 'tab') return '\t'
  if (delimiter === 'comma') return ','
  return detectDelimiter(text)
}

/** Tokenize the full text into a 2-D field matrix, honouring quoted fields
 *  (embedded delimiter/newline) and doubled-quote escaping. Throws on an
 *  unterminated quote. */
function tokenize(text: string, delim: string): string[][] {
  const rows: string[][] = []
  let field = ''
  let row: string[] = []
  let inQuotes = false
  let started = false // any char seen on the current record

  const pushField = (): void => {
    row.push(field)
    field = ''
  }
  const pushRow = (): void => {
    pushField()
    rows.push(row)
    row = []
    started = false
  }

  for (let i = 0; i < text.length; i++) {
    const ch = text[i]
    started = true
    if (inQuotes) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          field += '"'
          i++ // consume the escaped quote
        } else {
          inQuotes = false
        }
      } else {
        field += ch
      }
      continue
    }
    if (ch === '"') {
      inQuotes = true
      continue
    }
    if (ch === delim) {
      pushField()
      continue
    }
    if (ch === '\n') {
      pushRow()
      continue
    }
    if (ch === '\r') {
      // Swallow CR; the following LF (if any) ends the record.
      if (text[i + 1] === '\n') continue
      pushRow()
      continue
    }
    field += ch
  }

  if (inQuotes) {
    throw new CsvParseError('Unterminated quoted field')
  }
  // Flush a trailing record (no terminating newline) when content was seen.
  if (started || field.length > 0 || row.length > 0) {
    pushRow()
  }
  return rows
}

/** Parse CSV/TSV text into the `{ rows, columns }` shape the committed
 *  `sui-table` consumes. Each data row carries a stable `id` for the table's
 *  `{#for} key=`. */
export function parseCsv(text: string, options: CsvParseOptions = {}): CsvParseResult {
  const header = options.header ?? true
  const delim = resolveDelimiter(text, options.delimiter ?? 'auto')

  const matrix = tokenize(text, delim)
  if (matrix.length === 0) {
    return { columns: [], rows: [] }
  }

  let fields: string[]
  let dataStart: number
  if (header) {
    fields = matrix[0]!.map((h, i) => (h.trim() === '' ? `col${i}` : h))
    dataStart = 1
  } else {
    const width = matrix.reduce((w, r) => Math.max(w, r.length), 0)
    fields = Array.from({ length: width }, (_, i) => `col${i}`)
    dataStart = 0
  }

  const columns: CsvColumn[] = fields.map((field) => ({ field, label: field }))

  const rows: CsvRow[] = []
  for (let r = dataStart; r < matrix.length; r++) {
    const cells = matrix[r]!
    const record: Record<string, string> = {}
    for (let c = 0; c < fields.length; c++) {
      record[fields[c]!] = cells[c] ?? ''
    }
    rows.push({ ...record, id: `row-${r - dataStart}` })
  }

  return { columns, rows }
}
