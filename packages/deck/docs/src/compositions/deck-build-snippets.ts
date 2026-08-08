// Build & deploy page snippets. Plain data module — never scope-scanned.
// Output tree braces would parse as interpolation inside a template.

export const buildCmd = `$ sail build --project . --bundle-framework`

export const buildOut = `dist/
  index.html                      importmap shell
  _deck/{runtime,dom,route}.js    framework singletons, materialized
  assets/page-<hash>.js           one lazy chunk per route`

export const baseCmd = `$ sail build --project . --bundle-framework --base assets
  ▸ asset URLs become /assets/<file>`
