// L1 conformance spec: <avatar> (mechanism-1, use:mediaFallback).
// visual.cross: tone · size · shape.

const IMG = 'data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 width=%2240%22 height=%2240%22%3E%3Crect width=%2240%22 height=%2240%22 fill=%22%234a90d9%22/%3E%3Ccircle cx=%2220%22 cy=%2214%22 r=%226%22 fill=%22white%22/%3E%3C/svg%3E'
const TONES = ['primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
const av = (a) => `<span class="sail-avatar" data-sail-host="avatar" ${a}></span>`

export default {
  tag: 'avatar',
  width: 760,
  rows() {
    return [
      `${av('class="l1-mark" name="Ada Lovelace"')} ${av('initials="AL"')} ${av(`src="${IMG}" name="Img"`)} ${av('src="broken-nope.png" name="Broken"')}`,
      TONES.map((t) => av(`tone="${t}" name="${t[0]}"`)).join(' '),
      ['xs', 'sm', 'md', 'lg', 'xl'].map((s) => av(`size="${s}" initials="${s.toUpperCase()}"`)).join(' ') +
        ' ' +
        av('shape="square" size="lg" tone="primary" initials="SQ"') +
        ' ' +
        av('shape="round" size="md" initials="RD"'),
    ]
  },
  async probe() {
    const initialsOf = (el) => {
      const ex = (el.getAttribute('initials') || '').trim()
      if (ex) return ex.slice(0, 2).toUpperCase()
      return ((el.getAttribute('name') || '').trim().slice(0, 1).toUpperCase()) || '?'
    }
    const showInitials = (el) => {
      el.textContent = ''
      const s = document.createElement('span')
      s.setAttribute('part', 'initials')
      s.textContent = initialsOf(el)
      el.appendChild(s)
    }
    document.querySelectorAll('.sail-avatar').forEach((el) => {
      const src = (el.getAttribute('src') || '').trim()
      if (!src) {
        showInitials(el)
        return
      }
      el.textContent = ''
      const img = document.createElement('img')
      img.setAttribute('part', 'img')
      img.alt = el.getAttribute('name') || ''
      img.addEventListener('error', () => showInitials(el), { once: true })
      img.src = src
      el.appendChild(img)
    })
    await new Promise((r) => setTimeout(r, 700))
    const bad = []
    document.querySelectorAll('.sail-avatar').forEach((el) => {
      const hasImg = !!el.querySelector('img')
      const hasInitials = !!el.querySelector('[part=initials]')
      const src = el.getAttribute('src') || ''
      if (!hasImg && !hasInitials) bad.push({ why: 'empty', name: el.getAttribute('name'), src })
      if (src.includes('broken') && !hasInitials) bad.push({ why: 'broken-no-fallback' })
      if (!src && !hasInitials) bad.push({ why: 'no-initials', name: el.getAttribute('name') })
    })
    return { count: document.querySelectorAll('.sail-avatar').length, bad: bad.slice(0, 10) }
  },
}
