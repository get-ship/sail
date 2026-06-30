// Layout examples for the Layouts page. Plain data module — never scope-scanned
// by the compiler.

export const layoutFile = `<!-- routes/_layout.sui -->
<sui-shell></sui-shell>`

export const layoutChain = `routes/
  _layout.sui          wraps every route
  blog/
    _layout.sui        wraps /blog/* as well
    page.sui`
