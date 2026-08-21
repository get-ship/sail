<!--
  <sail-file-upload> — drag-and-drop zone with removable file list.
  Emits sail-change { files }; max-size / accept gate incoming files.
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
import { fmtSize, matchesAccept } from './file-math'

interface DisplayFile {
  file: SailFile
  idx: number
  key: string
  name: string
  sizeStr: string
}

export default controller((_ctx: ControllerScope<Scope>) => {
    const files = signal<SailFile[]>([])
    const over = signal(false)
    const errorMsg = signal('')
    // Cross-element ref: openPicker clicks this input from the dropzone handlers.
    const inputEl = signal<HostBag | null>(null)
    const captureInput: Action<void> = (node) => {
      inputEl.set(node)
      return { destroy: () => inputEl.set(null) }
    }
    const displayFiles = derived<DisplayFile[]>(() =>
      files().map((file, idx) => ({
        file,
        idx,
        key: `${idx}:${file.name}:${file.size}`,
        name: file.name,
        sizeStr: fmtSize(file.size),
      })),
    )

    let hostEl: HostBag | null = null

    const emitChange = (list: SailFile[]) => {
      ;(hostEl)?.onChange?.({ files: [...list] })
    }

    const addFiles = (incoming: SailFile[]) => {
      errorMsg.set('')
      const valid: SailFile[] = []
      for (const f of incoming) {
        if (f.size > (_ctx.maxSize() as number)) {
          errorMsg.set(`\"${f.name}\" exceeds max size (${fmtSize(_ctx.maxSize() as number)}).`)
          continue
        }
        if (_ctx.accept() && !matchesAccept(f, _ctx.accept() as string)) {
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
      const next = cur.filter((_: SailFile, i: number) => i !== idx)
      files.set(next)
      errorMsg.set('')
      emitChange(next)
    }

    const openPicker = () => {
      if (_ctx.disabled()) return
      ;(inputEl() as HostBag | null)?.click()
    }

    const onClick = () => openPicker()
    const onKey = (e: DomEv) => {
      if (_ctx.disabled()) return
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        openPicker()
      }
    }
    const onDragOver = (e: DomEv) => {
      if (_ctx.disabled()) return
      e.preventDefault()
      over.set(true)
    }
    const onDragLeave = () => { over.set(false) }
    const onDrop = (e: DomEv) => {
      if (_ctx.disabled()) return
      e.preventDefault()
      over.set(false)
      const list = e.dataTransfer?.files
      if (list && list.length) addFiles(Array.from(list))
    }
    const onInputChange = (e: DomEv) => {
      const inp = e.target as HostBag
      if (inp.files && inp.files.length) addFiles(Array.from(inp.files))
      inp.value = ''
    }

    // Named dropTarget (not dropzone) — dropzone is a built-in action name.
    const dropTarget: Action<void> = (node) => {
      node.addEventListener('click', onClick)
      node.addEventListener('keydown', onKey)
      node.addEventListener('dragover', onDragOver)
      node.addEventListener('dragleave', onDragLeave)
      node.addEventListener('drop', onDrop)
      return {
        destroy: () => {
          node.removeEventListener('click', onClick)
          node.removeEventListener('keydown', onKey)
          node.removeEventListener('dragover', onDragOver)
          node.removeEventListener('dragleave', onDragLeave)
          node.removeEventListener('drop', onDrop)
        },
      }
    }

    mount((host) => {
      hostEl = host
      const inputElNode = inputEl() as HostBag | null
      inputElNode?.addEventListener('change', onInputChange)

      const onShadowClick = (e: DomEv) => {
        const target = e.target as HostBag | null
        const btn = target?.closest?.('button[data-idx]') as HostBag | null
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

    return { files, over, errorMsg, displayFiles, dropTarget, captureInput }
  })

</controller>
<center
  gap="2"
  data-state={over() ? 'over' : null}
  role="button"
  tabindex={disabled() ? '-1' : '0'}
  aria-label={accept() ? 'Upload files (' + accept() + ')' : 'Upload files'}
  aria-disabled={disabled() ? 'true' : 'false'}
  use:dropTarget
  style({ pad({ x: 4, y: 6 }).group().border({ color: border, width: 1, style: dashed }).radius(lg).cursor(pointer).fg(muted-foreground).select(none), when: { focus: border({ kind: outline, color: ring }), stateOver: border({ color: ring, width: 1 }).bg(muted) } })
>
  <span style({ display(contents).type({ align: center }) })>
    <row
      align="center"
      justify="center"
      gap="0"
      aria-hidden="true"
      style({ size({ w: 9, h: 9, min-w: 9 }).radius(lg).bg(muted).fg(muted-foreground), when: { groupStateOver: bg(selected-bg).fg(ring) } })
    >
      <span style({ display(contents).type({ size: lg, leading: none }) })>
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          style({ size({ w: icon-size-md, h: icon-size-md }) })
        ><path d="M12 16V4M7 9l5-5 5 5"/><path d="M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"/></svg>
      </span>
    </row>
    <span>Drag &amp; drop or <span style({ fg(primary) })>browse</span></span>
  </span>
</center>
<file
  accept={accept()}
  multiple={multiple()}
  aria-hidden="true"
  tabindex="-1"
  use:captureInput
  style({ display(none) })
/>
<if {errorMsg()}>
  <div role="alert" style({ fg(danger) })>
    <span style({ display(contents).type({ size: sm }) })>
      {errorMsg()}
    </span>
  </div>
</if>
<if {displayFiles().length > 0}>
  <column gap="1" aria-live="polite" style({ margin({ t: 2 }) })>
    <for {displayFiles() as f} key={f.key}>
      <row align="center" gap="3">
        <span aria-hidden="true" style({ fg(muted-foreground) })>
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            style({ size({ w: icon-size-sm, h: icon-size-sm }) })
          ><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2 2V8z"/><path d="M14 2v6h6"/></svg>
        </span>
        <column gap="1" style({ grow(1).size({ min-w: 0 }) })>
          <span style({ fg(foreground).type({ size: sm, wrap: truncate }) })>{f.name}</span>
          <span style({ fg(muted-foreground).type({ size: xs }) })>{f.sizeStr}</span>
        </column>
        <button variant="ghost" size="sm" type="button" data-idx={f.idx} aria-label={'Remove ' + f.name}>×</button>
      </row>
    </for>
  </column>
</if>
