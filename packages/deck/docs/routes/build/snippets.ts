// Build & deploy examples for the Build page. Plain data module — never
// scope-scanned by the compiler. The output tree contains `{runtime,dom,route}`,
// which would parse as interpolation inside a template, so it must live here.

export const buildCmd = `$ deck build --project . --bundle-framework`

export const buildOut = `dist/
  index.html                      importmap shell
  _deck/{runtime,dom,route}.js    framework singletons, materialized
  assets/page-<hash>.js           one lazy chunk per route`

export const baseCmd = `$ deck build --project . --bundle-framework --base assets
  ▸ asset URLs become /assets/<file>`
