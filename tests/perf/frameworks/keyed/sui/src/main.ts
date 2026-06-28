// Side-effect import: the compiled .sui calls component() at import time,
// registering <sui-keyed> as a custom element. <stem>.js (not .compiled.js)
// because precompile.mjs passes --out (see that script).
import './dist/sui-keyed.js'
