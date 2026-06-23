import { controller, derived, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-file-upload.sui'

function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}

interface DisplayFile {
  file: File
  idx: number
  key: string
  name: string
  sizeStr: string
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const files = signal<File[]>([])
    const over = signal(false)
    const errorMsg = signal('')
    const dropzoneEl = ref<HTMLDivElement>(null)
    const inputEl = ref<HTMLInputElement>(null)
    const fileListEl = ref<HTMLDivElement>(null)
    const displayFiles = derived<DisplayFile[]>(() =>
      files().map((file, idx) => ({
        file,
        idx,
        key: `${idx}:${file.name}:${file.size}`,
        name: file.name,
        sizeStr: fmtSize(file.size),
      })),
    )
    return { files, over, errorMsg, displayFiles, dropzoneEl, inputEl, fileListEl }
  },

  mount(state, host) {
    const matchesAccept = (file: File): boolean => {
      const tokens = (state.accept as string)
        .split(',')
        .map((t: string) => t.trim().toLowerCase())
        .filter(Boolean)
      if (tokens.length === 0) return true
      const ext = '.' + (file.name.split('.').pop() || '').toLowerCase()
      const type = file.type.toLowerCase()
      return tokens.some((t: string) => {
        if (t.startsWith('.')) return ext === t
        if (t.endsWith('/*')) return type.startsWith(t.slice(0, -1))
        return type === t
      })
    }

    const emitChange = (list: File[]) => {
      host.dispatchEvent(
        new CustomEvent('sui-change', {
          detail: { files: [...list] },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const addFiles = (incoming: File[]) => {
      state.errorMsg = ''
      const valid: File[] = []
      for (const f of incoming) {
        if (f.size > (state.maxSize as number)) {
          state.errorMsg = `\"${f.name}\" exceeds max size (${fmtSize(state.maxSize as number)}).`
          continue
        }
        if (state.accept && !matchesAccept(f)) {
          state.errorMsg = `"${f.name}" is not an accepted file type.`
          continue
        }
        valid.push(f)
      }
      const cur = state.files
      const next = state.multiple ? [...cur, ...valid] : valid.slice(0, 1)
      state.files = next
      emitChange(next)
    }

    const removeAt = (idx: number) => {
      const cur = state.files
      if (idx < 0 || idx >= cur.length) return
      const next = cur.filter((_: File, i: number) => i !== idx)
      state.files = next
      state.errorMsg = ''
      emitChange(next)
    }

    const openPicker = () => {
      if (state.disabled) return
      ;(state.inputEl as HTMLInputElement | null)?.click()
    }

    const onClick = () => openPicker()
    const onKey = (e: KeyboardEvent) => {
      if (state.disabled) return
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        openPicker()
      }
    }
    const onDragOver = (e: DragEvent) => {
      if (state.disabled) return
      e.preventDefault()
      state.over = true
    }
    const onDragLeave = () => { state.over = false }
    const onDrop = (e: DragEvent) => {
      if (state.disabled) return
      e.preventDefault()
      state.over = false
      const list = e.dataTransfer?.files
      if (list && list.length) addFiles(Array.from(list))
    }
    const onInputChange = (e: Event) => {
      const inp = e.target as HTMLInputElement
      if (inp.files && inp.files.length) addFiles(Array.from(inp.files))
      inp.value = ''
    }

    const dropzoneEl = state.dropzoneEl as HTMLDivElement | null
    const inputEl = state.inputEl as HTMLInputElement | null

    dropzoneEl?.addEventListener('click', onClick)
    dropzoneEl?.addEventListener('keydown', onKey)
    dropzoneEl?.addEventListener('dragover', onDragOver)
    dropzoneEl?.addEventListener('dragleave', onDragLeave)
    dropzoneEl?.addEventListener('drop', onDrop)
    inputEl?.addEventListener('change', onInputChange)

    // Event delegation for the dynamic .remove buttons inside the file list.
    const onShadowClick = (e: Event) => {
      const target = e.target as HTMLElement | null
      const btn = target?.closest?.('button.remove') as HTMLElement | null
      if (!btn || !host.shadowRoot?.contains(btn)) return
      const idx = Number(btn.dataset.idx)
      if (Number.isInteger(idx)) removeAt(idx)
    }
    host.shadowRoot?.addEventListener('click', onShadowClick)

    return () => {
      dropzoneEl?.removeEventListener('click', onClick)
      dropzoneEl?.removeEventListener('keydown', onKey)
      dropzoneEl?.removeEventListener('dragover', onDragOver)
      dropzoneEl?.removeEventListener('dragleave', onDragLeave)
      dropzoneEl?.removeEventListener('drop', onDrop)
      inputEl?.removeEventListener('change', onInputChange)
      host.shadowRoot?.removeEventListener('click', onShadowClick)
    }
  },
})
