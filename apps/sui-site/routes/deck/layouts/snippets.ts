// Layout examples for the deck Layouts page.

export const layoutFile = `<!-- routes/_layout.sui -->
<sui-site-shell></sui-site-shell>`

export const layoutChain = `routes/
  _layout.sui          wraps every route
  blog/
    _layout.sui        wraps /blog/* as well
    page.sui`
