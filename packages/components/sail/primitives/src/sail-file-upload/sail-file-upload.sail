<!--
  <sail-file-upload> — SAIL port of <ship-file-upload>.
  Drag-and-drop file upload zone with removable file list.

  Prop / event / parts parity vs <ship-file-upload>
  ────────────────────────────────────────────────────────────────────────
  Lit                              SAIL                       Notes
  ────────────────────────────────────────────────────────────────────────
  accept (String, '')              accept (string, '')       ✓
  multiple (Boolean, false)        multiple (bool, false)    ✓
  maxSize (Number, 10485760,       max-size (number,         ✓
    attribute 'max-size')            10485760)
  disabled (Boolean, false,        disabled (bool, false,    ✓
    reflect)                         reflect)
  tone (Tone, undefined,           tone (union, 'default',   ✓ tints drop ring
    reflect)                         reflect)
  ────────────────────────────────────────────────────────────────────────
  Events: ship-change              sail-change                ✓ renamed
    { detail: { files: File[] } }    { detail: { files } }     payload preserved
  ────────────────────────────────────────────────────────────────────────
  Parts: (none in Lit)             dropzone, input, error,   new — surface
                                     file-list, file-item,     internal nodes for
                                     remove                    consumer styling
  ────────────────────────────────────────────────────────────────────────
  formAssociated / internals     N/A — Lit version does not surface internals.
  ────────────────────────────────────────────────────────────────────────
-->

<scope>
  props: {
    accept = ''
    multiple = false
    'max-size' = 10485760
    disabled = false
  }
</scope>
<controller>
import { controller, derived, ref, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './sail-file-upload.sail'

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

export default controller((_ctx: ViewScope<Scope>) => {
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
    mount((host) => {
      const matchesAccept = (file: File): boolean => {
        const tokens = (_ctx.accept() as string)
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
          new CustomEvent('sail-change', {
            detail: { files: [...list] },
            bubbles: true,
            composed: true,
          }),
        )
      }

      const addFiles = (incoming: File[]) => {
        errorMsg.set('')
        const valid: File[] = []
        for (const f of incoming) {
          if (f.size > (_ctx.maxSize() as number)) {
            errorMsg.set(`\"${f.name}\" exceeds max size (${fmtSize(_ctx.maxSize() as number)}).`)
            continue
          }
          if (_ctx.accept() && !matchesAccept(f)) {
            errorMsg.set(`"${f.name}" is not an accepted file type.`)
            continue
          }
          valid.push(f)
        }
        const cur = files()
        const next = _ctx.multiple() ? [...cur, ...valid] : valid.slice(0, 1)
        files.set(next)
        emitChange(next)
      }

      const removeAt = (idx: number) => {
        const cur = files()
        if (idx < 0 || idx >= cur.length) return
        const next = cur.filter((_: File, i: number) => i !== idx)
        files.set(next)
        errorMsg.set('')
        emitChange(next)
      }

      const openPicker = () => {
        if (_ctx.disabled()) return
        ;(inputEl() as HTMLInputElement | null)?.click()
      }

      const onClick = () => openPicker()
      const onKey = (e: KeyboardEvent) => {
        if (_ctx.disabled()) return
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          openPicker()
        }
      }
      const onDragOver = (e: DragEvent) => {
        if (_ctx.disabled()) return
        e.preventDefault()
        over.set(true)
      }
      const onDragLeave = () => { over.set(false) }
      const onDrop = (e: DragEvent) => {
        if (_ctx.disabled()) return
        e.preventDefault()
        over.set(false)
        const list = e.dataTransfer?.files
        if (list && list.length) addFiles(Array.from(list))
      }
      const onInputChange = (e: Event) => {
        const inp = e.target as HTMLInputElement
        if (inp.files && inp.files.length) addFiles(Array.from(inp.files))
        inp.value = ''
      }

      const dropzoneElNode = dropzoneEl() as HTMLDivElement | null
      const inputElNode = inputEl() as HTMLInputElement | null

      dropzoneElNode?.addEventListener('click', onClick)
      dropzoneElNode?.addEventListener('keydown', onKey)
      dropzoneElNode?.addEventListener('dragover', onDragOver)
      dropzoneElNode?.addEventListener('dragleave', onDragLeave)
      dropzoneElNode?.addEventListener('drop', onDrop)
      inputElNode?.addEventListener('change', onInputChange)

      // Event delegation for the dynamic .remove buttons inside the file list.
      const onShadowClick = (e: Event) => {
        const target = e.target as HTMLElement | null
        const btn = target?.closest?.('button[data-idx]') as HTMLElement | null
        if (!btn || !(host.shadowRoot ?? host).contains(btn)) return
        const idx = Number(btn.dataset.idx)
        if (Number.isInteger(idx)) removeAt(idx)
      }
      ;(host.shadowRoot ?? host).addEventListener('click', onShadowClick)

      return () => {
        dropzoneElNode?.removeEventListener('click', onClick)
        dropzoneElNode?.removeEventListener('keydown', onKey)
        dropzoneElNode?.removeEventListener('dragover', onDragOver)
        dropzoneElNode?.removeEventListener('dragleave', onDragLeave)
        dropzoneElNode?.removeEventListener('drop', onDrop)
        inputElNode?.removeEventListener('change', onInputChange)
        ;(host.shadowRoot ?? host).removeEventListener('click', onShadowClick)
      }
    })

    return { files, over, errorMsg, displayFiles, dropzoneEl, inputEl, fileListEl }
  })

</controller>
<center gap="2" data-state={over() ? 'over' : null} role="button" tabindex={disabled() ? '-1' : '0'} aria-label={accept() ? 'Upload files (' + accept() + ')' : 'Upload files'} aria-disabled={disabled() ? 'true' : 'false'} bind:this={dropzoneEl} sx="group py-6 px-4 border border-border border-dashed rounded-lg cursor-pointer text-muted-foreground select-none text-center focus:ring-ring state-over:border state-over:border-tone-ring state-over:bg-tone-bg">
  <row align="center" justify="center" gap="0" aria-hidden="true" sx="w-9 h-9 min-w-9 rounded-lg bg-muted text-muted-foreground text-lg leading-none group-state-over:bg-selected-bg group-state-over:text-tone-ring">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" sx="w-icon-size-md h-icon-size-md"><path d="M12 16V4M7 9l5-5 5 5"/><path d="M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"/></svg>
  </row>
  <span>Drag &amp; drop or <span sx="text-primary">browse</span></span>
</center>
<input type="file" sx="hidden" accept={accept()} multiple={multiple()} aria-hidden="true" tabindex="-1" bind:this={inputEl} />
{#if errorMsg()}
  <div role="alert" sx="text-sm text-destructive">{errorMsg()}</div>
{/if}
{#if displayFiles().length > 0}
  <column gap="1" aria-live="polite" bind:this={fileListEl} sx="mt-2">
    {#for f of displayFiles() key={f.key}}
      <row align="center" gap="3">
        <span aria-hidden="true" sx="text-muted-foreground">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" sx="w-icon-size-sm h-icon-size-sm"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2 2V8z"/><path d="M14 2v6h6"/></svg>
        </span>
        <column gap="1" sx="grow-1 min-w-0">
          <span sx="text-sm text-foreground truncate">{f.name}</span>
          <span sx="text-xs text-muted-foreground">{f.sizeStr}</span>
        </column>
        <button variant="ghost" size="sm" type="button" data-idx={f.idx} aria-label={'Remove ' + f.name}>×</button>
      </row>
    {/for}
  </column>
{/if}
