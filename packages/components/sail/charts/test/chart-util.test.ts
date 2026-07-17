import { describe, expect, it } from 'vitest'
import { arcPath, linePath, parseJson, parseLabels, parseNums, toneColor } from '../src/chart-util'

describe('toneColor', () => {
  it('returns --foreground for null', () => {
    expect(toneColor(null)).toBe('var(--foreground)')
  })

  it('returns --foreground for undefined', () => {
    expect(toneColor(undefined)).toBe('var(--foreground)')
  })

  it('returns --foreground for empty string', () => {
    expect(toneColor('')).toBe('var(--foreground)')
  })

  it('maps primary to --primary', () => {
    expect(toneColor('primary')).toBe('var(--primary)')
  })

  it('maps info to --info', () => {
    expect(toneColor('info')).toBe('var(--info)')
  })

  it('maps success to --success', () => {
    expect(toneColor('success')).toBe('var(--success)')
  })

  it('maps warning to --warning', () => {
    expect(toneColor('warning')).toBe('var(--warning)')
  })

  it('maps destructive to --destructive', () => {
    expect(toneColor('destructive')).toBe('var(--destructive)')
  })

  it('maps accent to --accent', () => {
    expect(toneColor('accent')).toBe('var(--accent)')
  })

  it('maps muted to --muted-foreground', () => {
    expect(toneColor('muted')).toBe('var(--muted-foreground)')
  })

  it('maps default to --foreground', () => {
    expect(toneColor('default')).toBe('var(--foreground)')
  })

  it('falls back to --foreground for unknown tone', () => {
    expect(toneColor('rainbow')).toBe('var(--foreground)')
  })
})

describe('parseNums', () => {
  it('returns empty array for null', () => {
    expect(parseNums(null)).toEqual([])
  })

  it('returns empty array for undefined', () => {
    expect(parseNums(undefined)).toEqual([])
  })

  it('returns empty array for empty string', () => {
    expect(parseNums('')).toEqual([])
  })

  it('parses comma-separated integers', () => {
    expect(parseNums('1,2,3')).toEqual([1, 2, 3])
  })

  it('parses comma-separated floats', () => {
    expect(parseNums('1.5,2.7,3.14')).toEqual([1.5, 2.7, 3.14])
  })

  it('trims whitespace around values', () => {
    expect(parseNums(' 1 , 2 , 3 ')).toEqual([1, 2, 3])
  })

  it('filters out non-numeric values', () => {
    expect(parseNums('1,abc,3')).toEqual([1, 3])
  })

  it('filters out NaN and Infinity', () => {
    expect(parseNums('1,NaN,Infinity,2')).toEqual([1, 2])
  })
})

describe('parseLabels', () => {
  it('returns empty array for null', () => {
    expect(parseLabels(null)).toEqual([])
  })

  it('returns empty array for undefined', () => {
    expect(parseLabels(undefined)).toEqual([])
  })

  it('returns empty array for empty string', () => {
    expect(parseLabels('')).toEqual([])
  })

  it('parses comma-separated labels', () => {
    expect(parseLabels('a,b,c')).toEqual(['a', 'b', 'c'])
  })

  it('trims whitespace around labels', () => {
    expect(parseLabels(' Jan , Feb , Mar ')).toEqual(['Jan', 'Feb', 'Mar'])
  })

  it('preserves internal spaces in labels', () => {
    expect(parseLabels('Q1 2024,Q2 2024')).toEqual(['Q1 2024', 'Q2 2024'])
  })
})

describe('parseJson', () => {
  it('returns fallback for null', () => {
    expect(parseJson(null, [])).toEqual([])
  })

  it('returns fallback for undefined', () => {
    expect(parseJson(undefined, [])).toEqual([])
  })

  it('returns fallback for empty string', () => {
    expect(parseJson('', [])).toEqual([])
  })

  it('parses a JSON array', () => {
    expect(parseJson('[1,2,3]', [])).toEqual([1, 2, 3])
  })

  it('parses a JSON object', () => {
    expect(parseJson('{"a":1}', {})).toEqual({ a: 1 })
  })

  it('parses nested JSON', () => {
    const input = '[{"name":"a","data":[1,2,3]}]'
    expect(parseJson(input, [])).toEqual([{ name: 'a', data: [1, 2, 3] }])
  })

  it('returns fallback for malformed JSON', () => {
    expect(parseJson('not-json', [])).toEqual([])
  })

  it('returns null fallback for malformed JSON', () => {
    expect(parseJson('{bad}', null)).toBeNull()
  })
})

describe('linePath', () => {
  it('returns empty d and points for empty data', () => {
    const { d, points } = linePath([], 100, 50)
    expect(d).toBe('')
    expect(points).toEqual([])
  })

  it('single point centers horizontally at w/2', () => {
    const { points } = linePath([5], 100, 50)
    expect(points[0]![0]).toBe(50)
  })

  it('two points produce M and L commands', () => {
    const { d } = linePath([0, 10], 100, 50)
    expect(d).toMatch(/^M/)
    expect(d).toContain('L')
  })

  it('point count matches data length', () => {
    const { points } = linePath([1, 2, 3, 4, 5], 100, 50)
    expect(points).toHaveLength(5)
  })

  it('all equal values produce a flat line (no crash)', () => {
    const { d, points } = linePath([5, 5, 5], 100, 50)
    expect(d).not.toBe('')
    expect(points).toHaveLength(3)
  })

  it('uses pad to inset y coordinates', () => {
    const { points } = linePath([0, 10], 100, 50, 5)
    // min value → y = pad + innerH = 5 + 40 = 45
    expect(points[0]![1]).toBe(45)
  })
})

describe('arcPath', () => {
  it('returns a path string starting with M', () => {
    const p = arcPath(50, 50, 40, 0, Math.PI)
    expect(p).toMatch(/^M/)
    expect(p).toContain('A')
  })

  it('produces large-arc flag 1 for arc > PI', () => {
    const p = arcPath(50, 50, 40, 0, Math.PI * 1.5)
    expect(p).toContain(' 1 ')
  })

  it('produces large-arc flag 0 for arc <= PI', () => {
    const p = arcPath(50, 50, 40, 0, Math.PI * 0.5)
    expect(p).toContain(' 0 ')
  })
})
