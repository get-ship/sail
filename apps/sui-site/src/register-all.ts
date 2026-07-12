// The packs the SITE actually uses.
//
// This used to import every pack — charts, canvas, chat-elements, media,
// mermaid, primitives — because the old component catalog surfaced all of them.
// That coupled the docs build to seven packs the docs argument does not use, so
// one pack failing the floor took the whole site down.
//
// The tutorial is a real editing experience and needs a real editor. CodeMirror
// is L2, and that is fine: L2 is not "not allowed", it is "not in the compiler".
import '@ship/sui-code-editor'
