// sail-calendar engine — pure date arithmetic (Slice 0).
//
// TIMEZONE (DECIDED, not deferred): timed events are LOCAL WALL-CLOCK
// `ISODateTime` strings (`YYYY-MM-DDThh:mm`, no offset), matching the
// sail-date-picker's local-`Date` arithmetic and avoiding a tz library. DST
// fold/gap handling and an explicit-offset / IANA-zone mode are an explicit
// v1 GAP (pinned by an `it.skip` DST-gap test in date-math.test.ts) — a
// consumer needing zone-correct times normalizes upstream and feeds
// wall-clock strings. All helpers below are pure: no `signal`/`derived`, no
// I/O, no DOM. The proven primitives (isoFor/parseIso/daysInMonth/addDays/
// clampIso) are ported verbatim-in-spirit from sail-date-picker.controller.ts
// so two packs share one implementation (CLAUDE.md "one way to do things").

export type ISODate = string // 'YYYY-MM-DD'
export type ISODateTime = string // 'YYYY-MM-DDThh:mm' (local wall-clock)

export type WeekStart = 0 | 1 | 2 | 3 | 4 | 5 | 6

export interface DateParts {
  y: number
  m: number // 0-based
  d: number
}
export interface DateTimeParts extends DateParts {
  hh: number
  mm: number
}

export interface GridCell {
  iso: ISODate
  day: number
  inMonth: boolean
  isWeekend: boolean
}

const pad2 = (n: number): string => (n < 10 ? `0${n}` : String(n))

/** Build an ISO date string. `month` is 0-based (matches `Date.getMonth()`). */
export const isoFor = (year: number, month: number, day: number): ISODate =>
  `${year}-${pad2(month + 1)}-${pad2(day)}`

/** Parse `YYYY-MM-DD`. Returns null on any malformed / out-of-range input. */
export const parseIso = (s: string): DateParts | null => {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(s)
  if (!m) return null
  const y = Number(m[1])
  const mo = Number(m[2]) - 1
  const d = Number(m[3])
  if (mo < 0 || mo > 11 || d < 1 || d > 31) return null
  return { y, m: mo, d }
}

/** Parse `YYYY-MM-DDThh:mm`. Returns null on malformed / out-of-range input. */
export const parseIsoDateTime = (s: string): DateTimeParts | null => {
  const m = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})$/.exec(s)
  if (!m) return null
  const y = Number(m[1])
  const mo = Number(m[2]) - 1
  const d = Number(m[3])
  const hh = Number(m[4])
  const mm = Number(m[5])
  if (mo < 0 || mo > 11 || d < 1 || d > 31) return null
  if (hh < 0 || hh > 23 || mm < 0 || mm > 59) return null
  return { y, m: mo, d, hh, mm }
}

/** Tuple [year, month1based, day] for a valid ISO date (0,0,0 on junk). */
export const ymd = (iso: ISODate): [number, number, number] => {
  const p = parseIso(iso)
  return p ? [p.y, p.m + 1, p.d] : [0, 0, 0]
}

/** Number of days in a (0-based) month, leap-aware. */
export const daysInMonth = (year: number, month: number): number =>
  new Date(year, month + 1, 0).getDate()

/** Today's local date as ISO. */
export const todayIso = (): ISODate => {
  const d = new Date()
  return isoFor(d.getFullYear(), d.getMonth(), d.getDate())
}

/** Today's local wall-clock as ISODateTime (minute resolution). */
export const nowIsoDateTime = (): ISODateTime => {
  const d = new Date()
  return `${isoFor(d.getFullYear(), d.getMonth(), d.getDate())}T${pad2(
    d.getHours(),
  )}:${pad2(d.getMinutes())}`
}

/** Advance an ISO date by `delta` days (negative = backward). */
export const addDays = (iso: ISODate, delta: number): ISODate => {
  const p = parseIso(iso)
  if (!p) return iso
  const d = new Date(p.y, p.m, p.d + delta)
  return isoFor(d.getFullYear(), d.getMonth(), d.getDate())
}

/** Advance an ISO date by `delta` months, keeping the day-of-month clamped. */
export const addMonths = (iso: ISODate, delta: number): ISODate => {
  const p = parseIso(iso)
  if (!p) return iso
  const d = new Date(p.y, p.m + delta, 1)
  const day = Math.min(p.d, daysInMonth(d.getFullYear(), d.getMonth()))
  return isoFor(d.getFullYear(), d.getMonth(), day)
}

/** Clamp an ISO date into `[min, max]` (lexical compare is valid for ISO). */
export const clampIso = (iso: ISODate, min: ISODate, max: ISODate): ISODate => {
  if (min && iso < min) return min
  if (max && iso > max) return max
  return iso
}

/** 0=Sun..6=Sat day-of-week for an ISO date. */
export const dayOfWeek = (iso: ISODate): number => {
  const p = parseIso(iso)
  if (!p) return 0
  return new Date(p.y, p.m, p.d).getDay()
}

export const isWeekend = (iso: ISODate): boolean => {
  const dow = dayOfWeek(iso)
  return dow === 0 || dow === 6
}

/** First ISO date of the week containing `iso`, given `weekStartsOn`. */
export const startOfWeek = (iso: ISODate, weekStartsOn: WeekStart): ISODate => {
  const dow = dayOfWeek(iso)
  const back = (dow - weekStartsOn + 7) % 7
  return addDays(iso, -back)
}

/** The 7 ISO days of the week containing `iso`. */
export const weekDays = (iso: ISODate, weekStartsOn: WeekStart): ISODate[] => {
  const start = startOfWeek(iso, weekStartsOn)
  const out: ISODate[] = []
  for (let i = 0; i < 7; i++) out.push(addDays(start, i))
  return out
}

/**
 * The fixed 6×7 = 42-cell month grid for (year, 0-based month), aligned to
 * `weekStartsOn`. Leading/trailing cells from adjacent months carry
 * `inMonth: false`. Never virtualized — always exactly 42 cells.
 */
export const monthGrid = (
  year: number,
  month: number,
  weekStartsOn: WeekStart,
): GridCell[] => {
  const firstOfMonth = isoFor(year, month, 1)
  const start = startOfWeek(firstOfMonth, weekStartsOn)
  const out: GridCell[] = []
  for (let i = 0; i < 42; i++) {
    const iso = addDays(start, i)
    const p = parseIso(iso)!
    out.push({
      iso,
      day: p.d,
      inMonth: p.y === year && p.m === month,
      isWeekend: isWeekend(iso),
    })
  }
  return out
}

/** Minutes-from-midnight for the wall-clock time of an ISODateTime. */
export const dateTimeMinutes = (dt: ISODateTime): number => {
  const p = parseIsoDateTime(dt)
  if (!p) return 0
  return p.hh * 60 + p.mm
}

/** Advance an ISODateTime by `delta` minutes, rolling the date as needed. */
export const addMinutes = (dt: ISODateTime, delta: number): ISODateTime => {
  const p = parseIsoDateTime(dt)
  if (!p) return dt
  const d = new Date(p.y, p.m, p.d, p.hh, p.mm + delta)
  return `${isoFor(d.getFullYear(), d.getMonth(), d.getDate())}T${pad2(
    d.getHours(),
  )}:${pad2(d.getMinutes())}`
}

/** Signed wall-clock minute difference `b - a`. */
export const diffMinutes = (a: ISODateTime, b: ISODateTime): number => {
  const pa = parseIsoDateTime(a)
  const pb = parseIsoDateTime(b)
  if (!pa || !pb) return 0
  const da = new Date(pa.y, pa.m, pa.d, pa.hh, pa.mm).getTime()
  const db = new Date(pb.y, pb.m, pb.d, pb.hh, pb.mm).getTime()
  return Math.round((db - da) / 60000)
}

/** Half-open interval overlap test: [aStart,aEnd) ∩ [bStart,bEnd) ≠ ∅. */
export const rangeOverlaps = (
  aStart: number,
  aEnd: number,
  bStart: number,
  bEnd: number,
): boolean => aStart < bEnd && bStart < aEnd
