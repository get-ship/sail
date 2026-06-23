// Model invocation against the Anthropic Messages API. We use fetch
// directly so the rig has no SDK install footprint — the wire format
// is small and stable. The system primer + catalog + few-shots go in
// the system block with cache_control=ephemeral so repeat tasks reuse
// the prompt cache.

import type { PromptParts } from './prompt.ts'

export interface DispatchOptions {
  model: string
  apiKey: string
  baseUrl?: string
  maxTokens?: number
  /** Inject a stub for tests; real callers should leave this undefined. */
  fetchImpl?: typeof fetch
}

export interface DispatchResult {
  text: string
  usage?: {
    inputTokens: number
    outputTokens: number
    cacheReadInputTokens?: number
    cacheCreationInputTokens?: number
  }
}

const DEFAULT_BASE = 'https://api.anthropic.com'
const DEFAULT_MAX_TOKENS = 4096

export async function dispatch(
  parts: PromptParts,
  opts: DispatchOptions,
): Promise<DispatchResult> {
  if (!opts.apiKey) {
    throw new Error('dispatch: apiKey is required')
  }
  const fetchImpl = opts.fetchImpl ?? fetch
  const url = (opts.baseUrl ?? DEFAULT_BASE).replace(/\/$/, '') + '/v1/messages'

  const body = {
    model: opts.model,
    max_tokens: opts.maxTokens ?? DEFAULT_MAX_TOKENS,
    system: [
      {
        type: 'text',
        text: parts.system,
        cache_control: { type: 'ephemeral' },
      },
    ],
    messages: [{ role: 'user', content: parts.user }],
  }

  const res = await fetchImpl(url, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'x-api-key': opts.apiKey,
      'anthropic-version': '2023-06-01',
    },
    body: JSON.stringify(body),
  })

  if (!res.ok) {
    const text = await safeText(res)
    throw new Error(`anthropic API ${res.status}: ${text}`)
  }

  const json = (await res.json()) as AnthropicMessageResponse
  const text = (json.content ?? [])
    .filter((b): b is AnthropicTextBlock => b.type === 'text')
    .map((b) => b.text)
    .join('')

  const result: DispatchResult = { text }
  if (json.usage) {
    result.usage = {
      inputTokens: json.usage.input_tokens,
      outputTokens: json.usage.output_tokens,
      ...(json.usage.cache_read_input_tokens !== undefined
        ? { cacheReadInputTokens: json.usage.cache_read_input_tokens }
        : {}),
      ...(json.usage.cache_creation_input_tokens !== undefined
        ? { cacheCreationInputTokens: json.usage.cache_creation_input_tokens }
        : {}),
    }
  }
  return result
}

async function safeText(res: Response): Promise<string> {
  try {
    return await res.text()
  } catch {
    return '<no body>'
  }
}

interface AnthropicTextBlock {
  type: 'text'
  text: string
}

interface AnthropicMessageResponse {
  content?: Array<AnthropicTextBlock | { type: string }>
  usage?: {
    input_tokens: number
    output_tokens: number
    cache_read_input_tokens?: number
    cache_creation_input_tokens?: number
  }
}
