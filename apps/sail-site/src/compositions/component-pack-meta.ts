// Pack-component detail metadata for /components/[tag]. Controllers import this
// so the route file stays wire-up + stage markup.

export const PACK_TAGS = new Set([
  'sail-sparkline',
  'sail-area-chart',
  'sail-bar-chart',
  'sail-donut',
  'sail-gauge',
  'sail-heatmap',
  'sail-line-chart',
  'sail-media-image',
  'sail-media-av',
  'sail-code-editor',
  'sail-mermaid',
  'sail-message',
  'sail-citation',
  'sail-composer',
  'sail-thinking-dots',
  'sail-tool-call',
  'sail-canvas',
  'sail-board',
  'sail-draw',
  'sail-graph-canvas',
  'sail-annotation-layer',
  'sail-annotation-list',
  'sail-annotation-marker',
])

export const PACK_PROPS: Record<string, { name: string; type: string; desc: string }[]> = {
  'sail-sparkline': [
    { name: 'data', type: 'string', desc: 'CSV numbers, e.g. "2,5,3,8,4,6,7".' },
    { name: 'width', type: 'number', desc: 'SVG width in px.' },
    { name: 'height', type: 'number', desc: 'SVG height in px.' },
  ],
  'sail-area-chart': [
    { name: 'series', type: 'string (JSON)', desc: '[{name, data, tone?}] — multi-series.' },
    { name: 'labels', type: 'string (CSV)', desc: 'X-axis labels.' },
  ],
  'sail-bar-chart': [
    { name: 'data', type: 'string (CSV)', desc: 'Bar values, e.g. "2,5,3,8,4".' },
    { name: 'labels', type: 'string (CSV)', desc: 'Bar labels.' },
  ],
  'sail-donut': [
    { name: 'data', type: 'string (JSON)', desc: '[{label,value,tone?}] segments.' },
    { name: 'diameter', type: 'number', desc: 'Outer diameter in px.' },
  ],
  'sail-gauge': [
    { name: 'value', type: 'number', desc: 'Indicator position (0..max).' },
    { name: 'diameter', type: 'number', desc: 'Outer diameter in px.' },
  ],
  'sail-heatmap': [
    { name: 'rows', type: 'count or labels', desc: 'Row count ("4") or comma labels.' },
    { name: 'cols', type: 'count or labels', desc: 'Column count or comma labels.' },
    { name: 'values', type: 'string (CSV)', desc: 'Row-major cell values (alias of data).' },
    { name: 'data', type: 'string', desc: 'Same as values; 1D CSV or 2D JSON.' },
  ],
  'sail-line-chart': [
    { name: 'series', type: 'string (JSON)', desc: '[{name, data, tone?}].' },
    { name: 'labels', type: 'string (CSV)', desc: 'X-axis labels.' },
  ],
  'sail-media-image': [
    { name: 'src', type: 'string', desc: 'Image URL.' },
    { name: 'alt', type: 'string', desc: 'Alt text.' },
  ],
  'sail-media-av': [
    { name: 'src', type: 'string', desc: 'Audio/video URL.' },
    { name: 'mime', type: 'string', desc: 'Override mime type.' },
  ],
  'sail-code-editor': [
    { name: 'value', type: 'string', desc: 'Editor text.' },
    { name: 'language', type: 'string', desc: 'CodeMirror language.' },
    { name: 'line-numbers', type: 'bool', desc: 'Show gutter.' },
  ],
  'sail-mermaid': [{ name: 'value', type: 'string', desc: 'Mermaid source text.' }],
  'sail-message': [
    { name: 'from', type: 'string', desc: 'Message author (user|assistant|system|tool).' },
    { name: 'state', type: 'string', desc: 'Message state (user|assistant|...).' },
    { name: 'content', type: 'string', desc: 'Body text.' },
  ],
  'sail-citation': [
    { name: 'source', type: 'string', desc: 'Citation source label.' },
    { name: 'href', type: 'string', desc: 'Click target.' },
  ],
  'sail-composer': [
    { name: 'placeholder', type: 'string', desc: 'Empty-state copy.' },
    { name: 'multiline', type: 'bool', desc: 'Growing textarea (default true). Set false for a single-line field.' },
    { name: 'disabled', type: 'bool', desc: 'Blocks input and submit.' },
  ],
  'sail-thinking-dots': [{ name: 'label', type: 'string', desc: 'Live-region label.' }],
  'sail-tool-call': [
    { name: 'name', type: 'string', desc: 'Tool name.' },
    { name: 'status', type: 'string', desc: 'running|ok|error.' },
  ],
  'sail-canvas': [
    { name: 'width', type: 'number', desc: 'Viewport width in px.' },
    { name: 'height', type: 'number', desc: 'Viewport height in px.' },
  ],
  'sail-graph-canvas': [
    { name: 'nodes', type: 'string (JSON)', desc: '[{id, kind, title, x, y}].' },
    { name: 'edges', type: 'string (JSON)', desc: '[{from, to}].' },
  ],
  'sail-annotation-marker': [
    { name: 'x', type: 'number', desc: 'X position.' },
    { name: 'y', type: 'number', desc: 'Y position.' },
  ],
}

export function packBlurbFor(tag: string): string {
  switch (tag) {
    case 'sail-area-chart':
      return 'Multi-series area chart with stacked fills. Series JSON + label list.'
    case 'sail-bar-chart':
      return 'Vertical bars, one per label. CSV data string + label list.'
    case 'sail-donut':
      return 'Stroked-arc segments, sized by value. JSON label/value/tone.'
    case 'sail-gauge':
      return 'Single-arc gauge, value 0..max. Auto-centers the indicator.'
    case 'sail-heatmap':
      return 'Grid of cells, colored by value. rows × cols × values.'
    case 'sail-line-chart':
      return 'Multi-series line chart with markers and grid. Series JSON + label list.'
    case 'sail-sparkline':
      return 'Compact single-line trend. CSV data, width × height.'
    case 'sail-media-image':
      return 'Image primitive with src/alt + framework theming.'
    case 'sail-media-av':
      return 'Audio/video primitive with platform controls + framework skin.'
    case 'sail-code-editor':
      return 'CodeMirror 6 wrapped as a primitive; controlled value + language prop.'
    case 'sail-mermaid':
      return 'Mermaid-rendered SVG; pass the source as the value prop.'
    case 'sail-message':
      return 'A single chat message; from, status, content, timestamp props.'
    case 'sail-citation':
      return 'Citation pill with source/label; click to follow href.'
    case 'sail-composer':
      return 'Chat composer: multiline grow by default, Enter submits, Shift+Enter newline.'
    case 'sail-thinking-dots':
      return 'Three bouncing dots with polite live status.'
    case 'sail-tool-call':
      return 'Tool-call disclosure with name + status (running|ok|error).'
    case 'sail-canvas':
      return 'The pannable/zoomable world layer canvas-pack components mount on.'
    case 'sail-board':
      return 'A bounded canvas surface hosting a world layer.'
    case 'sail-draw':
      return 'Freehand drawing surface over the world layer.'
    case 'sail-graph-canvas':
      return 'Node/edge graph canvas; JSON nodes + edges.'
    case 'sail-annotation-layer':
      return 'Click-to-annotate overlay for the world layer.'
    case 'sail-annotation-list':
      return 'List view of the annotations dropped on a layer.'
    case 'sail-annotation-marker':
      return 'A single point annotation marker at x/y.'
    default:
      return 'Pack component.'
  }
}

export function packLabelFor(tag: string): string {
  if (
    tag.startsWith('sail-area-chart') ||
    tag.startsWith('sail-bar-chart') ||
    tag.startsWith('sail-line-chart') ||
    tag === 'sail-donut' ||
    tag === 'sail-gauge' ||
    tag === 'sail-heatmap' ||
    tag === 'sail-sparkline'
  ) {
    return 'Pack: charts'
  }
  if (tag.startsWith('sail-media-')) return 'Pack: media'
  if (
    tag === 'sail-canvas' ||
    tag === 'sail-board' ||
    tag === 'sail-draw' ||
    tag === 'sail-graph-canvas' ||
    tag.startsWith('sail-annotation-')
  ) {
    return 'Pack: canvas'
  }
  if (
    tag.startsWith('sail-message') ||
    tag === 'sail-citation' ||
    tag === 'sail-composer' ||
    tag === 'sail-thinking-dots' ||
    tag === 'sail-tool-call'
  ) {
    return 'Pack: chat'
  }
  if (tag === 'sail-mermaid') return 'Pack: mermaid'
  if (tag === 'sail-code-editor') return 'Pack: code-editor'
  return 'Pack'
}

export function packSlotCopy(tag: string): string {
  if (
    tag === 'sail-canvas' ||
    tag === 'sail-board' ||
    tag === 'sail-draw' ||
    tag === 'sail-graph-canvas' ||
    tag.startsWith('sail-annotation-')
  ) {
    return 'The world layer — pans, zooms, and drops annotations on click.'
  }
  return ''
}

/** Unsplash demo for sail-media-image (src is JS identity, not a reflect attr). */
export const DEMO_PHOTO =
  'https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=960&h=640&fit=crop&auto=format&q=80'
