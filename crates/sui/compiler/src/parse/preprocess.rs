//! Source preprocessor — the single, Rust-only path. (There is no JS twin:
//! the legacy `@ship/sui-dom` preprocess was removed when every directive was
//! made to round-trip through this crate; deck's bundler and the Node test
//! harness both compile in-process through `sui_compiler`.)
//!
//! Transforms, in order, before the HTML parser sees the source:
//!  1. Quote unquoted brace attributes — `disabled={x || y}` becomes
//!     `disabled="{x || y}"` with `"` inside the expression escaped to `&quot;`.
//!  2. Expand non-void self-closing tags — `<data … />` becomes `<data …></data>`.
//!  3. Lower `sx="…"` surface attributes → atomic `class`es.
//!  4. Lower the layout keywords (`<column>`/`<row>`/…) → `<div>` + atomic classes.
//! Steps 3–4 emit deduped CSS that `preprocess` injects as one `<style>`.

use crate::diagnostics::Span;
use crate::ir::directive::{PropDecl, PropKind};

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "source", "track", "wbr",
];

pub fn preprocess(source: &str) -> String {
    preprocess_impl(source, true)
}

/// The full pipeline with the stdlib-element pass switchable — element
/// sources compile WITHOUT it (no element-in-element; no recursion when the
/// registry compiles the stdlib with this same crate).
pub(crate) fn preprocess_impl(source: &str, expand_elements: bool) -> String {
    // Brace-quoting + self-close expansion normalize the raw source first. Then
    // the two compile-time lowerings run on that HTML: `sx=` attributes → atomic
    // surface classes, then the layout keywords (`<column>`/`<row>`/…) → `<div>`
    // + atomic layout classes. Both return the deduped CSS they need; we inject
    // it as ONE `<style>` at the very end so the brace-attr quoter never sees its
    // `{…}`-bearing rules. sx runs before layout so a layout element's own `sx=`
    // is already a `class` that the layout rewrite folds into the div's classes.
    // Off-vocab values are rejected earlier by the lints (`sui-sx` / `sui-layout`)
    // reading the raw source; here we only lower valid ones.
    let s = expand_self_close(&quote_brace_attrs(source));
    // Bare framework-enum shorthand: `<button primary solid sm>` →
    // tone/variant/size attributes. Framework vocabulary on framework tags
    // only (L1 controls + sui-*); userland components stay explicit.
    let s = lower_prop_shorthand(&s);
    // L1 stdlib elements: `<spinner …>` splices into its structure snippet
    // (use-site attrs merged onto the root, default-slot children placed),
    // its skin CSS joining the injected style once per tag.
    let (s, elements_css) = if expand_elements {
        lower_stdlib_elements(&s)
    } else {
        (s, String::new())
    };
    // The `<component>` block is parsed directly in `parse` — props → PropDecl,
    // `variant` props → CSS. No desugar to `<prop>`/`<variants>` (both retired).
    // Framework-owned host display: the component presents as its root element
    // naturally would (inheriting the standard, not a per-component knob). Block
    // is the default, so this only emits an override for inline-natured roots.
    // Computed now, before lower_layout rewrites <row>/<column> into <div>.
    let host_display_css = derive_host_display(&s);
    // The `<host>` tag is retired — root styling is `sx` on the component's root
    // element (which lowers to a class), not a magic host declaration. A stale
    // `<host>` now survives to `parse` and is rejected as an unknown element.
    let (s, sx_css) = lower_sx(&s);
    // An element that already carried `class="…"` now has a SECOND class attr
    // from the sx lowering; downstream the DOM emitter would issue two
    // `setAttribute("class", …)` calls and the sx one clobbers the author's.
    // Merge them into one attribute here, while the tag text is still flat.
    let s = merge_duplicate_class_attrs(&s);
    let (s, layout_css) = lower_layout(&s);
    // The `<theme>` tag is retired — tone/size are the framework channel
    // contract (union props reflected to attributes, mapped to tokens by the
    // skin), not a per-component channel declaration. A stale `<theme>` now
    // survives to `parse` and is rejected as an unknown element.
    // NOTE elements_css rides the per-component injected style for now —
    // duplicated across components that use the same element. It moves to
    // the framework sheet with the deck style-delivery work.
    let css = format!(
        "{host_display_css}{sx_css}{layout_css}{elements_css}"
    );
    if css.is_empty() {
        s
    } else {
        format!("<style>{css}</style>\n{s}")
    }
}

/// Lower every static `sx="…"` attribute to `class="sx-…"`, returning the
/// rewritten body and the deduped atomic CSS (the caller injects it into the
/// shared `<style>`). The prop→CSS mapping is `sui_lint::rules::sx::lower` — the
/// same vocabulary the lint validates.
fn lower_sx(source: &str) -> (String, String) {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len() + 256);
    let mut css = String::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    let mut last = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if i > 0
            && bytes[i - 1].is_ascii_whitespace()
            && source[i..].starts_with("sx")
            && is_eq_quote(&bytes[i + 2..])
        {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j] != b'"' && bytes[j] != b'{' {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'"' {
                let vstart = j + 1;
                if let Some(rel) = source[vstart..].find('"') {
                    let val = &source[vstart..vstart + rel];
                    out.push_str(&source[last..i]);
                    out.push_str("class=\"");
                    out.push_str(&lower_sx_value(val, &mut css, &mut seen));
                    out.push('"');
                    i = vstart + rel + 1;
                    last = i;
                    continue;
                }
            }
        }
        // Advance one FULL UTF-8 char. A bare `i += 1` lands mid-codepoint on
        // any multi-byte char (…, emoji, accents) and the next `source[i..]`
        // slice panics — the crash the async golden fixtures hit. Mirrors the
        // char-safe stepping in `quote_brace_attrs` / `expand_self_close`.
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        i = n;
    }
    out.push_str(&source[last..]);
    (out, css)
}

fn lower_sx_value(
    val: &str,
    css: &mut String,
    seen: &mut std::collections::HashSet<String>,
) -> String {
    let mut classes: Vec<String> = Vec::new();
    for pair in val.split_whitespace() {
        // Bare `group` marks an ancestor so descendants can react to its state
        // with `group-<state>:` (the `.field:hover .chevron` propagation case).
        if pair == "group" {
            classes.push("sui-grp".to_string());
            continue;
        }
        // `prop:token`, or `state:prop:token` — a state prefix lowers to an atomic
        // class carrying that pseudo-class (`hover:bg:surface-2` →
        // `.sx-hover-bg-surface2:hover{background:var(--surface-2)}`).
        let (state, prop, token) = match pair.splitn(3, ':').collect::<Vec<_>>().as_slice() {
            [prop, token] => (None, *prop, *token),
            [state, prop, token] => (Some(*state), *prop, *token),
            _ => continue,
        };
        // A `<state>` is a pseudo (`hover`) or a propagation (`group-hover` → the
        // rule keys on an ancestor `.sui-grp:hover`). `prefix` is a leading
        // ancestor selector; `pseudo` a trailing pseudo-class on the element.
        let (media, prefix, pseudo): (&str, String, String) = match state {
            None => ("", String::new(), String::new()),
            Some(s) => {
                if let Some(m) = sui_lint::rules::sx::condition_media(s) {
                    (m, String::new(), String::new())
                } else if let Some(base) = s.strip_prefix("group-") {
                    match sui_lint::rules::sx::state_pseudo(base)
                        .map(str::to_string)
                        .or_else(|| sui_lint::rules::sx::state_data(base))
                    {
                        Some(p) => ("", format!(".sui-grp{p} "), String::new()),
                        None => continue,
                    }
                } else {
                    match sui_lint::rules::sx::state_pseudo(s)
                        .map(str::to_string)
                        .or_else(|| sui_lint::rules::sx::state_data(s))
                    {
                        Some(p) => ("", String::new(), p),
                        None => continue,
                    }
                }
            }
        };
        let sanitized: String = token.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        let class = match state {
            None => format!("sx-{prop}-{sanitized}"),
            Some(s) => format!("sx-{s}-{prop}-{sanitized}"),
        };
        // The token vocabulary + framework channels (tone-*/size-*) live in the
        // registry `lower` consults. There is deliberately NO per-file channel
        // path: a `<theme>`-declared channel is not sx vocabulary (the closed
        // back door — ad-hoc channels were how translation snuck past the
        // design system).
        if let Some(decls) = sui_lint::rules::sx::lower(prop, token) {
            if seen.insert(class.clone()) {
                let body = decls
                    .iter()
                    .map(|(p, v)| format!("{p}:{v}"))
                    .collect::<Vec<_>>()
                    .join(";");
                let rule = format!("{prefix}.{class}{pseudo}{{{body}}}");
                if media.is_empty() {
                    css.push_str(&rule);
                } else {
                    css.push_str(&format!("@media {media}{{{rule}}}"));
                }
            }
        }
        classes.push(class);
    }
    classes.join(" ")
}

/// After the `sx` name: optional ws, `=`, optional ws, then a `"`.
/// L1 STDLIB ELEMENT LOWERING — splice each `<tag …>children</tag>` use of a
/// registry element into its compiled structure snippet:
///   - use-site attributes merge onto the snippet root (use-site wins on a
///     name collision; `class` values concatenate),
///   - children replace the snippet's default `<slot></slot>` (named-slot
///     compile-time projection is slice 3; today's stdlib elements use at
///     most the default slot),
///   - the element's skin CSS joins the injected style ONCE per tag.
fn lower_stdlib_elements(source: &str) -> (String, String) {
    let registry = crate::stdlib::elements();
    if registry.is_empty() {
        return (source.to_string(), String::new());
    }
    let mut out = String::with_capacity(source.len());
    let mut css = String::new();
    let mut css_done: std::collections::HashSet<&str> = std::collections::HashSet::new();
    // STDLIB ELEMENT SKINS ARE NOT INLINED. They ship in the framework stylesheet
    // (`element-skins.css`, generated by `sui-compiler framework-css` — the compiler's
    // half of `sui.css`), which the app links once and the browser caches.
    //
    // They used to be spliced into every component's emitted `<style>`, which meant
    // 28.7 KB of CSS embedded in the JAVASCRIPT of a single page using twelve L1
    // components — and duplicated again in the next page's chunk. The comment that
    // used to sit here said so: "interim delivery … moves to the framework sheet with
    // deck style-delivery." It never moved, so it kept charging every component.
    //
    // A stylesheet belongs in a stylesheet. Same call as the contracts sheet, and the
    // same one link (`sui.css`) delivers both. LAYOUT css is different and stays
    // inline: the preprocessor generates only the rules a file actually uses, so it is
    // not in the framework sheet and has nothing to be deduplicated against.
    lower_into(source, registry, &mut out, &mut css, &mut css_done);
    (out, css)
}

/// The recursive core of stdlib lowering. Factored out so a nested element
/// (a `<card>` inside a `<card>`) gets ITS children lowered too: the old
/// single pass used `find("</tag>")` — the FIRST close — which paired the
/// inner card's close with the outer card, corrupting the tree. Now the
/// matching close is found depth-aware (elements CAN self-nest), and the
/// children are lowered recursively before they splice into the slot. `css`
/// / `css_done` are threaded so each tag's skin still joins exactly once,
/// no matter how deeply it nests.
fn lower_into(
    source: &str,
    registry: &'static [crate::stdlib::StdlibElement],
    out: &mut String,
    css: &mut String,
    css_done: &mut std::collections::HashSet<&'static str>,
) {
    let bytes = source.as_bytes();
    let mut i = 0;
    'outer: while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            let end = source[i..].find("-->").map(|e| i + e + 3).unwrap_or(bytes.len());
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        // <style> blocks pass through UNTOUCHED — CSS text (selectors,
        // /* comments */ mentioning element names) must never be spliced
        // (the sonnet-agent find: a CSS comment naming <label> swallowed
        // the rest of the file).
        if source[i..].starts_with("<style") {
            let end = source[i..]
                .find("</style>")
                .map(|e| i + e + "</style>".len())
                .unwrap_or(bytes.len());
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            for el in registry {
                let tag = el.tag;
                // NATIVE-KIND elements (root tag == element tag, e.g. label
                // → <label>) are skin + manifest only — the tag already IS
                // the element; splicing would rewrite every real HTML use.
                if el.out.native {
                    continue;
                }
                if source[i + 1..].starts_with(tag) {
                    let after = i + 1 + tag.len();
                    let boundary = bytes
                        .get(after)
                        .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>');
                    if !boundary {
                        continue;
                    }
                    // Open-tag end, quote-aware.
                    let mut k = after;
                    let mut quote: Option<u8> = None;
                    while k < bytes.len() {
                        match (quote, bytes[k]) {
                            (Some(q), b) if b == q => quote = None,
                            (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                            (None, b'>') => break,
                            _ => {}
                        }
                        k += 1;
                    }
                    if k >= bytes.len() {
                        break;
                    }
                    let use_attrs = source[after..k].trim().to_string();
                    // Matching close, DEPTH-AWARE: skip the closes of nested
                    // same-tag opens (expand_self_close already paired the
                    // self-closing forms, so only `<tag …>` opens count).
                    let open_prefix = format!("<{tag}");
                    let close = format!("</{tag}>");
                    let mut depth = 1usize;
                    let mut scan = k + 1;
                    let close_start = loop {
                        if scan >= bytes.len() {
                            break None;
                        }
                        // Byte-level compares — `scan` advances by one byte in
                        // the fallthrough, so it can land mid-multibyte-char;
                        // slicing `source[scan..]` there would panic (the ⌘/—/°
                        // in real pages). `bytes` slicing is boundary-free.
                        if bytes[scan..].starts_with(close.as_bytes()) {
                            depth -= 1;
                            if depth == 0 {
                                break Some(scan);
                            }
                            scan += close.len();
                        } else if bytes[scan..].starts_with(open_prefix.as_bytes())
                            && bytes
                                .get(scan + open_prefix.len())
                                .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>')
                        {
                            depth += 1;
                            scan += open_prefix.len();
                        } else {
                            scan += 1;
                        }
                    };
                    let Some(close_start) = close_start else { break };
                    let children = &source[k + 1..close_start];
                    let end = close_start + close.len();

                    // The skin is NOT joined — it lives in the framework stylesheet
                    // (see `lower_stdlib_elements`). `css_done` is still marked so the
                    // bookkeeping stays honest for anything else keyed on it.
                    css_done.insert(tag);
                    // Lower nested elements in the children, then splice.
                    let mut lowered = String::new();
                    lower_into(children, registry, &mut lowered, css, css_done);
                    out.push_str(&splice_element(&el.out, &use_attrs, &lowered));
                    i = end;
                    continue 'outer;
                }
            }
        }
        let next = bytes[i + 1..]
            .iter()
            .position(|&b| b == b'<')
            .map(|n| i + 1 + n)
            .unwrap_or(bytes.len());
        out.push_str(&source[i..next]);
        i = next;
    }
}

/// Merge use-site attrs onto the snippet root and place children.
fn splice_element(
    el: &crate::emit::element::ElementOutput,
    use_attrs: &str,
    children: &str,
) -> String {
    let snippet = &el.structure_html;
    let Some(root_end) = snippet.find('>') else { return snippet.clone() };
    let root_open = &snippet[..root_end]; // "<span class=\"sui-x …\" …"
    let rest = &snippet[root_end..]; // ">…</span>"

    let mut snippet_attrs = parse_attr_list(&root_open[root_open.find(char::is_whitespace).map(|p| p + 1).unwrap_or(root_open.len())..]);
    let use_list = parse_attr_list(use_attrs);
    for (name, value) in use_list {
        if name == "class" {
            if let Some(slot) = snippet_attrs.iter_mut().find(|(n, _)| n == "class") {
                let merged = format!(
                    "{} {}",
                    slot.1.trim_matches('"'),
                    value.trim_matches('"')
                );
                slot.1 = format!("\"{merged}\"");
                continue;
            }
        }
        // Use-site wins on collision.
        snippet_attrs.retain(|(n, _)| *n != name);
        snippet_attrs.push((name, value));
    }
    let tag_name: String = root_open[1..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    let mut outb = format!("<{tag_name}");
    for (n, v) in &snippet_attrs {
        outb.push(' ');
        outb.push_str(n);
        if !v.is_empty() {
            outb.push('=');
            outb.push_str(v);
        }
    }
    // COMPILE-TIME SLOT PROJECTION. Partition the use-site children by their
    // `slot="name"` attribute (un-slotted nodes → the default slot), then fill
    // each `<slot [name=…]>fallback</slot>` region of the structure. A slot
    // with no matching projected content keeps its fallback; children with no
    // matching slot are dropped (shadow parity, same as the runtime).
    let (default_children, named_children) = project_children(children);
    let body = fill_slots(rest, &default_children, &named_children);
    outb.push_str(&body);
    outb
}

/// Partition use-site children into the default bucket (un-slotted nodes, in
/// source order) and named buckets keyed by each top-level node's `slot=`
/// attribute. A node's slot assigns its WHOLE subtree — a `slot=` deeper in
/// the tree belongs to that descendant, not to us. Quote/void/comment aware,
/// depth-aware on the matching close (mirrors `lower_into`).
fn project_children(children: &str) -> (String, std::collections::HashMap<String, String>) {
    let mut default = String::new();
    let mut named: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let bytes = children.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if children[i..].starts_with("<!--") {
            let end = children[i..].find("-->").map(|e| i + e + 3).unwrap_or(bytes.len());
            default.push_str(&children[i..end]);
            i = end;
            continue;
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let tag_start = i + 1;
            let mut te = tag_start;
            while te < bytes.len() && (bytes[te].is_ascii_alphanumeric() || bytes[te] == b'-') {
                te += 1;
            }
            let tag = &children[tag_start..te];
            // Open-tag end, quote-aware.
            let mut k = te;
            let mut quote: Option<u8> = None;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k >= bytes.len() {
                default.push_str(&children[i..]);
                break;
            }
            let open_tag = &children[i..=k];
            let self_closed = bytes[k.saturating_sub(1)] == b'/';
            let node_end = if self_closed || crate::emit::element::is_void_element(tag) {
                k + 1
            } else {
                let open_prefix = format!("<{tag}");
                let close = format!("</{tag}>");
                let mut depth = 1usize;
                let mut scan = k + 1;
                loop {
                    if scan >= bytes.len() {
                        break bytes.len();
                    }
                    if bytes[scan..].starts_with(close.as_bytes()) {
                        depth -= 1;
                        if depth == 0 {
                            break scan + close.len();
                        }
                        scan += close.len();
                    } else if bytes[scan..].starts_with(open_prefix.as_bytes())
                        && bytes
                            .get(scan + open_prefix.len())
                            .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>')
                    {
                        depth += 1;
                        scan += open_prefix.len();
                    } else {
                        scan += 1;
                    }
                }
            };
            let node = &children[i..node_end];
            match open_tag_attr(open_tag, "slot") {
                Some(name) => named.entry(name).or_default().push_str(node),
                None => default.push_str(node),
            }
            i = node_end;
            continue;
        }
        let next = bytes[i + 1..]
            .iter()
            .position(|&b| b == b'<')
            .map(|n| i + 1 + n)
            .unwrap_or(bytes.len());
        default.push_str(&children[i..next]);
        i = next;
    }
    (default, named)
}

/// Fill each `<slot [name=…]>fallback</slot>` in a structure snippet with the
/// projected content for that slot name (default slot = no `name`). An empty
/// or absent bucket falls back to the slot's own inner content.
fn fill_slots(
    rest: &str,
    default: &str,
    named: &std::collections::HashMap<String, String>,
) -> String {
    let bytes = rest.as_bytes();
    let mut out = String::with_capacity(rest.len());
    let mut i = 0;
    while i < bytes.len() {
        if rest[i..].starts_with("<slot")
            && bytes
                .get(i + 5)
                .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>')
        {
            let mut k = i + 5;
            let mut quote: Option<u8> = None;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k >= bytes.len() {
                out.push_str(&rest[i..]);
                break;
            }
            let open_tag = &rest[i..=k];
            let close_at = rest[k..].find("</slot>").map(|p| k + p);
            let (fallback, end) = match close_at {
                Some(c) => (&rest[k + 1..c], c + "</slot>".len()),
                None => ("", k + 1),
            };
            let projected = match open_tag_attr(open_tag, "name") {
                Some(n) => named.get(&n).map(String::as_str).filter(|s| !s.is_empty()),
                None => Some(default).filter(|s| !s.is_empty()),
            };
            out.push_str(projected.unwrap_or(fallback));
            i = end;
            continue;
        }
        let next = bytes[i + 1..]
            .iter()
            .position(|&b| b == b'<')
            .map(|n| i + 1 + n)
            .unwrap_or(bytes.len());
        out.push_str(&rest[i..next]);
        i = next;
    }
    out
}

/// Read a single attribute's value off an element's open tag (`<tag a="1" …>`),
/// quotes stripped. Returns None if the attribute is absent.
fn open_tag_attr(open_tag: &str, attr: &str) -> Option<String> {
    let inner = open_tag
        .trim_start_matches('<')
        .trim_end_matches('>')
        .trim_end_matches('/');
    let after_tag = inner
        .find(char::is_whitespace)
        .map(|p| &inner[p + 1..])
        .unwrap_or("");
    parse_attr_list(after_tag)
        .into_iter()
        .find(|(n, _)| n == attr)
        .map(|(_, v)| v.trim_matches(['"', '\'']).to_string())
}

/// Parse a flat attribute list `a="1" b c={x}` into (name, raw-value) pairs
/// (value includes its quotes; empty for bare attrs). Quote-aware.
fn parse_attr_list(s: &str) -> Vec<(String, String)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut outv = Vec::new();
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'/' {
            break;
        }
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'>'
        {
            i += 1;
        }
        let name = s[name_start..i].to_string();
        if name.is_empty() {
            break;
        }
        let mut value = String::new();
        if bytes.get(i) == Some(&b'=') {
            i += 1;
            if let Some(&q) = bytes.get(i).filter(|&&b| b == b'"' || b == b'\'') {
                let vstart = i;
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                i = (i + 1).min(bytes.len());
                value = s[vstart..i].to_string();
            } else {
                let vstart = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                    i += 1;
                }
                value = s[vstart..i].to_string();
            }
        }
        outv.push((name, value));
    }
    outv
}

/// Bare framework-enum shorthand (authoring sugar): on framework tags, a bare
/// attribute matching a framework enum value desugars to its prop —
/// `<button primary solid sm>` → `<button tone="primary" variant="solid"
/// size="sm">`. Rules that keep it deterministic:
///   - Framework tags only: the L1 control set + any `sui-*` tag. Userland
///     components keep explicit attributes (their prop names could collide).
///   - The vocabularies are disjoint by construction (tones ∪ sizes ∪
///     variants share no member); `default` is excluded (ambiguous between
///     tone and variant, and it IS the default — writing it is a no-op).
///   - An explicit attribute wins: a tag already carrying `tone=` gets no
///     tone sugar. First bare value wins per category; later ones pass
///     through untouched (harmless attributes a lint can flag).
///   - Static only, like all sugar — the DOM carries the real attributes, so
///     skins, contracts, and CEM see nothing new.
fn lower_prop_shorthand(source: &str) -> String {
    const TONES: &[&str] =
        &["primary", "info", "success", "warning", "destructive", "accent", "muted"];
    const SIZES: &[&str] = &["xs", "sm", "md", "lg", "xl"];
    const VARIANTS: &[&str] = &["solid", "soft", "outline", "ghost"];
    const L1_TAGS: &[&str] = &["button", "input", "textarea", "select"];

    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            let end = source[i..].find("-->").map(|e| i + e + 3).unwrap_or(bytes.len());
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            // Tag name + framework check.
            let mut j = i + 1;
            while j < bytes.len()
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-')
            {
                j += 1;
            }
            let tag_name = &source[i + 1..j];
            let is_framework = L1_TAGS.contains(&tag_name) || tag_name.starts_with("sui-");
            // Find tag end honoring quotes.
            let mut k = j;
            let mut quote: Option<u8> = None;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k < bytes.len() && is_framework {
                let tag = &source[i..=k];
                out.push_str(&desugar_tag(tag, TONES, SIZES, VARIANTS));
                i = k + 1;
                continue;
            }
            if k < bytes.len() {
                out.push_str(&source[i..=k]);
                i = k + 1;
                continue;
            }
        }
        let next = bytes[i + 1..]
            .iter()
            .position(|&b| b == b'<')
            .map(|n| i + 1 + n)
            .unwrap_or(bytes.len());
        out.push_str(&source[i..next]);
        i = next;
    }
    out
}

/// Desugar one framework open-tag: replace bare vocab attributes with their
/// prop form, honoring explicit-attribute precedence and first-wins.
fn desugar_tag(tag: &str, tones: &[&str], sizes: &[&str], variants: &[&str]) -> String {
    let has = |needle: &str| tag.contains(needle);
    let mut tone_done = has(" tone=") || has("\ntone=");
    let mut size_done = has(" size=") || has("\nsize=");
    let mut variant_done = has(" variant=") || has("\nvariant=");

    let bytes = tag.as_bytes();
    let mut out = String::with_capacity(tag.len() + 24);
    let mut i = 0;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        match (quote, bytes[i]) {
            (Some(q), b) if b == q => {
                quote = None;
                out.push(bytes[i] as char);
                i += 1;
                continue;
            }
            (Some(_), _) => {
                // Inside a quoted value — copy one full UTF-8 char.
                let mut n = i + 1;
                while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
                    n += 1;
                }
                out.push_str(&tag[i..n]);
                i = n;
                continue;
            }
            (None, b'"') | (None, b'\'') => {
                quote = Some(bytes[i]);
                out.push(bytes[i] as char);
                i += 1;
                continue;
            }
            _ => {}
        }
        // A bare word starts after whitespace.
        if i > 0 && bytes[i - 1].is_ascii_whitespace() && bytes[i].is_ascii_alphabetic() {
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-') {
                j += 1;
            }
            let word = &tag[i..j];
            let ends_bare = bytes
                .get(j)
                .is_none_or(|&b| b.is_ascii_whitespace() || b == b'>' || b == b'/');
            if ends_bare {
                let prop = if !tone_done && tones.contains(&word) {
                    tone_done = true;
                    Some("tone")
                } else if !size_done && sizes.contains(&word) {
                    size_done = true;
                    Some("size")
                } else if !variant_done && variants.contains(&word) {
                    variant_done = true;
                    Some("variant")
                } else {
                    None
                };
                if let Some(p) = prop {
                    out.push_str(&format!("{p}=\"{word}\""));
                    i = j;
                    continue;
                }
            }
            out.push_str(word);
            i = j;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Merge every open tag's duplicate static `class="…"` attributes into the
/// first one (values space-joined, source order). `lower_sx` rewrites `sx=` to
/// a fresh `class=` without looking for an existing one, so
/// `<span class="badge" sx="p:4">` arrives here as two class attrs — the HTML
/// parser keeps both and the emitted `setAttribute("class")` pair lets the
/// second wipe the first at runtime (the sui-badge `.badge` regression).
/// Conservative: a tag containing a dynamic `class="{…}"` is left untouched.
fn merge_duplicate_class_attrs(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            let end = source[i..].find("-->").map(|e| i + e + 3).unwrap_or(bytes.len());
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        // Only open tags: `<` + ASCII letter (not `</`, `<!`, `<{`).
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            // Find the tag end, honoring quoted attribute values (a `>` inside
            // a quoted value must not terminate the tag).
            let mut j = i + 1;
            let mut quote: Option<u8> = None;
            while j < bytes.len() {
                match (quote, bytes[j]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[j]),
                    (None, b'>') => break,
                    _ => {}
                }
                j += 1;
            }
            if j < bytes.len() {
                let tag = &source[i..=j];
                out.push_str(&merge_tag_classes(tag));
                i = j + 1;
                continue;
            }
        }
        // Copy up to the next `<` (or the rest). Byte-scan, NOT a str slice
        // at `i + 1` — when `i` sits on a multi-byte char, `source[i + 1..]`
        // is mid-codepoint and panics (the sui-date-picker `‹` regression).
        // A `b'<'` byte position is always a char boundary.
        let next = bytes[i + 1..]
            .iter()
            .position(|&b| b == b'<')
            .map(|n| i + 1 + n)
            .unwrap_or(bytes.len());
        out.push_str(&source[i..next]);
        i = next;
    }
    out
}

/// Merge the static `class="…"` attributes inside one open-tag slice. Returns
/// the tag unchanged when it has fewer than two, or when any class value is a
/// `{…}` expression (dynamic — the emitter owns that path).
fn merge_tag_classes(tag: &str) -> String {
    let bytes = tag.as_bytes();
    // (attr_start, value_start, value_end) for each `class="…"`, where
    // attr_start points at the `c` and the value span excludes the quotes.
    let mut hits: Vec<(usize, usize, usize)> = Vec::new();
    let mut i = 0;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        match (quote, bytes[i]) {
            (Some(q), b) if b == q => {
                quote = None;
                i += 1;
                continue;
            }
            (Some(_), _) => {
                i += 1;
                continue;
            }
            (None, b'"') | (None, b'\'') => {
                quote = Some(bytes[i]);
                i += 1;
                continue;
            }
            _ => {}
        }
        // `bytes[i] == b'c'` first: it guarantees `i` is a char boundary
        // (an ASCII byte is never a continuation byte), so the `tag[i..]`
        // slice below cannot land mid-codepoint.
        if i > 0
            && bytes[i] == b'c'
            && bytes[i - 1].is_ascii_whitespace()
            && tag[i..].starts_with("class=\"")
        {
            let vstart = i + "class=\"".len();
            if let Some(rel) = tag[vstart..].find('"') {
                hits.push((i, vstart, vstart + rel));
                i = vstart + rel + 1;
                continue;
            }
        }
        i += 1;
    }
    if hits.len() < 2 || hits.iter().any(|&(_, vs, ve)| tag[vs..ve].trim_start().starts_with('{')) {
        return tag.to_string();
    }
    let merged = hits
        .iter()
        .map(|&(_, vs, ve)| tag[vs..ve].trim())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut out = String::with_capacity(tag.len());
    let (_, first_vs, _) = hits[0];
    out.push_str(&tag[..first_vs]);
    out.push_str(&merged);
    // Copy the remainder, skipping every later `class="…"` attribute (and the
    // single whitespace run that precedes it).
    let mut pos = hits[0].2; // at the closing quote of the first value
    for &(start, _, vend) in &hits[1..] {
        let mut cut = start;
        while cut > pos && bytes[cut - 1].is_ascii_whitespace() {
            cut -= 1;
        }
        out.push_str(&tag[pos..cut]);
        pos = vend + 1; // past the closing quote
    }
    out.push_str(&tag[pos..]);
    out
}

fn is_eq_quote(rest: &[u8]) -> bool {
    let mut k = 0;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    if k >= rest.len() || rest[k] != b'=' {
        return false;
    }
    k += 1;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    k < rest.len() && rest[k] == b'"'
}

/// Lower each layout keyword (`<column>`, `<row>`, `<grid>`, `<center>`,
/// `<stack>`) to a `<div>` + atomic layout classes, returning the rewritten body
/// and the deduped CSS. Axis props (gap/align/justify/min/flow) become classes
/// and drop off the tag; every other attribute passes through, and an existing
/// `class` (static or dynamic) is merged into the computed classes. Close tags
/// become `</div>`. The class/CSS mapping is `sui_lint::rules::layout` — the same
/// vocabulary the `sui-layout` lint validates against the raw source.
fn lower_layout(source: &str) -> (String, String) {
    use sui_lint::rules::layout;
    let opens = layout::scan_open_tags(source);
    if opens.is_empty() {
        return (source.to_string(), String::new());
    }
    let open_at: std::collections::HashMap<usize, &layout::OpenTag> =
        opens.iter().map(|t| (t.start, t)).collect();

    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len() + 256);
    let mut css = String::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    out.push_str(&source[i..i + end + 3]);
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if let Some(t) = open_at.get(&i)
            && let Some((base_class, base_rules)) = layout::base(&t.tag)
        {
            for r in base_rules {
                if seen.insert(r.clone()) {
                    css.push_str(&r);
                }
            }
            // Static class parts join into `class="…"`; dynamic parts (a dynamic
            // axis or a user `class={expr}`) each become a JS expression, and the
            // whole thing lowers to a `class={…}` binding.
            let mut static_parts: Vec<String> = vec![base_class];
            let mut dyn_parts: Vec<String> = Vec::new();
            // `cols` vs `min` precedence on `<grid>`: a fixed column count and a
            // responsive `auto-fill minmax` axis are mutually exclusive — both
            // own `grid-template-columns`. `cols` wins, so when present we
            // skip the `min` lowering entirely (the underlying div only gets
            // the `cols` rule, no `min` class, no minmax track list).
            let has_cols = t.attrs.iter().any(|(n, _)| n == "cols");
            let mut skip_props: std::collections::HashSet<&str> =
                std::collections::HashSet::new();
            if t.tag == "grid" && has_cols {
                skip_props.insert("min");
            }
            for &prop in layout::tag_props(&t.tag) {
                if skip_props.contains(prop) {
                    continue;
                }
                let attr = t.attrs.iter().find(|(n, _)| n == prop).map(|(_, v)| v.as_str());
                match layout::lower_axis(&t.tag, prop, attr) {
                    Some(layout::AxisLower::Static { class, rule }) => {
                        if seen.insert(rule.clone()) {
                            css.push_str(&rule);
                        }
                        static_parts.push(class);
                    }
                    Some(layout::AxisLower::Dynamic { class_expr, rules }) => {
                        for r in rules {
                            if seen.insert(r.clone()) {
                                css.push_str(&r);
                            }
                        }
                        dyn_parts.push(class_expr);
                    }
                    None => {}
                }
            }
            let mut rest = String::new();
            for (name, value) in &t.attrs {
                match name.as_str() {
                    // Layout axis props — already consumed by the lowering loop
                    // above; their classes/rules are folded into the div's
                    // `class` and the injected stylesheet, NEVER emitted as
                    // raw HTML attrs.
                    "gap" | "align" | "justify" | "min" | "flow" | "cols" | "direction" | "tracks" => {}
                    "class" if value.starts_with('{') => {
                        dyn_parts.push(format!("({})", &value[1..value.len() - 1]));
                    }
                    "class" => {
                        if !value.is_empty() {
                            static_parts.push(value.clone());
                        }
                    }
                    name if layout::is_passthrough_attr(name) => {
                        // Recognized HTML/global pass-through (id, data-*,
                        // aria-*, on:*, type/value/href/src/…). Emit verbatim
                        // onto the div. The lint and the compiler share this
                        // pass-through list via `layout::is_passthrough_attr`,
                        // so a typo or unsupported layout attr never silently
                        // rides through to the div as a dead attribute.
                        rest.push(' ');
                        rest.push_str(name);
                        if !value.is_empty() {
                            rest.push_str("=\"");
                            rest.push_str(value);
                            rest.push('"');
                        }
                    }
                    _ => {
                        // An attribute the lint would have already rejected
                        // as an unknown layout-attr. We never reach here on
                        // well-formed sources (the lint is the build gate);
                        // emit nothing defensively so a bad attr can't
                        // silently flow through to the div.
                    }
                }
            }
            let static_str = static_parts.join(" ");
            out.push_str("<div class=");
            if dyn_parts.is_empty() {
                out.push_str(&format!("\"{static_str}\""));
            } else {
                // `class={'<static> ' + <dyn1> + ' ' + <dyn2> …}` — a quoted brace
                // attr, what quote_brace_attrs would have emitted for a binding.
                let dyn_expr = dyn_parts.join(" + ' ' + ");
                out.push_str(&format!("\"{{'{static_str} ' + {dyn_expr}}}\""));
            }
            out.push_str(&rest);
            out.push('>');
            if t.self_closing {
                out.push_str("</div>");
            }
            i = t.end;
            continue;
        }
        // Layout close tag → `</div>` (every layout tag lowers to a div).
        if source[i..].starts_with("</")
            && let Some(tag) = layout::TAGS.iter().find(|&&tg| {
                source[i + 2..].starts_with(tg) && bytes.get(i + 2 + tg.len()) == Some(&b'>')
            })
        {
            out.push_str("</div>");
            i += 2 + tag.len() + 1;
            continue;
        }
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        out.push_str(&source[i..n]);
        i = n;
    }
    out.push_str(&source[i..]);
    (out, css)
}

/// Split `s` on top-level `delim`, ignoring delimiters nested inside `{}`/`[]`
/// or string literals. The primitive under the `<variants>` object parser.
fn split_top_level(s: &str, delim: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    // COMMENTS ARE OPAQUE to the split. A delimiter inside one is not a delimiter:
    //
    //     /** The tone, e.g. primary */
    //     tone: 'default' | 'primary',
    //
    // Without this, that comma split the field mid-comment and everything after it
    // became the prop's NAME — a perfectly ordinary JSDoc silently destroyed the prop
    // it was documenting. (Found by a test written for the OTHER half of this bug.)
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut prev = '\0';
    let mut cur = String::new();
    for c in s.chars() {
        if in_line_comment {
            cur.push(c);
            if c == '\n' {
                in_line_comment = false;
            }
            prev = c;
            continue;
        }
        if in_block_comment {
            cur.push(c);
            if prev == '*' && c == '/' {
                in_block_comment = false;
                prev = '\0';
                continue;
            }
            prev = c;
            continue;
        }
        if in_str.is_none() && prev == '/' && (c == '/' || c == '*') {
            if c == '/' {
                in_line_comment = true;
            } else {
                in_block_comment = true;
            }
            cur.push(c);
            prev = c;
            continue;
        }
        prev = c;
        match in_str {
            Some(q) => {
                cur.push(c);
                if c == q {
                    in_str = None;
                }
            }
            None => match c {
                '\'' | '"' => {
                    in_str = Some(c);
                    cur.push(c);
                }
                '{' | '[' => {
                    depth += 1;
                    cur.push(c);
                }
                '}' | ']' => {
                    depth -= 1;
                    cur.push(c);
                }
                d if d == delim && depth == 0 => {
                    parts.push(cur.trim().to_string());
                    cur.clear();
                }
                _ => cur.push(c),
            },
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur.trim().to_string());
    }
    parts
}

/// Index of the first top-level `:` in `s` (skips `:` inside `{}`/`[]`/strings —
/// e.g. the `:` in an sx value `'bg:tone'`).
fn top_level_colon(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    for (i, c) in s.char_indices() {
        match in_str {
            Some(q) => {
                if c == q {
                    in_str = None;
                }
            }
            None => match c {
                '\'' | '"' => in_str = Some(c),
                '{' | '[' => depth += 1,
                '}' | ']' => depth -= 1,
                ':' if depth == 0 => return Some(i),
                _ => {}
            },
        }
    }
    None
}

/// Strip a single layer of matching quotes.
fn unquote(s: &str) -> String {
    let s = s.trim();
    let b = s.as_bytes();
    if b.len() >= 2 && (b[0] == b'\'' || b[0] == b'"') && b[b.len() - 1] == b[0] {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Parse `{ key: value, … }` into `(key, raw-value)` pairs. Values keep their
/// raw text (a quoted string, a nested `{…}`, or an `[…]`) for the caller.
/// Strip a leading comment run (`/** … */`, `/* … */`, `// …`) off an object-field
/// entry, returning `(doc, rest)` where `doc` is the JSDoc text if there was one.
///
/// WHY THIS EXISTS. `split_top_level(inner, ',')` splits on commas, and a comment has
/// none — so a documented prop arrived here as
///
/// ```text
/// "/** The visual tone. */\n    tone: 'default' | 'primary'"
/// ```
///
/// and everything before the colon became the prop NAME. The compiler happily emitted
/// `__props.____The_visual_tone_______` and the prop `tone` simply did not exist. It
/// compiled clean and shipped a component whose props were gone. Documenting a prop —
/// the first thing anyone writing a real component does — silently broke it.
fn strip_leading_doc(entry: &str) -> (Option<String>, &str) {
    let mut rest = entry.trim_start();
    let mut doc: Option<String> = None;
    loop {
        if let Some(after) = rest.strip_prefix("/*") {
            let is_jsdoc = after.starts_with('*');
            let Some(end) = after.find("*/") else { break };
            if is_jsdoc && doc.is_none() {
                // `/** text */` → "text": drop the leading `*`s and the per-line `*`.
                let body = &after[1..end];
                let text = body
                    .lines()
                    .map(|l| l.trim().trim_start_matches('*').trim())
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                if !text.is_empty() {
                    doc = Some(text);
                }
            }
            rest = after[end + 2..].trim_start();
            continue;
        }
        if let Some(after) = rest.strip_prefix("//") {
            let end = after.find('\n').unwrap_or(after.len());
            rest = after[end..].trim_start();
            continue;
        }
        break;
    }
    (doc, rest)
}

/// Object fields WITH their JSDoc, when the caller wants the descriptions (props →
/// CEM → editor autocomplete, the component showcase, agent tooling).
pub(crate) fn parse_object_fields_with_docs(obj: &str) -> Vec<(String, String, Option<String>)> {
    let inner = obj
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    split_top_level(inner, ',')
        .into_iter()
        .filter_map(|entry| {
            let (doc, entry) = strip_leading_doc(entry.trim());
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            match top_level_colon(entry) {
                Some(idx) => Some((
                    unquote(entry[..idx].trim()),
                    entry[idx + 1..].trim().to_string(),
                    doc,
                )),
                None => Some((unquote(entry), String::new(), doc)),
            }
        })
        .collect()
}

pub(crate) fn parse_object_fields(obj: &str) -> Vec<(String, String)> {
    let inner = obj
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    split_top_level(inner, ',')
        .into_iter()
        .filter_map(|entry| {
            // Comments are stripped here too: EVERY caller reading a field name needs
            // the name, not the documentation glued to the front of it.
            let (_, entry) = strip_leading_doc(entry.trim());
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            // A colon-less field is a bare shorthand (`tone`, `size`) — key with
            // an empty value. Without this, ES-style shorthands were silently
            // dropped, so a built-in prop declared as `{ tone, size }` vanished.
            match top_level_colon(entry) {
                Some(idx) => Some((
                    unquote(entry[..idx].trim()),
                    entry[idx + 1..].trim().to_string(),
                )),
                None => Some((unquote(entry), String::new())),
            }
        })
        .collect()
}

/// Lower the TS-object form of a `<variants>` body to the same `(selector-dsl,
/// sx)` pairs the line DSL produces. CVA-shaped:
///   `{ variant: { solid: 'sx…' }, size: { sm: 'sx…' },`
///   `  compound: [ { when: { variant: 'solid', size: 'lg' }, sx: 'sx…' } ] }`
/// A prop group `variant: { solid: 'x' }` → `("variant=solid", "x")`; a compound
/// `{ when: { a:'1', b:'2' }, sx: 'x' }` → `("(a=1, b=2)", "x")`.
fn parse_variants_object(body: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for (key, val) in parse_object_fields(body) {
        if key == "compound" {
            let arr = val.trim().trim_start_matches('[').trim_end_matches(']');
            for obj in split_top_level(arr, ',') {
                let fields = parse_object_fields(&obj);
                let when = fields.iter().find(|(k, _)| k == "when").map(|(_, v)| v.clone());
                let sx = fields.iter().find(|(k, _)| k == "sx").map(|(_, v)| unquote(v));
                if let (Some(when), Some(sx)) = (when, sx) {
                    let conds: Vec<String> = parse_object_fields(&when)
                        .into_iter()
                        .map(|(k, v)| format!("{k}={}", unquote(&v)))
                        .collect();
                    pairs.push((format!("({})", conds.join(", ")), sx));
                }
            }
        } else {
            for (value, sx) in parse_object_fields(&val) {
                pairs.push((format!("{key}={value}"), unquote(&sx)));
            }
        }
    }
    pairs
}

/// Desugar a wrapper-less `<controller>` body — bare `export function`/`export
/// const` — into the canonical `controller({ scope })` form the pipeline reads.
/// The exported names become the template scope, so an author writes plain
/// functions with no `controller({…})` ceremony. Bodies that already call
/// `controller(` or have a `export default` pass through unchanged.
pub fn wrap_controller_body(body: &str) -> String {
    if body.contains("controller(") || body.contains("export default") {
        return body.to_string();
    }
    let mut names: Vec<String> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    for line in body.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("export function ") {
            if let Some(name) = rest.split('(').next().map(str::trim) {
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
            lines.push(line.replacen("export ", "", 1));
        } else if let Some(rest) = t.strip_prefix("export const ") {
            if let Some(name) = rest.split([' ', '=', ':']).next().map(str::trim) {
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
            lines.push(line.replacen("export ", "", 1));
        } else {
            lines.push(line.to_string());
        }
    }
    format!(
        "import {{ controller }} from '@ship/sui-runtime/internal';\n{}\nexport default controller({{ scope: () => ({{ {} }}) }});\n",
        lines.join("\n"),
        names.join(", ")
    )
}

/// First occurrence of `needle` outside an HTML comment.
///
/// This runs before there is a tree to walk, so it scans raw text — and comments
/// are exactly where the language's own tokens turn up as prose.
fn find_outside_comments(source: &str, needle: &str) -> Option<usize> {
    let mut i = 0usize;
    while i < source.len() {
        // `i` stays on a char boundary: advancing by byte and then slicing panics
        // on the first non-ASCII character in the file.
        let rest = &source[i..];
        if rest.starts_with("<!--") {
            // An unterminated comment is the parser's error to report, not ours.
            match rest[4..].find("-->") {
                Some(rel) => i += 4 + rel + 3,
                None => return None,
            }
            continue;
        }
        if rest.starts_with(needle) {
            return Some(i);
        }
        i += rest.chars().next().map_or(1, char::len_utf8);
    }
    None
}

/// Extract an inline `<controller>` block for the single-file-component model.
/// Returns the template with the block removed + the block's raw TS body (the
/// one locked-down `controller({…})` function). `None` if there's no block.
///
/// The SFC compile model (as Vue extracts its `<script>`): the compiler writes
/// this body to the generated `<stem>.controller.ts` module that the emit
/// already imports and the analysis already reads — so no emit surgery, and the
/// author writes ONE file. The body is type-checked by the tsgo bridge and the
/// existing `controller_*` lockdown lints, exactly as a paired file is today.
pub fn extract_controller_block(source: &str) -> (String, Option<String>) {
    // Skip comments: a comment above the block that mentions `<controller>` would
    // otherwise be taken as the block's opening tag, and everything from inside the
    // comment through `</controller>` spliced out — carrying the comment's `-->`
    // with it, leaving the file unparseable.
    let Some(start) = find_outside_comments(source, "<controller") else {
        return (source.to_string(), None);
    };
    let Some(open_rel) = source[start..].find('>') else {
        return (source.to_string(), None);
    };
    let body_start = start + open_rel + 1;
    let Some(rel) = source[body_start..].find("</controller>") else {
        return (source.to_string(), None);
    };
    let body_end = body_start + rel;
    let body = wrap_controller_body(source[body_start..body_end].trim());
    let block_end = body_end + "</controller>".len();
    let mut stripped = String::with_capacity(source.len());
    stripped.push_str(&source[..start]);
    stripped.push_str(&source[block_end..]);
    (stripped, Some(body))
}

/// Desugar the SFC `<component>` block — the single declarative surface — into
/// the `<prop>` tags + object-`<variants>` block the existing pipeline lowers.
/// `props:` maps built-in shorthands (`tone`/`size`) to their canonical union +
/// contract, arrays to enum props, and `bool`/`string`/`number` to typed props;
/// `variants:` becomes a `<variants>` block; `meta:` is filename-derived today.
/// Derive the framework-owned `:host` display from the component's root template
/// element — the host presents as that element naturally would. Block is the
/// framework default, so this emits an override only for inline-natured roots
/// (a link is inline, a label is inline-block, a row/button control is
/// inline-flex). No per-component declaration, no renamed CSS.
fn derive_host_display(source: &str) -> String {
    let display = match first_template_tag(source).as_deref() {
        Some("a") => "inline",
        Some("label") => "inline-block",
        Some("row") | Some("button") | Some("input") | Some("select") | Some("span") => {
            "inline-flex"
        }
        // Block-natured or unknown roots keep the framework block default.
        _ => return String::new(),
    };
    format!(":host {{ display: {display}; }}\n")
}

/// The tag name of the first real template element — skipping HTML comments and
/// the non-visual blocks the compiler generates/consumes (`<prop>`, `<variants>`,
/// `<theme>`, `<host>`, `<style>`, `<component>`, `<controller>`).
fn first_template_tag(source: &str) -> Option<String> {
    const SKIP: &[&str] = &[
        "prop",
        "variants",
        "theme",
        "host",
        "style",
        "component",
        "controller",
    ];
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        // A close tag `</…>` or a non-name char after `<` — step over it.
        let name: String = source[i + 1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        if name.is_empty() {
            i += 1;
            continue;
        }
        if SKIP.contains(&name.as_str()) {
            match source[i..].find('>') {
                Some(gt) => {
                    i += gt + 1;
                    continue;
                }
                None => break,
            }
        }
        return Some(name);
    }
    None
}

/// Return the raw body of the `<component>` block (the `{ props, variants, meta }`
/// object text), or `None` if there is no block. Used by the tsgo type-check.
pub(crate) fn extract_component_body(source: &str) -> Option<String> {
    let start = source.find("<component")?;
    let open_rel = source[start..].find('>')?;
    let body_start = start + open_rel + 1;
    let rel = source[body_start..].find("</component>")?;
    Some(source[body_start..body_start + rel].trim().to_string())
}

/// Whether the `<component>` block opts INTO shadow DOM (`shadow: true`).
///
/// Light DOM is the default for every layer (invariant 9). Shadow DOM is a
/// per-component containment FEATURE declared in the component's own manifest —
/// used only for wrapping foreign content (tiptap-class editors, FFI). The
/// SOURCE is authoritative for every compile path (bundler, napi, CLI); no
/// per-invocation flag decides it.
pub(crate) fn declares_shadow_dom(source: &str) -> bool {
    extract_component_body(source)
        .map(|body| {
            parse_object_fields(&body)
                .iter()
                .any(|(k, v)| k == "shadow" && v.trim() == "true")
        })
        .unwrap_or(false)
}

/// Parse the `form: { value: '…', disabled: '…' }` object into the two prop
/// names. Unknown keys are ignored (tsgo owns shape validation of the block).
pub(crate) fn parse_form_field(obj: &str) -> (Option<String>, Option<String>) {
    let mut value = None;
    let mut disabled = None;
    let inner = obj.trim().trim_start_matches('{').trim_end_matches('}');
    for (k, v) in parse_object_fields(inner) {
        match k.as_str() {
            "value" => value = Some(unquote(v.trim())),
            "disabled" => disabled = Some(unquote(v.trim())),
            _ => {}
        }
    }
    (value, disabled)
}

/// Build `PropDecl`s directly from the `<component>` block's `props` object.
/// `<prop>` was our own tag; the component block replaces it with props-as-ts,
/// so props lower straight to `PropDecl` — no `<prop>` tag, no desugar, no IR.
pub(crate) fn component_props_to_decls(
    obj: &str,
    form_value: Option<&str>,
    form_disabled: Option<&str>,
) -> Vec<PropDecl> {
    let mut out = Vec::new();
    // WITH_DOCS: a `/** … */` above a prop is its DESCRIPTION — it reaches the CEM,
    // which is what feeds editor autocomplete, the component showcase's prop tables,
    // and agent tooling. (It also used to be silently parsed as part of the prop's
    // NAME; see `strip_leading_doc`.)
    for (name, val, doc) in parse_object_fields_with_docs(obj) {
        let n = name.as_str();
        let val = val.trim();
        // Variant prop: reserved `variant` key with an object, or any key via
        // `variant({…})`. The prop is a union of the map's keys (first =
        // default), reflected. Styles emit separately (variant CSS).
        if let Some(map) = variant_map(n, val) {
            let keys: Vec<String> = parse_object_fields(
                map.trim().trim_start_matches('{').trim_end_matches('}'),
            )
            .into_iter()
            .map(|(k, _)| k)
            .collect();
            out.push(PropDecl {
                name: n.to_string(),
                kind: PropKind::Union(keys.clone()),
                default: keys.first().cloned(),
                reflect: true,
                bindable: false,
                form_value: form_value == Some(n),
                form_disabled: form_disabled == Some(n),
                description: doc.clone(),
                span: Span::new(0, 0),
            });
            continue;
        }
        // Object / array type (`config: { … }`, `columns: Column[]`) — caught
        // before the union/`|` logic so a `{ a: 'x' | 'y' }` isn't misread as a
        // union. Its type is authored TypeScript, preserved verbatim; a plain
        // reactive property, never an attribute.
        if val.starts_with('{') || val.ends_with("[]") {
            out.push(PropDecl {
                name: n.to_string(),
                kind: PropKind::Identity(val.to_string()),
                default: None,
                reflect: false,
                bindable: false,
                form_value: form_value == Some(n),
                form_disabled: form_disabled == Some(n),
                description: doc.clone(),
                span: Span::new(0, 0),
            });
            continue;
        }
        let (kind, default, reflect): (PropKind, Option<String>, bool) =
            if (n == "tone" || n == "size") && !val.contains('|') {
                let (vocab, builtin) = if n == "tone" {
                    ("default|primary|info|success|warning|destructive|accent|muted", "default")
                } else {
                    ("xs|sm|md|lg", "md")
                };
                let vals = vocab.split('|').map(str::to_string).collect();
                (PropKind::Union(vals), Some(builtin.to_string()), true)
            } else if n == "tone" || n == "size" {
                let builtin = if n == "tone" { "default" } else { "md" };
                let vals: Vec<String> =
                    val.split('|').map(|s| unquote(s.trim())).filter(|s| !s.is_empty()).collect();
                let default = if vals.iter().any(|v| v == builtin) {
                    builtin.to_string()
                } else {
                    vals.first().cloned().unwrap_or_default()
                };
                (PropKind::Union(vals), Some(default), true)
            } else if val.contains('|') {
                let vals: Vec<String> =
                    val.split('|').map(|s| unquote(s.trim())).filter(|s| !s.is_empty()).collect();
                let default = vals.first().cloned().unwrap_or_default();
                (PropKind::Union(vals), Some(default), true)
            } else if val.starts_with('[') {
                let inner = val.trim_start_matches('[').trim_end_matches(']');
                let vals: Vec<String> =
                    split_top_level(inner, ',').into_iter().map(|s| unquote(&s)).collect();
                let default = vals.first().cloned().unwrap_or_default();
                (PropKind::Union(vals), Some(default), true)
            } else if val == "boolean" || val == "bool" {
                (PropKind::Bool, Some("false".to_string()), true)
            } else if val == "true" || val == "false" {
                (PropKind::Bool, Some(val.to_string()), true)
            } else if val == "number" {
                (PropKind::Number, Some("0".to_string()), false)
            } else if val.parse::<f64>().is_ok() {
                (PropKind::Number, Some(val.to_string()), false)
            } else if (val.starts_with('\'') && val.ends_with('\'') && val.len() >= 2)
                || (val.starts_with('"') && val.ends_with('"') && val.len() >= 2)
            {
                (PropKind::String, Some(unquote(val)), false)
            } else if val == "string" || val == "String" {
                (PropKind::String, Some(String::new()), false)
            } else {
                // Any other RHS is authored TypeScript — a custom-typed reactive
                // property, preserved verbatim. Not attribute-backed, no default.
                (PropKind::Identity(val.to_string()), None, false)
            };
        out.push(PropDecl {
            name: n.to_string(),
            kind,
            default,
            reflect,
            bindable: false,
            form_value: form_value == Some(n),
            form_disabled: form_disabled == Some(n),
            description: doc.clone(),
            span: Span::new(0, 0),
        });
    }
    out
}

/// Recognise a variant prop's map. The reserved `variant` key with an object
/// value IS the map (no wrapper); any key wrapped in `variant({…})` is an
/// explicit variant axis. Returns the map body (`{ solid: 'sx', … }`).
fn variant_map<'a>(name: &str, val: &'a str) -> Option<&'a str> {
    if let Some(inner) = val.strip_prefix("variant(") {
        Some(inner.strip_suffix(')').unwrap_or(inner).trim())
    } else if name == "variant" && val.starts_with('{') {
        Some(val)
    } else {
        None
    }
}

/// Emit variant CSS for a component: collect the `variant` props from the `props`
/// object plus a sibling `compound` list into the CVA-shaped object the CSS core
/// consumes, then lower it. No `<variants>` block, no desugar.
pub(crate) fn component_variants_to_css(props_obj: &str, compound: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (name, val) in parse_object_fields(props_obj) {
        if let Some(map) = variant_map(name.as_str(), val.trim()) {
            parts.push(format!("{name}: {map}"));
        }
    }
    if let Some(c) = compound {
        parts.push(format!("compound: {}", c.trim()));
    }
    if parts.is_empty() {
        return String::new();
    }
    variants_object_to_css(&format!("{{ {} }}", parts.join(", ")))
}

/// Emit CSS from `(selector-dsl, sx)` variant pairs — the one variant→CSS core.
fn variant_pairs_to_css(pairs: &[(String, String)]) -> String {
    let mut css = String::new();
    for (sel, sx) in pairs {
        let Some(selector) = variant_selector(sel) else { continue };
        for (state, decls) in lower_variant_sx(sx) {
            if decls.is_empty() {
                continue;
            }
            let (media, host_pseudo, _) = classify_variant_state(&state, false);
            let rule = format!(":host({selector}{host_pseudo}){{{decls}}}");
            match media {
                Some(q) => css.push_str(&format!("@media {q}{{{rule}}}")),
                None => css.push_str(&rule),
            }
        }
    }
    css
}

/// Lower a CVA-shaped variants-object body into `:host([prop='value'])` CSS.
pub(crate) fn variants_object_to_css(body: &str) -> String {
    variant_pairs_to_css(&parse_variants_object(body))
}

/// Lower every `<theme prop="…" channels="…">` block to `:host` custom-property
/// declarations — the *channel contract* (StyleX/vanilla-extract theme model). A
/// row `value: fill1, fill2, …` fills the declared channels for that prop value;
/// `default:` fills them on the bare `:host`. Fills are token names → `var(--…)`.
/// The channels themselves become file-local sx values (see `theme_channels`).
///
///   <theme prop="tone" channels="tone-fg, tone-border">
///     default: foreground, border
///     info:    info, tone-info-border
///   </theme>
///
/// →  :host{--tone-fg:var(--foreground);--tone-border:var(--border);}
///    :host([tone='info']){--tone-fg:var(--info);--tone-border:var(--tone-info-border);}
/// A variant selector → a `:host(...)` attribute-match. `tone=success` →
/// `[tone='success']`; `[disabled]` → `[disabled]`; `(a=x, b=y)` →
/// `[a='x'][b='y']` (compound).
/// Where a variants-v2 state prefix attaches. Returns
/// `(media_query, host_pseudo, target_pseudo)`:
/// - base (`""`) → nowhere
/// - a condition (`motion-reduce`, `dark`, …) → an `@media` wrapper
/// - `group-<s>` → pseudo on the HOST (host state drives the targeted
///   descendant — the `label:hover .box` pattern); without a target it
///   degrades to a plain host state
/// - a plain state → pseudo on the target when one exists, else on the host
fn classify_variant_state(
    state: &str,
    has_target: bool,
) -> (Option<&'static str>, String, String) {
    if state.is_empty() {
        return (None, String::new(), String::new());
    }
    if let Some(q) = sui_lint::rules::sx::condition_media(state) {
        return (Some(q), String::new(), String::new());
    }
    if let Some(base) = state.strip_prefix("group-")
        && let Some(p) = sui_lint::rules::sx::state_pseudo(base)
    {
        return (None, p.to_string(), String::new());
    }
    match sui_lint::rules::sx::state_pseudo(state) {
        Some(p) if has_target => (None, String::new(), p.to_string()),
        Some(p) => (None, p.to_string(), String::new()),
        None => (None, String::new(), String::new()),
    }
}

/// One `name=value` condition → an attribute match. Bool props reflect as
/// presence attributes, so `x=true` → `[x]` and `x=false` → `:not([x])`.
fn attr_condition(k: &str, v: &str) -> String {
    match v {
        "true" => format!("[{k}]"),
        "false" => format!(":not([{k}])"),
        _ => format!("[{k}='{v}']"),
    }
}

fn variant_selector(sel: &str) -> Option<String> {
    let sel = sel.trim();
    if let Some(inner) = sel.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
        let mut out = String::new();
        for part in inner.split(',') {
            out.push_str(&attr_match(part.trim())?);
        }
        Some(out)
    } else {
        attr_match(sel)
    }
}

fn attr_match(s: &str) -> Option<String> {
    let s = s.trim();
    if s.starts_with('[') && s.ends_with(']') && s.len() > 2 {
        Some(s.to_string())
    } else if let Some((prop, val)) = s.split_once('=') {
        Some(attr_condition(prop.trim(), val.trim()))
    } else if !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
        // A bare identifier is a boolean attribute: `invalid` → `[invalid]`.
        Some(format!("[{s}]"))
    } else {
        None
    }
}

/// Lower a space-separated sx list to per-state declaration groups via the
/// shared sx vocab (variants v2). Returns `(state, "prop:value;…")` groups in
/// first-appearance order; `state` is `""` for the base group, a pseudo state
/// (`hover`, `focus`, …), a `group-<state>` propagation (host state →
/// descendant target), or a condition (`motion-reduce`, …). This is what lets
/// a variants table express `solid: 'bg:tone hover:bg:tone-bg-hover'` — the
/// per-variant hover that translation waves had to drop.
fn lower_variant_sx(list: &str) -> Vec<(String, String)> {
    let mut groups: Vec<(String, String)> = Vec::new();
    let mut push = |state: &str, decls: Vec<(&'static str, String)>| {
        let body: String = decls.iter().map(|(p, v)| format!("{p}:{v};")).collect();
        if let Some((_, existing)) = groups.iter_mut().find(|(s, _)| s == state) {
            existing.push_str(&body);
        } else {
            groups.push((state.to_string(), body));
        }
    };
    for pair in list.split_whitespace() {
        match pair.splitn(3, ':').collect::<Vec<_>>().as_slice() {
            [prop, tok] => {
                if let Some(decls) = sui_lint::rules::sx::lower(prop, tok) {
                    push("", decls);
                }
            }
            [state, prop, tok] => {
                let known = sui_lint::rules::sx::state_pseudo(state).is_some()
                    || sui_lint::rules::sx::condition_media(state).is_some()
                    || state
                        .strip_prefix("group-")
                        .is_some_and(|b| sui_lint::rules::sx::state_pseudo(b).is_some());
                if known && let Some(decls) = sui_lint::rules::sx::lower(prop, tok) {
                    push(state, decls);
                }
            }
            _ => {}
        }
    }
    groups
}

fn quote_brace_attrs(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            // Append one UTF-8 char (1–4 bytes). `bytes[i] as char` would
            // mangle multi-byte sequences like `…` (e2 80 a6) into latin-1
            // codepoints and re-encode them as gibberish.
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
                j += 1;
            }
            out.push_str(&source[i..j]);
            i = j;
            continue;
        }
        // HTML comments — skip the body verbatim. The tag-mode quote
        // tracker doesn't understand comments; an apostrophe inside
        // (e.g. "doesn't") would leak `in_single` state into every
        // later attribute on the page, breaking `={…}` brace-attr
        // quoting for the rest of the file.
        if source[i..].starts_with("<!--") {
            let body_start = i + 4;
            let close_rel = source[body_start..].find("-->");
            let end = match close_rel {
                Some(rel) => body_start + rel + 3,
                None => bytes.len(),
            };
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        out.push('<');
        i += 1;
        let mut in_single = false;
        let mut in_double = false;
        while i < bytes.len() {
            let ch = bytes[i];
            if ch == b'"' && !in_single {
                in_double = !in_double;
                out.push('"');
                i += 1;
                continue;
            }
            if ch == b'\'' && !in_double {
                in_single = !in_single;
                out.push('\'');
                i += 1;
                continue;
            }
            if !in_single && !in_double && ch == b'>' {
                out.push('>');
                i += 1;
                break;
            }
            if !in_single
                && !in_double
                && ch == b'='
                && i + 1 < bytes.len()
                && bytes[i + 1] == b'{'
                && let Some(end) = find_close(source, i + 1) {
                    let expr = &source[i + 1..=end];
                    out.push('=');
                    out.push('"');
                    for c in expr.chars() {
                        if c == '"' {
                            out.push_str("&quot;");
                        } else {
                            out.push(c);
                        }
                    }
                    out.push('"');
                    i = end + 1;
                    continue;
                }
            // Not the start of a brace attr — copy verbatim.
            // Use char boundary safe append.
            let ch_start = i;
            // Find next char boundary
            let mut j = ch_start + 1;
            while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
                j += 1;
            }
            out.push_str(&source[ch_start..j]);
            i = j;
        }
    }
    out
}

fn find_close(s: &str, open_brace_at: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut str_quote: Option<u8> = None;
    let mut i = open_brace_at;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = str_quote {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                str_quote = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            str_quote = Some(c);
            i += 1;
            continue;
        }
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn expand_self_close(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            // Append a single UTF-8 char.
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
                j += 1;
            }
            out.push_str(&source[i..j]);
            i = j;
            continue;
        }
        // Skip </…> close tags and <!…> directives whole.
        if i + 1 < bytes.len() && (bytes[i + 1] == b'/' || bytes[i + 1] == b'!') {
            if let Some(end_rel) = source[i..].find('>') {
                out.push_str(&source[i..=i + end_rel]);
                i += end_rel + 1;
                continue;
            }
            out.push_str(&source[i..]);
            break;
        }
        // Match a tag name `[A-Za-z][A-Za-z0-9-]*` after `<`.
        let tag_start = i + 1;
        let mut tag_end = tag_start;
        if tag_end >= bytes.len()
            || !(bytes[tag_end].is_ascii_alphabetic())
        {
            // Not actually a tag; copy `<` and continue.
            out.push('<');
            i += 1;
            continue;
        }
        tag_end += 1;
        while tag_end < bytes.len()
            && (bytes[tag_end].is_ascii_alphanumeric() || bytes[tag_end] == b'-')
        {
            tag_end += 1;
        }
        let tag = source[tag_start..tag_end].to_ascii_lowercase();
        // Find matching `>`.
        let mut j = tag_end;
        let mut in_single = false;
        let mut in_double = false;
        while j < bytes.len() {
            let ch = bytes[j];
            if ch == b'"' && !in_single {
                in_double = !in_double;
            } else if ch == b'\'' && !in_double {
                in_single = !in_single;
            } else if !in_single && !in_double && ch == b'>' {
                break;
            }
            j += 1;
        }
        if j == bytes.len() {
            out.push_str(&source[i..]);
            break;
        }
        let head = &source[i..j];
        // Self-close requires an actual `/` before `>`. Trailing whitespace
        // alone (multi-line attr layout `<button\n  …\n>`) MUST NOT be
        // treated as self-close — that would flatten every multi-line
        // element's children into siblings.
        let trimmed_ws = head.trim_end_matches(|c: char| c.is_whitespace());
        let is_self_close = trimmed_ws.ends_with('/');
        if is_self_close && !VOID_TAGS.contains(&tag.as_str()) {
            // Strip the slash, then any whitespace before it (`<x  />` → `<x>`).
            let trimmed_head = trimmed_ws
                .trim_end_matches('/')
                .trim_end_matches(|c: char| c.is_whitespace());
            out.push_str(trimmed_head);
            out.push('>');
            out.push_str("</");
            out.push_str(&tag);
            out.push('>');
        } else {
            out.push_str(&source[i..=j]);
        }
        i = j + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A comment may name `<controller>` above the block it describes.
    #[test]
    fn a_comment_may_name_the_controller_block_it_describes() {
        let src = "<!-- counter.sui — when there's state, add an inline <controller> block. -->\n\
                   <controller>\nexport default controller({})\n</controller>\n<p>hi</p>";
        let (stripped, body) = extract_controller_block(src);
        assert!(body.is_some(), "the REAL block is the one extracted");
        assert!(
            stripped.contains("-->"),
            "and the comment survives intact — it was not spliced through:\n{stripped}"
        );
        assert!(
            !stripped.contains("</controller>"),
            "while the real block is gone from the template:\n{stripped}"
        );
    }

    /// ...and a comment with no real block after it must not invent one.
    #[test]
    fn a_controller_named_only_in_a_comment_is_not_a_block() {
        let src = "<!-- state goes in a <controller> block -->\n<p>hi</p>";
        let (stripped, body) = extract_controller_block(src);
        assert!(body.is_none(), "there is no controller here — only prose about one");
        assert_eq!(stripped, src, "and the source is untouched");
    }

    /// Non-ASCII markup must not panic the scan: advancing by byte and slicing
    /// lands mid-codepoint on the first `…` or `é`.
    #[test]
    fn prose_with_non_ascii_does_not_panic_the_scan() {
        let src = "<aside>…</aside>\n<p>naïve — “curly” ✓</p>\n<controller>\nexport default controller({})\n</controller>";
        let (_, body) = extract_controller_block(src);
        assert!(body.is_some(), "the block is still found past the non-ASCII text");
        // And with no block at all, the scan must still walk off the end cleanly.
        let (s2, b2) = extract_controller_block("<p>… é — ✓</p>");
        assert!(b2.is_none());
        assert_eq!(s2, "<p>… é — ✓</p>");
    }

    #[test]
    fn quotes_unquoted_brace_attr() {
        let out = quote_brace_attrs(r#"<x disabled={a || b} />"#);
        assert!(out.contains(r#"disabled="{a || b}""#));
    }

    #[test]
    fn escapes_double_quote_inside_expr() {
        let out = quote_brace_attrs(r#"<x v={a("z")}>"#);
        assert!(out.contains(r#"v="{a(&quot;z&quot;)}""#));
    }

    #[test]
    fn expands_non_void_selfclose() {
        let out = expand_self_close("<custom-tag attr=\"v\" />");
        assert_eq!(out, "<custom-tag attr=\"v\"></custom-tag>");
    }

    #[test]
    fn leaves_void_selfclose() {
        let out = expand_self_close("<br />");
        assert_eq!(out, "<br />");
    }

    #[test]
    fn multiline_open_tag_is_not_self_close() {
        // Multi-line attribute layout — trailing newline before `>` must
        // NOT be treated as a self-close marker. Regression for sui-button
        // emitting children as siblings instead of nesting them inside.
        let src = "<button\n  type=\"button\"\n  disabled\n>\n  <span>x</span>\n</button>";
        let out = expand_self_close(src);
        assert_eq!(out, src, "multi-line open tag should pass through unchanged");
    }

    #[test]
    fn slash_with_leading_whitespace_is_self_close() {
        // `<x  />` (whitespace before `/`) is still XHTML-style self-close.
        let out = expand_self_close("<x  />");
        assert_eq!(out, "<x></x>");
    }

    #[test]
    fn sx_merges_into_existing_class_attr() {
        // Regression: `<span class="badge" sx="p:4">` used to lower into TWO
        // class attributes; the emitted setAttribute("class") pair let the sx
        // one clobber "badge" at runtime (sui-badge `.badge` querySelector
        // returned null — 82 primitive-suite failures).
        let out = preprocess(r#"<span class="badge" sx="p:4">x</span>"#);
        assert!(
            out.contains(r#"class="badge sx-p-4""#),
            "author class + sx classes must merge into one attribute, got:\n{out}"
        );
        assert_eq!(out.matches("class=").count(), 1, "exactly one class attr:\n{out}");
    }

    #[test]
    fn sx_before_class_also_merges() {
        let out = preprocess(r#"<span sx="p:4" class="badge">x</span>"#);
        assert!(
            out.contains(r#"class="sx-p-4 badge""#),
            "merge must work regardless of attribute order, got:\n{out}"
        );
        assert_eq!(out.matches("class=").count(), 1);
    }

    #[test]
    fn dynamic_class_with_sx_is_left_alone() {
        // A `{…}` class binding is the emitter's business — the merge pass
        // must not fold static sx classes into an expression it can't evaluate.
        let out = preprocess(r#"<span class={cls()} sx="p:4">x</span>"#);
        assert_eq!(
            out.matches("class=").count(),
            2,
            "dynamic class + sx stays two attrs for the emitter, got:\n{out}"
        );
    }

    #[test]
    fn quoted_gt_inside_attr_does_not_break_class_merge() {
        let out = preprocess(r#"<span title="a > b" class="x" sx="p:4">y</span>"#);
        assert!(out.contains(r#"class="x sx-p-4""#), "got:\n{out}");
    }

    #[test]
    fn stdlib_spinner_lowers_at_use_site() {
        let out = preprocess(r#"<div class="page"><spinner size="sm" aria-label="Saving"></spinner></div>"#);
        // Root class stamped, use-site attrs merged, use-site wins the
        // aria-label collision with the snippet default.
        assert!(out.contains("sui-spinner spin"), "{out}");
        assert!(out.contains(r#"size="sm""#), "{out}");
        assert!(out.contains(r#"aria-label="Saving""#), "{out}");
        assert!(!out.contains("aria-label=\"Loading\""), "use-site wins: {out}");
        assert!(out.contains(r#"class="ring""#), "structure spliced: {out}");
        // The SKIN — including its @keyframes — ships in element-skins.css, not in the
        // component's emitted <style>. This test guards the LOWERING, which is what
        // changed nothing: structure, root class, and use-site attributes.
        assert!(!out.contains(".sui-spinner[size='sm']"), "skin is in the sheet: {out}");
        assert!(!out.contains("@keyframes sui-spin"), "keyframes ride with it: {out}");
    }

    #[test]
    fn stdlib_separator_lowers_and_dedupes_skin() {
        let out = preprocess("<separator orientation=\"vertical\"></separator><separator></separator>");
        assert_eq!(out.matches("<hr").count(), 2, "{out}");
        assert!(out.contains("sui-separator"), "{out}");
        assert!(out.contains(r#"orientation="vertical""#), "{out}");
        // The SKIN is not here. Stdlib element skins ship in the framework stylesheet
        // (`element-skins.css`), not spliced into every component's emitted <style> —
        // inlining duplicated the same rules into every chunk that used the element
        // (28.7 KB of CSS inside the JS of one 12-component page). What this test
        // guards is the LOWERING, which is unchanged: the element becomes the right
        // structure with the right classes and the use-site attributes on it.
        assert!(
            !out.contains("--tone-bg: var(--border-subtle)"),
            "the skin comes from element-skins.css, not the emitted style: {out}"
        );
    }

    #[test]
    fn stdlib_element_children_fill_default_slot_or_drop() {
        // spinner has no default slot — children drop (shadow parity).
        let out = preprocess("<spinner><b>x</b></spinner>");
        assert!(!out.contains("<b>x</b>"), "slotless children drop: {out}");
    }

    #[test]
    fn stdlib_alert_projects_named_slots_end_to_end() {
        // The first registry element with named slots — proves compile-time
        // projection distributes use-site slotted children into the matching
        // regions (icon / title / actions) and the un-slotted node into body.
        let out = preprocess(
            r#"<alert tone="success" dismissible><svg slot="icon" class="i"></svg><span slot="title">Saved</span>All changes saved.<button slot="actions">Undo</button></alert>"#,
        );
        // root merges use-site attrs
        assert!(out.contains("sui-alert"), "root class stamped: {out}");
        assert!(out.contains(r#"tone="success""#), "tone merged: {out}");
        assert!(out.contains("dismissible"), "dismissible merged: {out}");
        assert!(out.contains("use:alert"), "behavior action rides the splice: {out}");
        // each slot region receives its projected child, none survive as <slot>
        assert!(!out.contains("<slot"), "no slot tags survive projection: {out}");
        assert!(
            out.contains(r#"class="ico"><svg slot="icon" class="i"></svg>"#),
            "icon → .ico: {out}"
        );
        assert!(
            out.contains(r#"class="title"><span slot="title">Saved</span>"#),
            "title → .title: {out}"
        );
        assert!(
            out.contains(r#"class="body">All changes saved."#),
            "un-slotted → body: {out}"
        );
        assert!(
            out.contains(r#"class="actions"><button slot="actions">Undo</button>"#),
            "actions → .actions: {out}"
        );
    }

    #[test]
    fn stdlib_hyphenated_element_lowers_and_projects() {
        // empty-state is the first multi-word (hyphenated) L1 element — proves
        // the registry-gated naming seam (a hyphenated REGISTERED tag compiles
        // away) plus named-slot projection through it.
        let out = preprocess(
            r#"<empty-state variant="placeholder"><svg slot="icon"></svg><span slot="title">No results</span>Try a different search.<button slot="action">Reset</button></empty-state>"#,
        );
        assert!(out.contains("sui-empty-state"), "root class stamped: {out}");
        assert!(out.contains(r#"variant="placeholder""#), "attr merged: {out}");
        assert!(out.contains("use:emptyState"), "action rides the splice: {out}");
        assert!(!out.contains("<slot"), "no slot survives: {out}");
        assert!(out.contains(r#"class="icon" aria-hidden="true"><svg slot="icon"></svg>"#), "icon slot: {out}");
        assert!(out.contains(r#"class="title"><span slot="title">No results</span>"#), "title slot: {out}");
        assert!(out.contains(r#"class="desc">Try a different search."#), "default → desc: {out}");
        assert!(out.contains(r#"class="actions"><button slot="action">Reset</button>"#), "action slot: {out}");
    }

    #[test]
    fn project_children_partitions_by_slot_attr() {
        let (def, named) =
            project_children(r#"<i slot="leading">L</i>Body<b slot="actions">A</b>more"#);
        assert_eq!(def, "Bodymore", "un-slotted nodes → default, in order");
        assert_eq!(named.get("leading").map(String::as_str), Some(r#"<i slot="leading">L</i>"#));
        assert_eq!(named.get("actions").map(String::as_str), Some(r#"<b slot="actions">A</b>"#));
    }

    #[test]
    fn project_children_assigns_whole_subtree_not_nested_slot() {
        // a `slot=` deeper in the tree belongs to that descendant, not hoisted.
        let (def, named) =
            project_children(r#"<div slot="a"><span slot="b">in</span></div>tail"#);
        assert_eq!(
            named.get("a").map(String::as_str),
            Some(r#"<div slot="a"><span slot="b">in</span></div>"#)
        );
        assert!(named.get("b").is_none(), "nested slot not hoisted");
        assert_eq!(def, "tail");
    }

    #[test]
    fn fill_slots_projects_named_default_and_keeps_fallback() {
        let mut named = std::collections::HashMap::new();
        named.insert("leading".to_string(), "<svg></svg>".to_string());
        let rest = r#">.<span class="lead"><slot name="leading"></slot></span><slot></slot><span><slot name="empty">FB</slot></span></div>"#;
        let out = fill_slots(rest, "DEFAULT", &named);
        assert!(out.contains(r#"<span class="lead"><svg></svg></span>"#), "named slot filled: {out}");
        assert!(out.contains(">DEFAULT<"), "default slot filled: {out}");
        assert!(out.contains(">FB<"), "unmatched named slot keeps fallback: {out}");
        assert!(!out.contains("<slot"), "no slot tags survive: {out}");
    }

    #[test]
    fn stdlib_icon_lowers_at_use_site() {
        let out = preprocess(r#"<icon size="lg"><svg viewBox="0 0 24 24"></svg></icon>"#);
        // Root class stamped, use-site attr forwarded, slotted svg spliced in.
        assert!(out.contains("sui-icon"), "{out}");
        assert!(out.contains(r#"size="lg""#), "{out}");
        assert!(out.contains("<svg viewBox=\"0 0 24 24\"></svg>"), "svg slotted: {out}");
        // The skin ships in element-skins.css; the LOWERING is what this guards.
        assert!(!out.contains(".sui-icon[size='lg']"), "skin is in the sheet: {out}");
        assert_eq!(
            out.matches("--sz: var(--icon-size-md)").count(),
            0,
            "skin once: {out}"
        );
    }

    #[test]
    fn stdlib_label_is_native_kind_skin_only() {
        // label's template root IS <label> — NATIVE KIND: the tag is never
        // spliced (splicing would rewrite every real HTML label inside
        // component templates — the sonnet-agent find), the skin keys on the
        // bare tag selector and joins once when the tag is used.
        let out = preprocess(r#"<label for="email" required>Email</label>"#);
        assert!(!out.contains("sui-label"), "native kind never splices: {out}");
        assert!(out.contains(r#"<label for="email" required>Email</label>"#), "{out}");
        // The SKIN is not here. Stdlib element skins ship in the framework stylesheet
        // (`element-skins.css`), not spliced into every component's emitted <style> —
        // inlining duplicated the same rules into every chunk that used the element
        // (28.7 KB of CSS inside the JS of one 12-component page). What this test
        // guards is the LOWERING, which is unchanged: the element becomes the right
        // structure with the right classes and the use-site attributes on it.
        assert!(
            !out.contains("font-family: var(--font-body)"),
            "the tag-keyed skin ships in element-skins.css: {out}"
        );
    }

    #[test]
    fn stdlib_pass_never_touches_style_blocks() {
        // The other sonnet-agent find: CSS text mentioning an element name
        // (a /* comment */ or selector containing `<label>`-ish text) must
        // pass through untouched — <style> blocks are opaque to the pass.
        let src = "<style>\n/* the <spinner> and <label> here are prose */\n.x { color: red; }\n</style>\n<div class=\"y\">z</div>";
        let out = preprocess(src);
        assert!(out.contains("/* the <spinner> and <label> here are prose */"), "{out}");
        assert!(out.contains(".x { color: red; }"), "{out}");
        assert!(!out.contains("sui-spinner"), "no splice inside style: {out}");
    }

    #[test]
    fn bare_enum_shorthand_desugars_on_framework_tags() {
        let out = preprocess(r#"<button primary solid sm on:click={f}>Go</button>"#);
        assert!(out.contains(r#"tone="primary""#), "{out}");
        assert!(out.contains(r#"variant="solid""#), "{out}");
        assert!(out.contains(r#"size="sm""#), "{out}");
        let out = preprocess(r#"<sui-chip accent outline>x</sui-chip>"#);
        assert!(out.contains(r#"tone="accent""#) && out.contains(r#"variant="outline""#), "{out}");
    }

    #[test]
    fn bare_enum_shorthand_respects_explicit_and_scope() {
        // Explicit attribute wins — no double tone.
        let out = preprocess(r#"<button tone="info" primary>x</button>"#);
        assert!(out.contains(r#"tone="info""#), "{out}");
        assert!(!out.contains(r#"tone="primary""#), "explicit wins: {out}");
        assert!(out.contains(" primary"), "bare word passes through: {out}");
        // Userland/HTML tags are untouched.
        let out = preprocess(r#"<my-card primary>x</my-card><p sm>y</p>"#);
        assert!(out.contains("<my-card primary>"), "{out}");
        assert!(out.contains("<p sm>"), "{out}");
        // Boolean props and quoted values never collide.
        let out = preprocess(r#"<button disabled data-x="sm primary">x</button>"#);
        assert!(out.contains(" disabled"), "{out}");
        assert!(out.contains(r#"data-x="sm primary""#), "quoted values untouched: {out}");
    }

    #[test]
    fn props_default_literals_are_honored() {
        let decls = component_props_to_decls("{ label: 'Loading', centered: false, count: 3 }", None, None);
        let get = |n: &str| decls.iter().find(|p| p.name == n).unwrap();
        assert_eq!(get("label").kind, PropKind::String);
        assert_eq!(get("label").default.as_deref(), Some("Loading"));
        assert_eq!(get("centered").kind, PropKind::Bool);
        assert_eq!(get("centered").default.as_deref(), Some("false"));
        assert!(get("centered").reflect);
        assert_eq!(get("count").kind, PropKind::Number);
        assert_eq!(get("count").default.as_deref(), Some("3"));
    }

    #[test]
    fn object_array_and_custom_typed_props_are_identity_props() {
        // Object/array/custom-typed props are plain reactive properties: the
        // authored TS type is preserved verbatim, nothing reflects to an
        // attribute, nothing is mistyped as a stringly prop. A recognized
        // primitive (`label: string`) is unaffected.
        let props = "{ label: string, config: { a: 'x' | 'y' }, columns: Column[], data: DataPoint }";
        let decls = component_props_to_decls(props, None, None);
        let get = |n: &str| decls.iter().find(|p| p.name == n).unwrap();
        // `{ a: 'x' | 'y' }` keeps its `|` — it's an object type, not a union.
        assert_eq!(get("config").kind, PropKind::Identity("{ a: 'x' | 'y' }".into()));
        assert_eq!(get("columns").kind, PropKind::Identity("Column[]".into()));
        // A bare custom type is preserved, not stringly-typed as before.
        assert_eq!(get("data").kind, PropKind::Identity("DataPoint".into()));
        for n in ["config", "columns", "data"] {
            assert!(!get(n).reflect, "{n} must not reflect");
            assert!(get(n).default.is_none(), "{n} has no scalar default");
        }
        assert_eq!(get("label").kind, PropKind::String);
    }

    #[test]
    fn shadow_is_opt_in_light_is_the_default() {
        // Light DOM is the default (invariant 9): no declaration ⇒ not shadow.
        assert!(!declares_shadow_dom("<component>{ props: { tone: union } }</component>"));
        // `shadow: true` is the per-component opt-in (foreign-content wrapping).
        assert!(declares_shadow_dom("<component>{ shadow: true, props: {} }</component>"));
    }

    #[test]
    fn unknown_component_field_is_rejected_generically() {
        // Retired constructs (`light`, `variants`) and typos are rejected by a
        // generic unknown-field error — the compiler names the valid set, not
        // the old forms (no fossilization).
        let err = crate::parse::parse(
            "<component>{ light: true, props: {} }</component><div>x</div>",
        )
        .unwrap_err();
        assert!(err.message.contains("unknown `<component>` field `light`"), "{}", err.message);
        // The valid fields all parse cleanly.
        assert!(crate::parse::parse("<component>{ props: {} }</component><div>x</div>").is_ok());
        assert!(
            crate::parse::parse("<component>{ shadow: true, props: {} }</component><div>x</div>")
                .is_ok()
        );
    }

    #[test]
    fn component_form_field_flags_the_named_props() {
        let decls = component_props_to_decls(
            "{ checked: bool, disabled: bool, name: string }",
            Some("checked"),
            Some("disabled"),
        );
        let get = |n: &str| decls.iter().find(|p| p.name == n).unwrap();
        assert_eq!(get("checked").kind, PropKind::Bool);
        assert!(get("checked").form_value);
        assert!(get("disabled").form_disabled);
        assert!(!get("name").form_value && !get("name").form_disabled);
    }

    #[test]
    fn component_form_field_before_props_also_flags() {
        let decls = component_props_to_decls("{ value: string }", Some("value"), None);
        assert!(decls.iter().find(|p| p.name == "value").unwrap().form_value);
    }

    #[test]
    fn variant_prop_is_union_and_emits_css() {
        let props = "{ variant: { solid: 'bg:primary', outline: 'border:primary' }, loading: false }";
        let decls = component_props_to_decls(props, None, None);
        let variant = decls.iter().find(|p| p.name == "variant").unwrap();
        assert_eq!(variant.kind, PropKind::Union(vec!["solid".into(), "outline".into()]));
        assert_eq!(variant.default.as_deref(), Some("solid"));
        assert!(variant.reflect);
        let css = component_variants_to_css(props, None);
        assert!(css.contains("[variant='solid']"), "got: {css}");
        assert!(css.contains("[variant='outline']"), "got: {css}");
    }

    #[test]
    fn multibyte_chars_do_not_panic_class_merge() {
        // Regression: the merge pass sliced `source[i + 1..]` while `i` sat on
        // a multi-byte char (sui-date-picker's `‹`/`›` in a comment) — panic.
        let src = "<!-- ‹prev / next› —— déjà-vu … -->\n<span class=\"x\" sx=\"p:4\">é…‹›</span>";
        let out = preprocess(src);
        assert!(out.contains(r#"class="x sx-p-4""#), "got:\n{out}");
        assert!(out.contains("é…‹›"), "text content preserved:\n{out}");
    }

    #[test]
    fn full_pipeline() {
        let src = r#"<data name="agents" source="ship.agents.summaries()" live />
<button on:click={() => x()}>hi</button>"#;
        let out = preprocess(src);
        assert!(out.contains("</data>"));
        assert!(out.contains(r#"on:click="{() =&gt; x()}""#) || out.contains(r#"on:click="{() => x()}""#));
    }

    #[test]
    fn html_comment_with_apostrophe_does_not_leak_quote_state() {
        // Regression: an apostrophe inside an HTML comment (e.g.
        // "doesn't") used to toggle the preprocessor's `in_single`
        // tracker, which then leaked into every subsequent attribute
        // — every brace-attr after the comment failed to get quoted
        // and the HTML parser later rejected the unquoted `={` shape.
        // Real repro: views/eval/page.sui's header comment.
        let src = r#"<!--
  Note: the renderable subset doesn't admit `.items({…})` here.
-->
<article attr:data-passed={c ? 'true' : 'false'}></article>"#;
        let out = preprocess(src);
        assert!(
            out.contains(r#"attr:data-passed="{c ? 'true' : 'false'}""#),
            "brace-attr after a comment containing an apostrophe must \
             still get quoted, got:\n{out}",
        );
    }

    #[test]
    fn html_comment_with_nested_tag_does_not_leak() {
        // The fake `<data>` inside this comment used to be read as a
        // tag boundary. The first `>` (inside `<data>`) ended the
        // inner-while's "tag mode" early; subsequent text was
        // misclassified. With the comment-aware skip, the body is
        // copied verbatim and the real `<button>` is parsed correctly.
        let src = r#"<!-- see <data> usage --><button on:click={() => f()}>x</button>"#;
        let out = preprocess(src);
        assert!(out.contains("</data>") || out.contains("<data>"));
        assert!(
            out.contains(r#"on:click="{() => f()}""#)
                || out.contains(r#"on:click="{() =&gt; f()}""#),
            "on:click brace-attr after comment must get quoted, got:\n{out}",
        );
    }

    #[test]
    fn lower_sx_handles_multibyte_chars_without_panicking() {
        // Regression: `lower_sx` byte-walked with `i += 1` and sliced
        // `source[i..]`, panicking mid-codepoint on any multi-byte char — the
        // crash the async golden fixtures hit (an em-dash / ellipsis in copy).
        let src = "<p>Failed — retry…</p>\n<sui-card sx=\"p:4\">café ☕</sui-card>";
        let out = preprocess(src);
        assert!(out.contains("café ☕"), "multibyte content preserved: {out}");
        assert!(out.contains("class=\"sx-p-4\""), "sx still lowered: {out}");
    }

    #[test]
    fn lowers_column_to_div_with_classes_and_style() {
        let out = preprocess(r#"<column gap="4"><p>hi</p></column>"#);
        assert!(out.contains(r#"<div class="sui-col sui-g-4 sui-ai-stretch">"#), "col lowered: {out}");
        assert!(out.contains("</div>"), "close tag lowered: {out}");
        assert!(!out.contains("<column"), "no column tag remains: {out}");
        assert!(out.contains(".sui-col{display:flex;flex-direction:column"), "base css: {out}");
        assert!(out.contains(".sui-g-4{gap:var(--space-4)}"), "gap css: {out}");
    }

    #[test]
    fn layout_element_sx_merges_into_one_class() {
        // sx lowers first (→ class), then layout folds it into the div's classes.
        let out = preprocess(r#"<column gap="2" sx="p:4">x</column>"#);
        assert!(out.contains(r#"class="sui-col sui-g-2 sui-ai-stretch sx-p-4""#), "merged: {out}");
        assert_eq!(out.matches("class=").count(), 1, "exactly one class attr: {out}");
    }

    #[test]
    fn passes_through_id_and_merges_static_class() {
        let out = preprocess(r#"<row id="bar" class="mine">x</row>"#);
        assert!(out.contains(r#"id="bar""#), "id preserved: {out}");
        assert!(out.contains("sui-row") && out.contains("mine"), "class merged: {out}");
    }

    #[test]
    fn nested_layout_lowers_every_box() {
        let out = preprocess(r#"<column><row><center>x</center></row></column>"#);
        assert_eq!(out.matches("<div").count(), 3, "three divs: {out}");
        assert_eq!(out.matches("</div>").count(), 3, "three closes: {out}");
    }

    // --- `<grid cols="…">` (m-sui-stdlib / job/sui-oss-grid) ---------------
    //
    // The original bug: `cols="7"` was treated as an unknown attribute and
    // silently flowed through to the lowered `<div>` as a dead HTML attr —
    // collapsing the calendar grid to a single column with no compile error.
    // These tests pin both halves of the fix:
    //   (1) `<grid cols="7">` lowers to a fixed-column grid (the API works).
    //   (2) An unknown attribute name on a layout keyword is dropped from
    //       the emit (the lint surfaces it as a hard error before this
    //       pass runs; we defend in depth here so a future regression on
    //       the lint side can't quietly produce broken HTML again).

    #[test]
    fn grid_cols_emits_repeat_minmax_track_list() {
        let out = preprocess(r#"<grid cols="7" gap="4">x</grid>"#);
        assert!(
            out.contains(".sui-gc-7{grid-template-columns:repeat(7,minmax(0,1fr))}"),
            "cols lowering produced the repeat(minmax) track: {out}"
        );
        assert!(out.contains("sui-gc-7"), "the atomic class is present: {out}");
        assert!(out.contains("sui-grid"), "the base grid class still applied: {out}");
        assert!(out.contains("sui-g-4"), "other axes still apply: {out}");
        assert!(!out.contains("cols=\"7\""), "cols doesn't leak as a raw attr: {out}");
    }

    #[test]
    fn grid_cols_precedence_min_is_suppressed() {
        // `cols` and `min` both own `grid-template-columns`; the fixed count
        // wins. With both present the responsive axis is dropped so the
        // class list doesn't double the property.
        let out = preprocess(r#"<grid cols="5" min="lg">x</grid>"#);
        assert!(out.contains("sui-gc-5"), "cols class applied: {out}");
        assert!(!out.contains("sui-gm-"), "min class suppressed: {out}");
        assert!(
            !out.contains("auto-fill"),
            "no auto-fill track when cols is set: {out}"
        );
    }

    #[test]
    fn grid_cols_dynamic_lowers_to_a_class_binding_with_all_rules() {
        // Same dynamic-axis shape as `<row align={pick()}>`: an inline
        // expression concatenated with the prefix, and EVERY rule for the
        // axis injected so any runtime value matches.
        let out = preprocess(r#"<grid cols={n()}>x</grid>"#);
        assert!(out.contains("'sui-gc-' + (n())"), "dynamic cols fragment: {out}");
        for n in 1..=12 {
            assert!(
                out.contains(&format!(".sui-gc-{n}{{grid-template-columns:repeat({n},minmax(0,1fr))}}")),
                "rule for cols={n} present: {out}"
            );
        }
    }

    #[test]
    fn unknown_layout_attribute_is_dropped_not_passed_through() {
        // Defensive: even if the lint weren't running, an unknown attr on a
        // layout keyword must not silently flow through to the lowered div.
        // The compile-emitted output must not carry a dead `bogus="x"`.
        let out = preprocess(r#"<grid bogus="x">x</grid>"#);
        assert!(!out.contains("bogus="), "unknown attr dropped from emit: {out}");
        assert!(out.contains("sui-grid"), "the base layout class still there: {out}");
    }

    #[test]
    fn recognized_passthroughs_ride_to_the_div() {
        // The compiler's `lower_layout` consults the lint's
        // `is_passthrough_attr` (single source of truth). Assert each known
        // pass-through survives on the lowered div.
        let out = preprocess(
            r#"<grid id="hero" title="t" data-x="1" aria-label="x" role="region">x</grid>"#,
        );
        assert!(out.contains(r#"id="hero""#), "id survived: {out}");
        assert!(out.contains(r#"title="t""#), "title survived: {out}");
        assert!(out.contains(r#"data-x="1""#), "data-* survived: {out}");
        assert!(out.contains(r#"aria-label="x""#), "aria-* survived: {out}");
        assert!(out.contains(r#"role="region""#), "role survived: {out}");
    }
    #[test]
    fn directive_prefixes_ride_to_the_div() {
        // SUI directive attrs (`bind:this`, `class:out`, `style:transform`,
        // `style:--custom-prop`, etc.) are framework bindings — they ride
        // through the layout lowering unchanged so the compiler's later
        // directive-handling pass can resolve them. The lint acknowledges
        // the same vocabulary, so they are not "unknown attribute" errors.
        let out = preprocess(
            r#"<row bind:this={panelEl} class:out={!cr.cell.inMonth} style:transform={t()}>x</row>"#,
        );
        assert!(out.contains("bind:this="), "bind:this survives: {out}");
        assert!(out.contains("class:out="), "class:out survives: {out}");
        assert!(out.contains("style:transform="), "style:transform survives: {out}");
    }

    #[test]
    fn column_direction_row_lowers_to_flex_direction_row_override() {
        // Backward-compat: `<column direction="row">` was the old
        // `<sui-stack direction="row">` idiom — the keyword lowering
        // needs to honor it as an explicit flex-direction override so
        // legacy components keep their visual layout.
        let out = preprocess(r#"<column gap="2" direction="row">x</column>"#);
        assert!(out.contains("sui-dir-row"), "direction class emitted: {out}");
        assert!(
            out.contains(".sui-dir-row{flex-direction:row}"),
            "direction rule emitted: {out}"
        );
        // No `sui-dir-column` because there's no default.
        assert!(!out.contains("sui-dir-column"), "no implicit direction: {out}");
    }

    #[test]
    fn dynamic_axis_lowers_to_a_class_binding_with_full_rules() {
        // a component parameterizing its layout: `align={pick()}`
        let out = preprocess(r#"<column align={pick()}>x</column>"#);
        assert!(out.contains(r#"class="{"#), "emits a dynamic class binding: {out}");
        assert!(out.contains("'sui-ai-' + (pick())"), "dynamic align fragment: {out}");
        // EVERY align rule injected, so whatever `pick()` returns has a match.
        assert!(
            out.contains(".sui-ai-start{") && out.contains(".sui-ai-center{") && out.contains(".sui-ai-stretch{"),
            "full align rule set injected: {out}"
        );
    }

    #[test]
    fn condition_prefix_wraps_in_media() {
        let out = preprocess(r#"<div sx="motion-reduce:shadow:none forced-colors:border:strong">x</div>"#);
        assert!(out.contains("@media (prefers-reduced-motion: reduce){.sx-motion-reduce-shadow-none{box-shadow:none}}"), "{out}");
        assert!(out.contains("@media (forced-colors: active){.sx-forced-colors-border-strong{border:1px solid var(--border-strong)}}"), "{out}");
    }

    #[test]
    fn unterminated_comment_copies_to_eof() {
        // Defensive: a malformed file with no closing `-->` shouldn't
        // panic or read past end-of-file. Copy the rest verbatim and
        // let the downstream parser produce its own diagnostic.
        let src = "<!-- never closed";
        let out = preprocess(src);
        assert_eq!(out, src);
    }

}
