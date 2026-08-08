/** ISO date helpers for sail-date-picker (YYYY-MM-DD). */

export const DOW = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as const
export const DOW_SHORT = ['S', 'M', 'T', 'W', 'T', 'F', 'S'] as const
export const MONTHS = [
  'January',
  'February',
  'March',
  'April',
  'May',
  'June',
  'July',
  'August',
  'September',
  'October',
  'November',
  'December',
] as const

export const pad2 = (n: number) => (n < 10 ? `0${n}` : String(n))

export const isoFor = (year: number, month: number, day: number): string =>
  `${year}-${pad2(month + 1)}-${pad2(day)}`

export const parseIso = (s: string): { y: number; m: number; d: number } | null => {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(s)
  if (!m) return null
  const y = Number(m[1])
  const mo = Number(m[2]) - 1
  const d = Number(m[3])
  if (mo < 0 || mo > 11 || d < 1 || d > 31) return null
  return { y, m: mo, d }
}

export const daysInMonth = (year: number, month: number): number =>
  new Date(year, month + 1, 0).getDate()

export const todayIso = (): string => {
  const d = new Date()
  return isoFor(d.getFullYear(), d.getMonth(), d.getDate())
}

export const addDays = (iso: string, delta: number): string => {
  const parts = parseIso(iso)
  if (!parts) return iso
  const d = new Date(parts.y, parts.m, parts.d + delta)
  return isoFor(d.getFullYear(), d.getMonth(), d.getDate())
}

export const clampIso = (iso: string, min: string, max: string): string => {
  if (min && iso < min) return min
  if (max && iso > max) return max
  return iso
}
