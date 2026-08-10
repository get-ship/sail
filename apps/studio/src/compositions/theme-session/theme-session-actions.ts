/**
 * Theme session shell actions — save / peek / add-to-project / path.
 * Keeps createThemeSession under the hard line budget.
 * No facet overflow menu (delete + remix live on Themes home).
 */
import { navigate } from '@ship/deck/router'
import {
  addThemeToProject,
  getDesignState,
  loadThemes,
  previewInApp,
  requestSaveTheme,
} from '@ship/studio/design'
import {
  confirmAddToProject,
} from './theme-confirm'
import {
  FACET_DOORS,
  PATH_NAV,
  facetCrumb,
  themeFacetHref,
  type ThemeFacet,
} from './theme-nav'
import { themeDisplayName, themeIdFromLocation } from './theme-paint'

export type SessionActionCtx = {
  facet: ThemeFacet
  saveSuccessMessage: string
  themeId: () => string
  canEdit: () => boolean
  setNote: (msg: string, tone?: string) => void
  sync: () => void
  ensureThemeOpen: (id: string) => void
  clearOpened: () => void
  navigateFacet: (f: ThemeFacet) => void
  pathCrumb: { set: (v: string) => void }
  /** Dirty leave guard — requestNavigate(href) may open dialog. */
  requestNavigate: (href: string, reason?: string) => void
}

export function makeSessionShellActions(ctx: SessionActionCtx) {
  const onSave = () => {
    if (!ctx.canEdit()) {
      ctx.setNote('Add this theme to the project before saving.', 'warning')
      return
    }
    void requestSaveTheme().then((ok) => {
      if (ok) ctx.setNote(ctx.saveSuccessMessage, 'success')
      else {
        const ds = getDesignState()
        const err = ds.lastWriteError || ds.statusNote
        if (ds.saveReviewOpen) {
          // Usage band + Confirm save in chrome own the review step.
          const label = ds.blast?.label
          ctx.setNote(
            label && label !== 'Not used in this project yet'
              ? `${label}. Confirm save to apply.`
              : 'Confirm save to apply changes.',
            'warning',
          )
        } else if (err) ctx.setNote(err, 'destructive')
      }
      ctx.sync()
    })
  }

  /**
   * Preview — postMessage bridge for overlay hosts only.
   * Silent on success (no toast theater); warn only when unavailable.
   */
  const onPeek = () => {
    const r = previewInApp()
    if (!r.ok) {
      ctx.setNote(r.message || 'Preview unavailable.', 'warning')
    }
  }

  const onAddToProject = () => {
    const id = ctx.themeId()
    if (!id) return
    if (ctx.canEdit()) {
      ctx.setNote('Already in the project.')
      return
    }
    const newId = id.startsWith('my-') ? id : `my-${id}`
    const name = themeDisplayName(id)
    if (!confirmAddToProject(name, newId)) {
      ctx.setNote('Add to project cancelled.')
      return
    }
    ctx.setNote(`Adding ${name} to the project…`)
    void addThemeToProject(newId, id).then((ok) => {
      if (ok) {
        ctx.setNote(`${themeDisplayName(newId)} is in the project.`, 'success')
        navigate(themeFacetHref(newId, ctx.facet))
        ctx.clearOpened()
        ctx.ensureThemeOpen(newId)
      } else {
        ctx.setNote(
          getDesignState().lastWriteError || 'Could not add that theme.',
          'destructive',
        )
      }
      ctx.sync()
    })
  }

  const isFacet = (v: string): v is ThemeFacet =>
    v === 'overview' ||
    v === 'color' ||
    v === 'shape' ||
    v === 'assets' ||
    v === 'components'

  const onPathSelect = (detail: { value?: string }) => {
    const v = detail?.value || ''
    if (!v || v === ctx.facet) {
      ctx.pathCrumb.set(facetCrumb(ctx.facet))
      return
    }
    if (isFacet(v)) {
      ctx.navigateFacet(v)
      return
    }
    // Unknown value (defensive — PATH_NAV is exhaustive for product facets).
    ctx.pathCrumb.set(facetCrumb(ctx.facet))
    const item = PATH_NAV.find((p) => p.value === v)
    ctx.setNote(`Unknown section “${item?.label ?? v}”.`, 'muted')
  }

  const onFacet = (e: Event) => {
    // Authored attr is `facet=` (data-* banned). dataset.facet only works for data-facet.
    const el = (e.target as Element | null)?.closest?.('[facet]') as HTMLElement | null
    const f = el?.getAttribute('facet') ?? ''
    if (!f) return
    if (isFacet(f)) {
      ctx.navigateFacet(f)
      return
    }
    const door = FACET_DOORS.find((d) => d.facet === f)
    ctx.setNote(`Unknown facet “${door?.label ?? f}”.`, 'muted')
  }

  const onRetry = () => {
    const id = themeIdFromLocation() || ctx.themeId()
    void loadThemes().then(() => {
      if (id) {
        ctx.clearOpened()
        ctx.ensureThemeOpen(id)
      } else ctx.sync()
    })
  }

  return { onSave, onPeek, onAddToProject, onPathSelect, onFacet, onRetry }
}
