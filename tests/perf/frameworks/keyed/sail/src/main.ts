// Side-effect import: the compiled .sail calls component() at import time,
// registering <sail-keyed> as a custom element. <stem>.js (not .compiled.js)
// because precompile.ts passes --out (see that script).
import './dist/sail-keyed.js'
