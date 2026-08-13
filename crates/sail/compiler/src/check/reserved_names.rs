//! RESERVED NAMES — the language owns the bare-name namespace.
//!
//! Custom elements are hyphenated (`my-card`); the language's own elements
//! are BARE (`card`, `dialog`, `button`). By check time, every language
//! construct has lowered away — stdlib elements are spliced to their HTML
//! structure, layout keywords (`<row>`/`<column>`) are `<div>`s — so any
//! remaining BARE tag must be a real HTML/SVG element. A bare tag that is
//! NEITHER valid HTML/SVG NOR a hyphenated component is an error: the
//! author either typo'd (`<buton>`) or reached for a language element that
//! does not exist yet (`<card>` before card is L1). Both should be told,
//! not silently rendered as an unknown element.
//!
//! This is what makes L1 growth SAFE: when we promote a component to a bare
//! language name, that name is already reserved — a user could never have
//! shadowed it (they can't register a bare custom element), and using it
//! before it lands gives a clear "not a known element" error.

use crate::diagnostics::CompileError;
use crate::ir::*;

const RULE_ID: &str = "sail-reserved-name";
pub fn check(program: &Program) -> Result<(), CompileError> {
    walk(&program.template, None)
}

fn walk(instrs: &[Instr], parent_tag: Option<&str>) -> Result<(), CompileError> {
    for instr in instrs {
        if let Instr::Element(el) = instr {
            check_tag(el, parent_tag)?;
            walk(&el.children, Some(el.tag.as_str()))?;
        }
        match instr {
            Instr::If(i) => {
                for b in &i.branches {
                    walk(&b.body, parent_tag)?;
                }
                if let Some(e) = &i.else_body {
                    walk(e, parent_tag)?;
                }
            }
            Instr::For(f) => {
                walk(&f.body, parent_tag)?;
                if let Some(e) = &f.else_body {
                    walk(e, parent_tag)?;
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    walk(&c.body, parent_tag)?;
                }
                if let Some(d) = &m.default {
                    walk(d, parent_tag)?;
                }
            }
            Instr::When(w) => {
                walk(&w.body, parent_tag)?;
                if let Some(e) = &w.else_body {
                    walk(e, parent_tag)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_tag(el: &ElementInstr, parent_tag: Option<&str>) -> Result<(), CompileError> {
    let tag = el.tag.as_str();
    // Hyphenated = a custom element (component). The language does not police
    // the hyphenated namespace — that belongs to the author.
    if tag.contains('-') {
        return Ok(());
    }
    // Post-RC region keyword — claim the name now so nothing else can.
    if tag.eq_ignore_ascii_case("test") {
        return Err(CompileError::new(
            "`<test>` is reserved — lands post-RC (behavior-space test regions). \
             Do not use it as a tag; app tests live outside .sail source via \
             @ship/sail-test semantic queries + mock seams.",
        )
        .with_span(el.span)
        .with_category(RULE_ID));
    }
    // `<on key=… action=…>` is the `<use-kbd>` template hook's declarative
    // shortcut record (see `dom_emit::collect_kbd_shortcuts`) — a bare tag
    // that is neither HTML nor hyphenated, but only valid as a direct child
    // of `<use-kbd>`.
    if tag.eq_ignore_ascii_case("on")
        && parent_tag.is_some_and(|p| p.eq_ignore_ascii_case("use-kbd"))
    {
        return Ok(());
    }
    // `<slot>` variants and the framework's own bare tags are HTML.
    if is_known_element(tag) {
        return Ok(());
    }
    Err(CompileError::new(format!(
        "unknown element `<{tag}>` — bare names are reserved for the language. \
         Use a hyphenated custom element (`<{tag}-x>`) for your own component, \
         or check the spelling; `<{tag}>` is not a known HTML element or a \
         language element."
    ))
    .with_span(el.span)
    .with_category(RULE_ID))
}

/// Case-insensitive membership. The parser normalizes tag case (`linearGradient`
/// arrives as `lineargradient`), so both tables are lowercase and we match the
/// lowered tag against both. (Correct-case SVG OUTPUT is the emitter's concern,
/// not this reserved-namespace gate.)
fn is_known_element(tag: &str) -> bool {
    let lower = tag.to_ascii_lowercase();
    HTML_ELEMENTS.contains(&lower.as_str()) || SVG_ELEMENTS.contains(&lower.as_str())
}

/// The HTML element names (lowercase). Comprehensive per the HTML Living
/// Standard, plus `slot`/`template` for web-component authoring.
const HTML_ELEMENTS: &[&str] = &[
    "a",
    "abbr",
    "address",
    "area",
    "article",
    "aside",
    "audio",
    "b",
    "base",
    "bdi",
    "bdo",
    "blockquote",
    "body",
    "br",
    "button",
    "canvas",
    "caption",
    "cite",
    "code",
    "col",
    "colgroup",
    "data",
    "datalist",
    "dd",
    "del",
    "details",
    "dfn",
    "dialog",
    "div",
    "dl",
    "dt",
    "em",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "i",
    "iframe",
    "img",
    "input",
    "ins",
    "kbd",
    "label",
    "legend",
    "li",
    "link",
    "main",
    "map",
    "mark",
    "menu",
    "meta",
    "meter",
    "nav",
    "noscript",
    "object",
    "ol",
    "optgroup",
    "option",
    "output",
    "p",
    "param",
    "picture",
    "pre",
    "progress",
    "q",
    "rp",
    "rt",
    "ruby",
    "s",
    "samp",
    "script",
    "search",
    "section",
    "select",
    "slot",
    "small",
    "source",
    "span",
    "strong",
    "style",
    "sub",
    "summary",
    "sup",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "time",
    "title",
    "tr",
    "track",
    "u",
    "ul",
    "var",
    "video",
    "wbr",
];

/// The SVG element names (exact case — SVG is case-sensitive). `a`/`title`/
/// `script`/`style`/`switch` overlap with HTML and are covered above.
const SVG_ELEMENTS: &[&str] = &[
    "svg",
    "path",
    "circle",
    "ellipse",
    "rect",
    "line",
    "polyline",
    "polygon",
    "g",
    "defs",
    "use",
    "symbol",
    "marker",
    "mask",
    "clippath",
    "pattern",
    "image",
    "text",
    "tspan",
    "textpath",
    "lineargradient",
    "radialgradient",
    "stop",
    "filter",
    "foreignobject",
    "desc",
    "metadata",
    "view",
    "mpath",
    "set",
    "animate",
    "animatemotion",
    "animatetransform",
    "fegaussianblur",
    "feoffset",
    "feblend",
    "fecolormatrix",
    "fecomponenttransfer",
    "fecomposite",
    "feconvolvematrix",
    "fediffuselighting",
    "fedisplacementmap",
    "fedistantlight",
    "fedropshadow",
    "feflood",
    "fefunca",
    "fefuncb",
    "fefuncg",
    "fefuncr",
    "feimage",
    "femerge",
    "femergenode",
    "femorphology",
    "fepointlight",
    "fespecularlighting",
    "fespotlight",
    "fetile",
    "feturbulence",
];
