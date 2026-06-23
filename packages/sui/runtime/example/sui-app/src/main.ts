// Side-effect import: the compiled .sui module calls component() at
// import time, which registers <sui-app> as a custom element.
// Filename is `<stem>.js` (not `.compiled.js`) because precompile.mjs
// passes --out — see the script for why (the compiler's default
// mirror-tree output lands under <repo-root>/.ship/sui-out/, awkward
// for a standalone example).
import './dist/sui-app.js'
