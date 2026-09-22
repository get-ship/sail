// Layout examples for the deck Layouts page.

export const layoutFile = `<!-- routes/_layout.sail -->
<sail-site-header></sail-site-header>`

export const layoutChain = `routes/
  _layout.sail          wraps every route (header)
  sail/
    _layout.sail        wraps /sail/* as well (docs rail)
    overview/
      page.sail`
