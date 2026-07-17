// Layout examples for the deck Layouts page.

export const layoutFile = `<!-- routes/_layout.sail -->
<sail-site-shell></sail-site-shell>`

export const layoutChain = `routes/
  _layout.sail          wraps every route
  blog/
    _layout.sail        wraps /blog/* as well
    page.sail`
