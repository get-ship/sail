// sail-media engine — audio waveform peak decode + draw.
// Pure of DOM constructor names: AudioContext via structural globalThis.

/** Number of vertical bars the waveform is reduced to. */
const BAR_COUNT = 96

type AudioCtx = {
  decodeAudioData(buf: ArrayBuffer): Promise<{ getChannelData(ch: number): ArrayLike<number> }>
  close(): void | Promise<void>
}

/** Decode resolved audio bytes into `BAR_COUNT` normalized peaks (0..1). */
export async function decodePeaks(bytes: ArrayBuffer | Uint8Array | null): Promise<number[]> {
  const idle = (): number[] => Array.from({ length: BAR_COUNT }, () => 0.15)
  if (!bytes) return idle()

  const g = globalThis as {
    AudioContext?: new () => AudioCtx
    webkitAudioContext?: new () => AudioCtx
  }
  const Ctor = g.AudioContext ?? g.webkitAudioContext
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

/** Draw normalized peaks onto a canvas as centered vertical bars. */
export function drawWaveform(
  canvas: {
    getContext(id: string): {
      clearRect(x: number, y: number, w: number, h: number): void
      fillStyle: string
      fillRect(x: number, y: number, w: number, h: number): void
    } | null
    width: number
    height: number
    clientWidth?: number
    clientHeight?: number
  },
  peaks: number[],
): void {
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  const w = canvas.width || canvas.clientWidth || 320
  const h = canvas.height || canvas.clientHeight || 48
  canvas.width = w
  canvas.height = h
  ctx.clearRect(0, 0, w, h)
  ctx.fillStyle = 'currentColor'
  const barW = w / peaks.length
  const gap = Math.max(1, barW * 0.3)
  for (let i = 0; i < peaks.length; i++) {
    const barH = Math.max(2, peaks[i]! * h)
    const x = i * barW
    const y = (h - barH) / 2
    ctx.fillRect(x, y, Math.max(1, barW - gap), barH)
  }
}
