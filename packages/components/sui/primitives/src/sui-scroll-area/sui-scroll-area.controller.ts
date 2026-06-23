import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-scroll-area.sui'

export default controller<Scope>({
  mount(state, host) {
    let scrollTimer: ReturnType<typeof setTimeout> | null = null

    const updateThumbs = () => {
      const vp = state.$signals.viewportEl() as HTMLElement | null
      if (!vp) return

      const orientation = state.$signals.orientation()
      const showV = orientation === 'vertical' || orientation === 'both'
      const showH = orientation === 'horizontal' || orientation === 'both'
      const vThumb = state.$signals.vThumb() as HTMLElement | null
      const hThumb = state.$signals.hThumb() as HTMLElement | null

      if (showV && vThumb) {
        const ratio = vp.clientHeight / vp.scrollHeight
        if (ratio < 1) {
          const thumbH = Math.max(24, ratio * vp.clientHeight)
          const scrollRatio = vp.scrollTop / (vp.scrollHeight - vp.clientHeight)
          const thumbTop = scrollRatio * (vp.clientHeight - thumbH)
          vThumb.style.height = `${thumbH}px`
          vThumb.style.top = `${thumbTop}px`
          vThumb.style.display = ''
        } else {
          vThumb.style.display = 'none'
        }
      }

      if (showH && hThumb) {
        const ratio = vp.clientWidth / vp.scrollWidth
        if (ratio < 1) {
          const thumbW = Math.max(24, ratio * vp.clientWidth)
          const scrollRatio = vp.scrollLeft / (vp.scrollWidth - vp.clientWidth)
          const thumbLeft = scrollRatio * (vp.clientWidth - thumbW)
          hThumb.style.width = `${thumbW}px`
          hThumb.style.left = `${thumbLeft}px`
          hThumb.style.display = ''
        } else {
          hThumb.style.display = 'none'
        }
      }
    }

    const onScroll = () => {
      updateThumbs()
      const vBar = state.$signals.vScrollbar() as HTMLElement | null
      const hBar = state.$signals.hScrollbar() as HTMLElement | null
      vBar?.classList.add('active')
      hBar?.classList.add('active')
      if (scrollTimer) clearTimeout(scrollTimer)
      scrollTimer = setTimeout(() => {
        vBar?.classList.remove('active')
        hBar?.classList.remove('active')
      }, 1200)
    }

    const vp = state.$signals.viewportEl() as HTMLElement | null
    if (vp) {
      vp.addEventListener('scroll', onScroll, { passive: true })
    }
    updateThumbs()

    return () => {
      if (scrollTimer) clearTimeout(scrollTimer)
      if (vp) vp.removeEventListener('scroll', onScroll)
    }
  },
})
