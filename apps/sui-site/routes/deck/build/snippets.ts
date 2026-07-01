// Build &amp; deploy examples for the deck Build page.

export const buildCmd = `$ deck build --project . --bundle-framework`

export const buildOut = `dist/
  index.html                      importmap shell
  _deck/{runtime,dom,route}.js    framework singletons, materialized
  assets/page-<hash>.js           one lazy chunk per route`

export const baseCmd = `$ deck build --project . --bundle-framework --base assets
  ▸ asset URLs become /assets/<file>`
