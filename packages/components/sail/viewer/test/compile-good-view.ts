import { compileSail } from '@ship/sail-dom/test-utils'

const source = `<div data-test="specimen">hello-from-view</div>`
const out = compileSail(source)
console.log(out.js)
