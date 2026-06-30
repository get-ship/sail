/**
 * Dynamic language loaders for <sui-code-editor>.
 *
 * Each language extension is imported on demand so the base bundle
 * only includes the editor core + theme. The dynamic imports are
 * justified by exception: the language is runtime-selected via the
 * `language` prop and we don't want every install to ship every parser
 * (~150KB gzipped across all 11 languages).
 */

import type { Extension } from '@codemirror/state'

export type SupportedLanguage =
  | 'yaml' | 'json' | 'jsonc' | 'javascript' | 'typescript'
  | 'rust' | 'markdown' | 'html' | 'css' | 'shell' | 'python'

export async function loadLanguage(lang: string): Promise<Extension> {
  switch (lang as SupportedLanguage) {
    case 'yaml': {
      // Runtime-selected parser — see file header.
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
