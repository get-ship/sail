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

import type { Action, ControllerScope } from '@ship/sail-runtime'
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

export default controller((_ctx: ControllerScope<Scope>) => {
    const files = signal<File[]>([])
    const over = signal(false)
    const errorMsg = signal('')
    // KEPT as a ref, deliberately: `openPicker` clicks this input from the
    // DROPZONE's handlers. The element is needed as a VALUE reachable from
    // another element's behavior, which is the one thing an action can't do.
    const inputEl = signal<HTMLInputElement | null>(null)
    const displayFiles = derived<DisplayFile[]>(() =>
      files().map((file, idx) => ({
        file,
        idx,
        key: `${idx}:${file.name}:${file.size}`,
        name: file.name,
        sizeStr: fmtSize(file.size),
      })),
    )

    // The change event is dispatched from the host. `mount` sets this before
    // any handler below can fire (actions only REGISTER at attach time).
    let hostEl: HTMLElement | null = null

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
      hostEl?.dispatchEvent(
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

    // The dropzone's five listeners are behavior ON the dropzone, so they ride
    // an action: it receives the element and owns the removal. NOT named
    // `dropzone` — that is a BUILT-IN action name, and a scope member that
    // shadows one is silently ignored in favour of the auto-import.
    const dropTarget: Action<void> = (node) => {
      node.addEventListener('click', onClick)
      node.addEventListener('keydown', onKey as EventListener)
      node.addEventListener('dragover', onDragOver as EventListener)
      node.addEventListener('dragleave', onDragLeave)
      node.addEventListener('drop', onDrop as EventListener)
      return {
        destroy: () => {
          node.removeEventListener('click', onClick)
          node.removeEventListener('keydown', onKey as EventListener)
          node.removeEventListener('dragover', onDragOver as EventListener)
          node.removeEventListener('dragleave', onDragLeave)
          node.removeEventListener('drop', onDrop as EventListener)
        },
      }
    }

    mount((host) => {
      hostEl = host
      const inputElNode = inputEl() as HTMLInputElement | null
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
        inputElNode?.removeEventListener('change', onInputChange)
        ;(host.shadowRoot ?? host).removeEventListener('click', onShadowClick)
      }
    })

    return { files, over, errorMsg, displayFiles, dropTarget, inputEl }
  })

</controller>
<center gap="2" data-state={over() ? 'over' : null} role="button" tabindex={disabled() ? '-1' : '0'} aria-label={accept() ? 'Upload files (' + accept() + ')' : 'Upload files'} aria-disabled={disabled() ? 'true' : 'false'} use:dropTarget sx={style.group().padY(6).padX(4).borderWidth(1).border(border).border(dashed).radius(lg).cursor(pointer).fg(muted-foreground).select(none).textAlign(center).when({ focus: style.outline(ring), stateOver: style.borderWidth(1).border(ring).bg(muted) })}>
  <row align="center" justify="center" gap="0" aria-hidden="true" sx={style.w(9).h(9).minW(9).radius(lg).bg(muted).fg(muted-foreground).fontSize(lg).leading(none).when({ groupStateOver: style.bg(selected-bg).fg(ring) })}>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" sx={style.w(icon-size-md).h(icon-size-md)}><path d="M12 16V4M7 9l5-5 5 5"/><path d="M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"/></svg>
  </row>
  <span>Drag &amp; drop or <span sx={style.fg(primary)}>browse</span></span>
</center>
<input type="file" sx={style.display(none)} accept={accept()} multiple={multiple()} aria-hidden="true" tabindex="-1" bind:this={inputEl} />
<if {errorMsg()}>
  <div role="alert" sx={style.fontSize(sm).fg(destructive)}>{errorMsg()}</div>
</if>
<if {displayFiles().length > 0}>
  <column gap="1" aria-live="polite" sx={style.mt(2)}>
    <for {displayFiles()} as="f" key={f.key}>
      <row align="center" gap="3">
        <span aria-hidden="true" sx={style.fg(muted-foreground)}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" sx={style.w(icon-size-sm).h(icon-size-sm)}><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2 2V8z"/><path d="M14 2v6h6"/></svg>
        </span>
        <column gap="1" grow min-w="0">
          <span sx={style.fontSize(sm).fg(foreground).truncate()}>{f.name}</span>
          <span sx={style.fontSize(xs).fg(muted-foreground)}>{f.sizeStr}</span>
        </column>
        <button variant="ghost" size="sm" type="button" data-idx={f.idx} aria-label={'Remove ' + f.name}>×</button>
      </row>
    </for>
  </column>
</if>
