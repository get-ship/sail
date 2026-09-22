import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

function assignFiles(input: HTMLInputElement, files: File[]) {
  try {
    const dt = new DataTransfer()
    for (const f of files) dt.items.add(f)
    input.files = dt.files
  } catch {
    Object.defineProperty(input, 'files', {
      configurable: true,
      get: () => files,
    })
  }
  input.dispatchEvent(new Event('change', { bubbles: true }))
}

describe('l1 file', () => {
  it('is locatable by name', async () => {
    const v = renderRoute(`<file />`)
    await v.flush()
    expect(getByRole(v.root, 'textbox', { name: 'Upload file' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Browse' })).toBeTruthy()
    expect(getByText(v.root, 'Drop a file or click to browse')).toBeTruthy()
    v.unmount()
  })

  it('pick shows the selected file name', async () => {
    const v = renderRoute(`<file />`)
    await v.flush()
    const input = getByRole(v.root, 'textbox', { name: 'Upload file' }) as HTMLInputElement
    assignFiles(input, [new File(['ok'], 'report.pdf', { type: 'application/pdf' })])
    await v.flush()
    expect(getByText(v.root, 'report.pdf')).toBeTruthy()
    v.unmount()
  })

  it('drop shows the selected file name', async () => {
    const v = renderRoute(`<file />`)
    await v.flush()
    const input = getByRole(v.root, 'textbox', { name: 'Upload file' }) as HTMLInputElement
    const file = new File(['ok'], 'notes.txt', { type: 'text/plain' })
    assignFiles(input, [file])
    input.dispatchEvent(new Event('drop', { bubbles: true, cancelable: true }))
    await v.flush()
    expect(getByText(v.root, 'notes.txt')).toBeTruthy()
    v.unmount()
  })
})
