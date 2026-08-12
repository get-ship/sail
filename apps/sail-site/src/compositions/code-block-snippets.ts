/** Snippet strings for the code-block docs host.
 *  Kept out of the .sail file so literal markup is not parsed as template. */

const close = (tag: string) => `</${tag}>`

export function codeBlockSnippet(source: string, lang = 'sail'): string {
  const escaped = source.replace(/\\/g, '\\\\').replace(/'/g, "\\'")
  return `<code-block code={'${escaped}'} lang="${lang}">${close('code-block')}`
}

export const CB_SIGNAL = 'const n = signal(0)\nconst double = derived(() => n() * 2)'
export const CB_CSS = '.chip {\n  border-radius: var(--radius-full);\n}'
export const CB_MARKUP = '<button variant="solid" tone="primary">Save</button>'
export const CB_PLAIN = '// your source here'

export const cbExBasic = () => codeBlockSnippet(CB_SIGNAL, 'sail')
export const cbExMarkup = () => codeBlockSnippet(CB_MARKUP, 'sail')
export const cbExPlain = () => codeBlockSnippet(CB_PLAIN)

export function sampleForLang(lang: string): string {
  if (lang === 'ts') return CB_SIGNAL
  if (lang === 'css') return CB_CSS
  return CB_MARKUP
}
