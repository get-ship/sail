/**
 * Dynamic language loaders for <ship-code-editor>.
 *
 * Each language extension is imported on demand so the base bundle
 * only includes the editor core and theme.
 */

import type { Extension } from '@codemirror/state'
import { sailHighlight } from './sail-code-editor.sail-highlight'

export type SupportedLanguage =
  | 'yaml' | 'json' | 'jsonc' | 'javascript' | 'typescript'
  | 'rust' | 'markdown' | 'html' | 'css' | 'shell' | 'python' | 'sail'

export async function loadLanguage(lang: string): Promise<Extension> {
  switch (lang as SupportedLanguage) {
    case 'yaml': {
      const { yaml } = await import('@codemirror/lang-yaml')
      return yaml()
    }
    case 'json':
    case 'jsonc': {
      const { json } = await import('@codemirror/lang-json')
      return json()
    }
    case 'javascript': {
      const { javascript } = await import('@codemirror/lang-javascript')
      return javascript()
    }
    case 'typescript': {
      const { javascript } = await import('@codemirror/lang-javascript')
      return javascript({ typescript: true })
    }
    case 'rust': {
      const { rust } = await import('@codemirror/lang-rust')
      return rust()
    }
    case 'markdown': {
      const { markdown } = await import('@codemirror/lang-markdown')
      return markdown()
    }
    // SAIL is HTML-shaped (tags, attributes, {expr} + {#directives}); base the
    // highlighting on the HTML mode so tags/attributes/text are colored,
    // then layer the SAIL directive + interpolation overlay on top.
    case 'sail': {
      const { html: htmlLang } = await import('@codemirror/lang-html')
      return [htmlLang(), sailHighlight]
    }
    case 'html': {
      const { html: htmlLang } = await import('@codemirror/lang-html')
      return htmlLang()
    }
    case 'css': {
      const { css: cssLang } = await import('@codemirror/lang-css')
      return cssLang()
    }
    case 'python': {
      const { python } = await import('@codemirror/lang-python')
      return python()
    }
    case 'shell':
    default:
      return []
  }
}
