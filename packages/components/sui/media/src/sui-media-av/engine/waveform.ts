// sui-media engine — audio waveform peak decode + draw. Extracted from the
// renderer's `mount` so the controller stays glue-only (build-spec Slice 7
// pre-committed split). PURE of `@ship/sail-runtime`; it touches only the
// canvas it is handed and the resolved bytes it is given.
//
// Source-in, never fetch: peaks are decoded from already-resolved `bytes`
// (the consumer fetched them); when no decodable bytes are present the helper
// draws a neutral idle baseline. No I/O.
//
// Authority: build-spec Slice 7 (waveform decode), §Accessibility (the canvas
// is decorative/aria-hidden; the real <audio> stays the accessible control).

/** Number of vertical bars the waveform is reduced to. */
const BAR_COUNT = 96

/** Decode resolved audio bytes into `BAR_COUNT` normalized peaks (0..1). When
 *  no AudioContext is available (headless) or bytes are absent, returns an
 *  idle baseline so the decorative canvas still paints. */
export async function decodePeaks(bytes: ArrayBuffer | Uint8Array | null): Promise<number[]> {
  const idle = (): number[] => Array.from({ length: BAR_COUNT }, () => 0.15)
  if (!bytes) return idle()

  const Ctor =
    (globalThis as { AudioContext?: typeof AudioContext; webkitAudioContext?: typeof AudioContext })
      .AudioContext ??
    (globalThis as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext
  if (!Ctor) return idle()

  const buffer = bytes instanceof Uint8Array ? (bytes.buffer as ArrayBuffer) : bytes
  try {
    const ctx = new Ctor()
    const audio = await ctx.decodeAudioData(buffer.slice(0))
    const channel = audio.getChannelData(0)
    const block = Math.floor(channel.length / BAR_COUNT) || 1
    const peaks: number[] = []
    for (let i = 0; i < BAR_COUNT; i++) {
      let max = 0
      for (let j = 0; j < block; j++) {
        const v = Math.abs(channel[i * block + j] ?? 0)
        if (v > max) max = v
      }
      peaks.push(max)
    }
    void ctx.close()
    return peaks
  } catch {
    return idle()
  }
}

/** Draw normalized peaks onto a canvas as centered vertical bars. Reads the
 *  current foreground token off the canvas's computed style so it themes. */
export function drawWaveform(canvas: HTMLCanvasElement, peaks: number[]): void {
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  const w = canvas.width || canvas.clientWidth || 320
  const h = canvas.height || canvas.clientHeight || 48
  canvas.width = w
  canvas.height = h
  ctx.clearRect(0, 0, w, h)

  const stroke = getComputedStyle(canvas).color || 'currentColor'
  ctx.fillStyle = stroke
  const barW = w / peaks.length
  const gap = Math.max(1, barW * 0.3)
  for (let i = 0; i < peaks.length; i++) {
    const barH = Math.max(2, peaks[i]! * h)
    const x = i * barW
    const y = (h - barH) / 2
    ctx.fillRect(x, y, Math.max(1, barW - gap), barH)
  }
}
