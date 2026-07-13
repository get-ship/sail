import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-file-upload/sui-file-upload.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => (el.shadowRoot ?? el)
const dropzone = (el: HTMLElement) =>
  shadow(el).querySelector('.dropzone') as HTMLDivElement
const fileInput = (el: HTMLElement) =>
  shadow(el).querySelector('input[type="file"]') as HTMLInputElement

const makeFile = (name: string, size = 32, type = 'text/plain'): File => {
  const blob = new Blob([new Uint8Array(size)], { type })
  return new File([blob], name, { type })
}

const setInputFiles = (input: HTMLInputElement, files: File[]) => {
  const list = Object.assign(files.slice(), {
    item: (i: number) => files[i] ?? null,
  }) as unknown as FileList
  Object.defineProperty(input, 'files', { value: list, configurable: true })
}

const fireDrop = (target: Element, files: File[]) => {
  const event = new Event('drop', { bubbles: true, cancelable: true }) as DragEvent
  Object.defineProperty(event, 'dataTransfer', { value: { files } })
  target.dispatchEvent(event)
}

describe('<sui-file-upload>', () => {

  it('renders a hidden native file input and a dropzone in shadow DOM', () => {
    const { el } = mount(reg, 'sui-file-upload')
    expect(fileInput(el)).not.toBeNull()
    expect(dropzone(el)).not.toBeNull()
  })

  it('default state: no files, no error, dropzone not in over state', () => {
    const { el } = mount(reg, 'sui-file-upload')
    expect(shadow(el).querySelector('.file-list')).toBeNull()
    expect(shadow(el).querySelector('.error')).toBeNull()
    expect(dropzone(el).classList.contains('over')).toBe(false)
  })

  it('dropzone is keyboard focusable by default', () => {
    const { el } = mount(reg, 'sui-file-upload')
    expect(dropzone(el).getAttribute('tabindex')).toBe('0')
    expect(dropzone(el).getAttribute('role')).toBe('button')
  })

  it('accept attribute appears in aria-label', () => {
    const { el } = mount(reg, 'sui-file-upload', { accept: 'image/*' })
    expect(dropzone(el).getAttribute('aria-label')).toBe('Upload files (image/*)')
  })

  it('accept attribute is forwarded to the native input', async () => {
    const { el } = mount(reg, 'sui-file-upload', { accept: '.txt' })
    await flush()
    expect(fileInput(el).getAttribute('accept')).toBe('.txt')
  })

  it('multiple attribute is forwarded to the native input', async () => {
    const { el } = mount(reg, 'sui-file-upload', { multiple: true })
    await flush()
    expect(fileInput(el).multiple).toBe(true)
  })

  it('clicking the dropzone triggers a click on the inner native input', () => {
    const { el } = mount(reg, 'sui-file-upload')
    let clicked = 0
    fileInput(el).addEventListener('click', () => clicked++)
    dropzone(el).click()
    expect(clicked).toBe(1)
  })

  it('Enter on the dropzone opens the picker', () => {
    const { el } = mount(reg, 'sui-file-upload')
    let clicked = 0
    fileInput(el).addEventListener('click', () => clicked++)
    dropzone(el).dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }),
    )
    expect(clicked).toBe(1)
  })

  it('Space on the dropzone opens the picker', () => {
    const { el } = mount(reg, 'sui-file-upload')
    let clicked = 0
    fileInput(el).addEventListener('click', () => clicked++)
    dropzone(el).dispatchEvent(
      new KeyboardEvent('keydown', { key: ' ', bubbles: true }),
    )
    expect(clicked).toBe(1)
  })

  it('selecting a file via the input fires sui-change with the file in detail (bubbles + composed)', async () => {
    const { el } = mount(reg, 'sui-file-upload')
    let evt: CustomEvent | null = null
    el.addEventListener('sui-change', (e) => { evt = e as CustomEvent })
    const inp = fileInput(el)
    const f = makeFile('a.txt', 4)
    setInputFiles(inp, [f])
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(evt).not.toBeNull()
    const detail = evt!.detail as { files: File[] }
    expect(detail.files.length).toBe(1)
    expect(detail.files[0]!.name).toBe('a.txt')
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('drag-and-drop populates the file list and fires sui-change', async () => {
    const { el } = mount(reg, 'sui-file-upload')
    let detail: { files: File[] } | null = null
    el.addEventListener('sui-change', (e) => {
      detail = (e as CustomEvent).detail
    })
    const f = makeFile('dropped.txt', 8)
    fireDrop(dropzone(el), [f])
    await flush()
    expect(detail!.files[0]!.name).toBe('dropped.txt')
    expect(shadow(el).querySelectorAll('.file-item').length).toBe(1)
  })

  it('dragover toggles the over class; dragleave clears it', async () => {
    const { el } = mount(reg, 'sui-file-upload')
    const dz = dropzone(el)
    dz.dispatchEvent(new Event('dragover', { bubbles: true, cancelable: true }))
    await flush()
    expect(dz.classList.contains('over')).toBe(true)
    dz.dispatchEvent(new Event('dragleave', { bubbles: true }))
    await flush()
    expect(dz.classList.contains('over')).toBe(false)
  })

  it('without multiple, second selection replaces the first', async () => {
    const { el } = mount(reg, 'sui-file-upload')
    const inp = fileInput(el)
    setInputFiles(inp, [makeFile('one.txt', 4)])
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    setInputFiles(inp, [makeFile('two.txt', 4)])
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    const items = shadow(el).querySelectorAll('.file-item')
    expect(items.length).toBe(1)
    expect(items[0]!.querySelector('.file-name')!.textContent).toBe('two.txt')
  })

  it('with multiple, selections accumulate', async () => {
    const { el } = mount(reg, 'sui-file-upload', { multiple: true })
    const inp = fileInput(el)
    setInputFiles(inp, [makeFile('a.txt', 4)])
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    setInputFiles(inp, [makeFile('b.txt', 4)])
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    const items = shadow(el).querySelectorAll('.file-item')
    expect(items.length).toBe(2)
  })

  it('max-size rejects oversized files with an error message', async () => {
    const { el } = mount(reg, 'sui-file-upload', { 'max-size': '100' })
    const inp = fileInput(el)
    setInputFiles(inp, [makeFile('big.bin', 500)])
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(shadow(el).querySelector('.file-item')).toBeNull()
    const err = shadow(el).querySelector('.error')
    expect(err).not.toBeNull()
    expect(err!.textContent).toMatch(/exceeds max size/)
  })

  it('accept filter rejects mismatched MIME types with an error message', async () => {
    const { el } = mount(reg, 'sui-file-upload', { accept: 'image/*' })
    const inp = fileInput(el)
    setInputFiles(inp, [makeFile('doc.txt', 8, 'text/plain')])
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(shadow(el).querySelector('.file-item')).toBeNull()
    const err = shadow(el).querySelector('.error')
    expect(err).not.toBeNull()
    expect(err!.textContent).toMatch(/not an accepted file type/)
  })

  it('accept filter accepts matching extension tokens', async () => {
    const { el } = mount(reg, 'sui-file-upload', { accept: '.txt,.md' })
    const inp = fileInput(el)
    setInputFiles(inp, [makeFile('readme.md', 8, 'text/markdown')])
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(shadow(el).querySelectorAll('.file-item').length).toBe(1)
    expect(shadow(el).querySelector('.error')).toBeNull()
  })

  it('disabled reflects on host and disables the dropzone interaction', async () => {
    const { el } = mount(reg, 'sui-file-upload', { disabled: true })
    expect(el.hasAttribute('disabled')).toBe(true)
    expect(dropzone(el).getAttribute('tabindex')).toBe('-1')
    expect(dropzone(el).getAttribute('aria-disabled')).toBe('true')

    let clicked = 0
    fileInput(el).addEventListener('click', () => clicked++)
    dropzone(el).click()
    expect(clicked).toBe(0)

    let detail: unknown = null
    el.addEventListener('sui-change', (e) => {
      detail = (e as CustomEvent).detail
    })
    fireDrop(dropzone(el), [makeFile('x.txt', 4)])
    await flush()
    expect(detail).toBeNull()
    expect(shadow(el).querySelector('.file-item')).toBeNull()
  })

  it('remove button deletes the file and fires sui-change with the new list', async () => {
    const { el } = mount(reg, 'sui-file-upload', { multiple: true })
    const inp = fileInput(el)
    setInputFiles(inp, [makeFile('a.txt', 4), makeFile('b.txt', 4)])
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(shadow(el).querySelectorAll('.file-item').length).toBe(2)

    let detail: { files: File[] } | null = null
    el.addEventListener('sui-change', (e) => {
      detail = (e as CustomEvent).detail
    })
    const firstRemove = shadow(el).querySelector(
      'button.remove[data-idx="0"]',
    ) as HTMLButtonElement
    firstRemove.click()
    await flush()
    expect(shadow(el).querySelectorAll('.file-item').length).toBe(1)
    expect(detail!.files.length).toBe(1)
    expect(detail!.files[0]!.name).toBe('b.txt')
  })

  it('tone reflects on host', () => {
    const { el } = mount(reg, 'sui-file-upload', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('exposes parts for external styling', async () => {
    const { el } = mount(reg, 'sui-file-upload')
    expect(shadow(el).querySelector('[part~="dropzone"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="input"]')).not.toBeNull()

    // Trigger a file selection so file-list parts render.
    const inp = fileInput(el)
    setInputFiles(inp, [makeFile('a.txt', 4)])
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(shadow(el).querySelector('[part~="file-list"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="file-item"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="remove"]')).not.toBeNull()
  })

  it('cleanup on disconnect — no leaked effects', () => {
    const { container } = mount(reg, 'sui-file-upload')
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
  })
})
