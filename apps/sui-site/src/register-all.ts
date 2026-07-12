// Nothing. The docs are built from L1 — the elements are in the language, so
// there is no component library to register.
//
// This used to import seven packs for the old catalog, which coupled the docs
// build to every one of them: an L2 pack failing the floor took the whole site
// down. The tutorial needs a real editor (CodeMirror, L2) and will import it
// itself when it is rebuilt.
export {}
