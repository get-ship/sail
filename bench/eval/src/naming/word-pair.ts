/**
 * Word-pair run ids (`calm-otter`). Never nanoids / timestamp-strings as names.
 * HARNESS-ARCH / HARNESS-2 non-negotiable.
 */

const ADJECTIVES = [
  'calm', 'brisk', 'clear', 'bold', 'quiet', 'swift', 'bright', 'keen',
  'steady', 'crisp', 'fair', 'noble', 'pluck', 'proud', 'rapid', 'solid',
  'vivid', 'warm', 'wise', 'zesty', 'amber', 'azure', 'coral', 'ember',
  'flint', 'frost', 'gleam', 'haven', 'ivory', 'jade', 'lunar', 'mossy',
  'pearl', 'quiet', 'rusty', 'silver', 'tidal', 'umber', 'velvet', 'winds',
] as const

const NOUNS = [
  'otter', 'heron', 'falcon', 'cedar', 'river', 'harbor', 'maple', 'osprey',
  'pine', 'reef', 'sparrow', 'stone', 'tide', 'willow', 'badger', 'canyon',
  'delta', 'eagle', 'fjord', 'glade', 'haven', 'inlet', 'jay', 'kestrel',
  'lark', 'meadow', 'nimbus', 'orchid', 'prairie', 'quail', 'raven', 'sage',
  'tern', 'upland', 'vale', 'wren', 'yarrow', 'zenith', 'asp', 'brook',
] as const

/** Deterministic hash → [0, 2^32). */
function fnv1a(s: string): number {
  let h = 0x811c9dc5
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = Math.imul(h, 0x01000193)
  }
  return h >>> 0
}

export interface WordPairOpts {
  /** Avoid these already-used pairs under the parent dir. */
  taken?: Set<string>
  /** Seed for reproducibility (tests). */
  seed?: string
  /** Max attempts before suffixing a counter. */
  maxAttempts?: number
}

/**
 * Allocate a word-pair run id. Prefers pure `adj-noun`; if exhausted under
 * `taken`, appends a small numeric suffix (`calm-otter-2`).
 */
export function allocateWordPair(opts: WordPairOpts = {}): string {
  const taken = opts.taken ?? new Set<string>()
  const max = opts.maxAttempts ?? 64
  const base = opts.seed ?? `${Date.now()}-${Math.random()}`

  for (let i = 0; i < max; i++) {
    const h = fnv1a(`${base}:${i}`)
    const adj = ADJECTIVES[h % ADJECTIVES.length]!
    const noun = NOUNS[(h >>> 8) % NOUNS.length]!
    const pair = `${adj}-${noun}`
    if (!taken.has(pair)) {
      taken.add(pair)
      return pair
    }
  }

  // Exhausted pure pairs in this namespace — still word-pair shaped, not a nanoid.
  for (let n = 2; n < 10_000; n++) {
    const h = fnv1a(`${base}:suffix:${n}`)
    const adj = ADJECTIVES[h % ADJECTIVES.length]!
    const noun = NOUNS[(h >>> 8) % NOUNS.length]!
    const pair = `${adj}-${noun}-${n}`
    if (!taken.has(pair)) {
      taken.add(pair)
      return pair
    }
  }
  throw new Error('word-pair allocator exhausted')
}

/** Validate a run id is word-pair shaped (not uuid/nanoid/timestamp). */
export function isWordPairId(id: string): boolean {
  if (!/^[a-z]+-[a-z]+(-\d+)?$/.test(id)) return false
  // Reject pure numeric / timestamp-looking
  if (/^\d{8}/.test(id)) return false
  // Reject uuid
  if (/^[0-9a-f]{8}-[0-9a-f]{4}-/.test(id)) return false
  return true
}
