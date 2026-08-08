// Layout examples for the Layouts page. Plain data module — never scope-scanned
// by the compiler.

export const layoutFile = `<!-- routes/_layout.sail -->
<sail-shell></sail-shell>`

export const layoutChain = `routes/
  _layout.sail          wraps every route
  blog/
    _layout.sail        wraps /blog/* as well
    page.sail`
