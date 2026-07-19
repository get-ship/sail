import { compileSui } from '@ship/sail-dom/test-utils'

const source = `<div data-test="specimen">hello-from-view</div>`
const out = compileSui(source)
console.log(out.js)
