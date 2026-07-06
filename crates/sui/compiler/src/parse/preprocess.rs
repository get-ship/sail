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

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "source", "track", "wbr",
];

pub fn preprocess(source: &str) -> String {
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
    // Desugar the SFC `<component>` block into the `<prop>` tags + object
    // `<variants>` the rest of the chain already lowers.
    let s = lower_component(&s);
    // Framework-owned host display: the component presents as its root element
    // naturally would (inheriting the standard, not a per-component knob). Block
    // is the default, so this only emits an override for inline-natured roots.
    // Computed now, before lower_layout rewrites <row>/<column> into <div>.
    let host_display_css = derive_host_display(&s);
    // `<host>` first: it consumes its own `sx` into a `:host{…}` rule before
    // `lower_sx` would otherwise fold that sx into a class on the (non-)element.
    let (s, host_css) = lower_host(&s);
    let (s, sx_css) = lower_sx(&s);
    // An element that already carried `class="…"` now has a SECOND class attr
    // from the sx lowering; downstream the DOM emitter would issue two
    // `setAttribute("class", …)` calls and the sx one clobbers the author's.
    // Merge them into one attribute here, while the tag text is still flat.
    let s = merge_duplicate_class_attrs(&s);
    let (s, layout_css) = lower_layout(&s);
    let (s, variants_css) = lower_variants(&s);
    let (s, theme_css) = lower_theme(&s);
    let css =
        format!("{host_display_css}{host_css}{theme_css}{sx_css}{layout_css}{variants_css}");
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
        let (media, prefix, pseudo): (&str, String, &str) = match state {
            None => ("", String::new(), ""),
            Some(s) => {
                if let Some(m) = sui_lint::rules::sx::condition_media(s) {
                    (m, String::new(), "")
                } else if let Some(base) = s.strip_prefix("group-") {
                    match sui_lint::rules::sx::state_pseudo(base) {
                        Some(p) => ("", format!(".sui-grp{p} "), ""),
                        None => continue,
                    }
                } else {
                    match sui_lint::rules::sx::state_pseudo(s) {
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
            for &prop in layout::tag_props(&t.tag) {
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
                    "gap" | "align" | "justify" | "min" | "flow" => {}
                    "class" if value.starts_with('{') => {
                        dyn_parts.push(format!("({})", &value[1..value.len() - 1]));
                    }
                    "class" => {
                        if !value.is_empty() {
                            static_parts.push(value.clone());
                        }
                    }
                    _ => {
                        rest.push(' ');
                        rest.push_str(name);
                        if !value.is_empty() {
                            rest.push_str("=\"");
                            rest.push_str(value);
                            rest.push('"');
                        }
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

/// Lower a `<variants>` block — the component's declarative variant table — into
/// `:host([prop='value']){…}` CSS. Each line is `<selector>: <sx-list>`:
/// ```text
///   tone=success:            bg:success-bg fg:success   → :host([tone='success'])
///   size=lg:                 text:lg min-h:12
///   [disabled]:              opacity:50 cursor:not-allowed → :host([disabled])
///   (tone=primary, size=lg): shadow:md                  → :host([tone='primary'][size='lg'])
/// ```
/// Values lower through the SAME `sui_lint::rules::sx::lower` vocabulary — the
/// author writes the closed sx grammar, never CSS. The block is stripped from the
/// template (the parser never sees it); its CSS joins the injected `<style>`.
fn lower_variants(source: &str) -> (String, String) {
    let mut out = source.to_string();
    let mut css = String::new();
    // Multiple blocks: a bare `<variants>` (styles `:host`) and any number of
    // `<variants part="field">` (Panda slot recipe — styles a shadow part).
    loop {
        let Some(start) = out.find("<variants") else { break };
        let Some(tag_rel) = out[start..].find('>') else { break };
        let tag_end = start + tag_rel + 1;
        let open_tag = out[start..tag_end].to_string();
        let Some(rel) = out[tag_end..].find("</variants>") else { break };
        let body = out[tag_end..tag_end + rel].to_string();
        let block_end = tag_end + rel + "</variants>".len();

        // `part="field"` re-targets the rule from `:host` to `[part='field']`.
        let part = attr_value(&open_tag, "part");

        // Two body forms, ONE emission. A body starting with `{` is the
        // TS-object form (CVA-shaped: `variant: { solid: 'sx…' }` + `compound`);
        // otherwise it's the legacy `selector: sx` line DSL. Both lower to the
        // same `(selector-dsl, sx)` pairs the existing helpers consume.
        let pairs: Vec<(String, String)> = if body.trim_start().starts_with('{') {
            parse_variants_object(&body)
        } else {
            body.lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() {
                        return None;
                    }
                    let (sel, sx) = line.split_once(':')?;
                    Some((sel.trim().to_string(), sx.trim().to_string()))
                })
                .collect()
        };
        for (sel, sx) in pairs {
            let Some(selector) = variant_selector(&sel) else { continue };
            let decls = lower_variant_sx(&sx);
            if decls.is_empty() {
                continue;
            }
            match &part {
                Some(p) => css.push_str(&format!(":host({selector}) [part='{p}']{{{decls}}}")),
                None => css.push_str(&format!(":host({selector}){{{decls}}}")),
            }
        }

        let mut next = String::with_capacity(out.len());
        next.push_str(&out[..start]);
        next.push_str(&out[block_end..]);
        out = next;
    }
    (out, css)
}

/// Split `s` on top-level `delim`, ignoring delimiters nested inside `{}`/`[]`
/// or string literals. The primitive under the `<variants>` object parser.
fn split_top_level(s: &str, delim: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    let mut cur = String::new();
    for c in s.chars() {
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
pub(crate) fn parse_object_fields(obj: &str) -> Vec<(String, String)> {
    let inner = obj
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    split_top_level(inner, ',')
        .into_iter()
        .filter_map(|entry| {
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
fn wrap_controller_body(body: &str) -> String {
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
        "import {{ controller }} from '@ship/sui-runtime';\n{}\nexport default controller({{ scope: () => ({{ {} }}) }});\n",
        lines.join("\n"),
        names.join(", ")
    )
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
    let Some(start) = source.find("<controller") else {
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

fn lower_component(source: &str) -> String {
    let Some(start) = source.find("<component") else {
        return source.to_string();
    };
    let Some(open_rel) = source[start..].find('>') else {
        return source.to_string();
    };
    let body_start = start + open_rel + 1;
    let Some(rel) = source[body_start..].find("</component>") else {
        return source.to_string();
    };
    let body_end = body_start + rel;
    let block_end = body_end + "</component>".len();

    let fields = parse_object_fields(&source[body_start..body_end]);
    // `form: { value: 'checked', disabled: 'disabled' }` — names the props that
    // carry ElementInternals form association. Parsed BEFORE the props expansion
    // so the named props' tags gain `form-value` / `form-disabled` flags
    // regardless of field order in the block.
    let (form_value, form_disabled) = fields
        .iter()
        .find(|(k, _)| k == "form")
        .map(|(_, v)| parse_form_field(v))
        .unwrap_or((None, None));
    let mut generated = String::new();
    for (key, val) in fields {
        match key.as_str() {
            "props" => generated.push_str(&expand_props(
                &val,
                form_value.as_deref(),
                form_disabled.as_deref(),
            )),
            "variants" => generated.push_str(&format!("<variants>{val}</variants>\n")),
            // `form` was consumed above; meta.tag is filename-derived today.
            _ => {}
        }
    }
    let mut out = String::with_capacity(source.len());
    out.push_str(&source[..start]);
    out.push_str(&generated);
    out.push_str(&source[block_end..]);
    out
}

/// Parse the `form: { value: '…', disabled: '…' }` object into the two prop
/// names. Unknown keys are ignored (tsgo owns shape validation of the block).
fn parse_form_field(obj: &str) -> (Option<String>, Option<String>) {
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

/// Expand a props object `{ tone, size, variant: ['solid','soft'], disabled: bool }`
/// into `<prop …/>` tags. `form_value` / `form_disabled` name the props that
/// gain the `form-value` / `form-disabled` association flags (from the
/// component block's `form:` field).
fn expand_props(obj: &str, form_value: Option<&str>, form_disabled: Option<&str>) -> String {
    let mut out = String::new();
    for (name, val) in parse_object_fields(obj) {
        let n = name.as_str();
        let val = val.trim();
        // Built-in framework props: `tone`/`size` — bare or annotated `: Tone`/
        // `: Size`. Naming them IS the contract opt-in (channels come for free).
        let tag = if n == "tone" {
            "<prop name=\"tone\" values=\"default|primary|info|success|warning|destructive|accent|muted\" default=\"default\" reflect />".to_string()
        } else if n == "size" {
            "<prop name=\"size\" values=\"xs|sm|md|lg\" default=\"md\" reflect />".to_string()
        } else if val.contains('|') {
            // Real-TS union: `variant: 'default' | 'solid' | 'outline'`. The
            // default is the first member — order the union with it first.
            let vals: Vec<String> = val
                .split('|')
                .map(|s| unquote(s.trim()))
                .filter(|s| !s.is_empty())
                .collect();
            let default = vals.first().cloned().unwrap_or_default();
            format!(
                "<prop name=\"{n}\" values=\"{}\" default=\"{default}\" reflect />",
                vals.join("|")
            )
        } else if val.starts_with('[') {
            // Transitional array form: `['solid','soft']`.
            let inner = val.trim_start_matches('[').trim_end_matches(']');
            let vals: Vec<String> = split_top_level(inner, ',')
                .into_iter()
                .map(|s| unquote(&s))
                .collect();
            let default = vals.first().cloned().unwrap_or_default();
            format!(
                "<prop name=\"{n}\" values=\"{}\" default=\"{default}\" reflect />",
                vals.join("|")
            )
        } else if val == "boolean" || val == "bool" {
            format!("<prop name=\"{n}\" type=\"bool\" default=\"false\" reflect />")
        } else if val == "number" {
            format!("<prop name=\"{n}\" type=\"number\" default=\"0\" />")
        } else {
            // `string`, `String`, or a bare shorthand → a string prop.
            format!("<prop name=\"{n}\" type=\"string\" default=\"\" />")
        };
        // Form association: the prop named by `form.value` syncs
        // ElementInternals.setFormValue; `form.disabled` receives the
        // fieldset-disabled callback. Flags append to whatever tag shape the
        // prop expanded to.
        let mut tag = tag;
        if form_value == Some(n) {
            tag = tag.replace(" />", " form-value />");
        }
        if form_disabled == Some(n) {
            tag = tag.replace(" />", " form-disabled />");
        }
        out.push_str(&tag);
        out.push('\n');
    }
    out
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
fn lower_theme(source: &str) -> (String, String) {
    let mut out = source.to_string();
    let mut css = String::new();
    // Multiple blocks (e.g. a `tone` theme and a `size` theme).
    loop {
        let Some(start) = out.find("<theme") else { break };
        let Some(tag_rel) = out[start..].find('>') else { break };
        let tag_end = start + tag_rel + 1;
        let open_tag = out[start..tag_end].to_string();
        let Some(close_rel) = out[tag_end..].find("</theme>") else { break };
        let body = out[tag_end..tag_end + close_rel].to_string();
        let block_end = tag_end + close_rel + "</theme>".len();

        let prop = attr_value(&open_tag, "prop");
        let channels: Vec<String> = attr_value(&open_tag, "channels")
            .map(|c| c.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();

        if let Some(prop) = prop {
            for line in body.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let Some((value, fills_part)) = line.split_once(':') else { continue };
                let value = value.trim();
                let mut decls = String::new();
                for (chan, fill) in channels.iter().zip(fills_part.split(',').map(str::trim)) {
                    if fill.is_empty() {
                        continue;
                    }
                    decls.push_str(&format!("--{chan}:var(--{fill});"));
                }
                if decls.is_empty() {
                    continue;
                }
                if value == "default" {
                    css.push_str(&format!(":host{{{decls}}}"));
                } else {
                    css.push_str(&format!(":host([{prop}='{value}']){{{decls}}}"));
                }
            }
        }
        let mut next = String::with_capacity(out.len());
        next.push_str(&out[..start]);
        next.push_str(&out[block_end..]);
        out = next;
    }
    (out, css)
}

/// Lower `<host sx="…">` — an element-less directive that styles the component
/// host itself. Its sx lowers to a `:host { … }` rule (not a class), then the tag
/// is removed. Closes the ubiquitous bare-`:host` base-styling residual (display,
/// vertical-align via `align`, surface props) that no other construct reaches —
/// the host is not a real element, so it cannot carry a class.
fn lower_host(source: &str) -> (String, String) {
    let mut out = source.to_string();
    let mut css = String::new();
    let mut from = 0;
    loop {
        let Some(rel) = out[from..].find("<host") else { break };
        let start = from + rel;
        // Must be a real `<host>` tag: the char after the name ends it.
        let after = out.as_bytes().get(start + 5).copied();
        if !matches!(after, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace()) {
            from = start + 5; // e.g. `<hostname>` — skip, keep searching
            continue;
        }
        let Some(tag_rel) = out[start..].find('>') else { break };
        let mut tag_end = start + tag_rel + 1;
        let open_tag = out[start..tag_end].to_string();
        // Consume a matching `</host>` for the non-self-closing form.
        if !open_tag.trim_end().ends_with("/>") {
            if let Some(close_rel) = out[tag_end..].find("</host>") {
                tag_end += close_rel + "</host>".len();
            }
        }
        if let Some(sx) = attr_value(&open_tag, "sx") {
            let decls = lower_variant_sx(&sx);
            if !decls.is_empty() {
                css.push_str(&format!(":host{{{decls}}}"));
            }
        }
        let mut next = String::with_capacity(out.len());
        next.push_str(&out[..start]);
        next.push_str(&out[tag_end..]);
        out = next;
        from = start;
    }
    (out, css)
}

/// Extract `name="value"` from an opening tag. `None` if absent.
fn attr_value(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let rel = tag[start..].find('"')?;
    Some(tag[start..start + rel].to_string())
}

/// A variant selector → a `:host(...)` attribute-match. `tone=success` →
/// `[tone='success']`; `[disabled]` → `[disabled]`; `(a=x, b=y)` →
/// `[a='x'][b='y']` (compound).
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
        Some(format!("[{}='{}']", prop.trim(), val.trim()))
    } else if !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
        // A bare identifier is a boolean attribute: `invalid` → `[invalid]`.
        Some(format!("[{s}]"))
    } else {
        None
    }
}

/// Lower a space-separated sx list to `prop:value;…` via the shared sx vocab.
fn lower_variant_sx(list: &str) -> String {
    let mut out = String::new();
    for pair in list.split_whitespace() {
        if let Some((prop, tok)) = pair.split_once(':')
            && let Some(decls) = sui_lint::rules::sx::lower(prop, tok)
        {
            for (p, v) in decls {
                out.push_str(&format!("{p}:{v};"));
            }
        }
    }
    out
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
        let out = preprocess(r#"<span class="badge" part="badge" sx="p:4">x</span>"#);
        assert!(
            out.contains(r#"class="badge sx-p-4" part="badge""#),
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
    fn component_form_field_flags_the_named_props() {
        // `form: { value, disabled }` → the named props' tags gain the
        // ElementInternals association flags, independent of field order.
        let src = "<component>{\n  props: { checked: bool, disabled: bool, name: string },\n  form: { value: 'checked', disabled: 'disabled' },\n}</component>\n<div>x</div>";
        let out = preprocess(src);
        assert!(
            out.contains(r#"<prop name="checked" type="bool" default="false" reflect form-value />"#),
            "checked carries form-value, got:\n{out}"
        );
        assert!(
            out.contains(r#"<prop name="disabled" type="bool" default="false" reflect form-disabled />"#),
            "disabled carries form-disabled, got:\n{out}"
        );
        assert!(
            out.contains(r#"<prop name="name" type="string" default="" />"#),
            "unnamed props unchanged, got:\n{out}"
        );
        assert!(!out.contains("<form"), "the form field itself emits nothing:\n{out}");
    }

    #[test]
    fn component_form_field_before_props_also_flags() {
        let src = "<component>{\n  form: { value: 'value' },\n  props: { value: string },\n}</component>\n<div>x</div>";
        let out = preprocess(src);
        assert!(
            out.contains(r#"<prop name="value" type="string" default="" form-value />"#),
            "form before props still flags, got:\n{out}"
        );
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
    fn host_directive_lowers_to_host_rule() {
        let out = preprocess(r#"<host sx="display:inline-flex p:2" /><button>x</button>"#);
        assert!(out.contains(":host{display:inline-flex;padding:var(--space-2);}"), "{out}");
        assert!(!out.contains("<host"), "tag removed: {out}");
    }

    #[test]
    fn theme_block_lowers_channels_and_sx_consumes_them() {
        let out = preprocess(
            "<theme prop=\"tone\" channels=\"tone-fg, tone-border\">\n  default: foreground, border\n  info: info, tone-info-border\n</theme>\n<div sx=\"fg:tone-fg border:tone-border\">x</div>",
        );
        assert!(!out.contains("<theme"), "block stripped: {out}");
        assert!(
            out.contains(":host{--tone-fg:var(--foreground);--tone-border:var(--border);}"),
            "default channel row: {out}"
        );
        assert!(
            out.contains(":host([tone='info']){--tone-fg:var(--info);--tone-border:var(--tone-info-border);}"),
            "info channel row: {out}"
        );
        assert!(out.contains(".sx-fg-tonefg{color:var(--tone-fg)}"), "channel as color: {out}");
        assert!(
            out.contains(".sx-border-toneborder{border:1px solid var(--tone-border)}"),
            "channel as full border: {out}"
        );
    }

    #[test]
    fn variants_part_attr_targets_a_slot() {
        let out = preprocess("<variants part=\"field\">\n  invalid: border:destructive\n  disabled: cursor:not-allowed\n</variants>\n<div part=\"field\">x</div>");
        assert!(out.contains(":host([invalid]) [part='field']{border:1px solid var(--destructive);}"), "slot variant: {out}");
        assert!(out.contains(":host([disabled]) [part='field']{cursor:not-allowed;}"), "bare boolean → slot: {out}");
    }

    #[test]
    fn variants_object_form_matches_line_dsl() {
        // The CVA-shaped TS-object body lowers to the same CSS as the line DSL:
        // prop groups → `[prop='value']`, `compound` → AND-ed conditions.
        let obj = preprocess(
            "<variants part=\"button\">{\n  variant: { solid: 'bg:primary fg:background', outline: 'border:strong fg:foreground' },\n  size: { sm: 'min-h:control-h-sm', lg: 'min-h:control-h-lg' },\n  compound: [ { when: { variant: 'solid', size: 'lg' }, sx: 'shadow:md' } ],\n}</variants>\n<button part=\"button\">x</button>",
        );
        assert!(!obj.contains("<variants"), "object block stripped: {obj}");
        assert!(
            obj.contains(":host([variant='solid']) [part='button']{background:var(--primary);color:var(--background);}"),
            "prop-group variant from object form: {obj}"
        );
        assert!(
            obj.contains(":host([size='lg']) [part='button']{min-height:var(--control-h-lg);}"),
            "prop-group size from object form: {obj}"
        );
        assert!(
            obj.contains(":host([variant='solid'][size='lg']) [part='button']{box-shadow:var(--shadow-md);}"),
            "compound AND-ed selector from object form: {obj}"
        );
    }

    #[test]
    fn variants_block_lowers_to_host_css() {
        let out = preprocess(
            "<variants>\n  tone=success: bg:tone-success-bg fg:tone-success-fg\n  [disabled]: opacity:50 cursor:not-allowed\n  (tone=primary, size=lg): shadow:md\n</variants>\n<button>x</button>",
        );
        assert!(!out.contains("<variants>"), "block stripped from template: {out}");
        assert!(
            out.contains(":host([tone='success']){background:var(--tone-success-bg);color:var(--tone-success-fg);}"),
            "attr variant: {out}"
        );
        assert!(out.contains(":host([disabled]){opacity:0.5;cursor:not-allowed;}"), "boolean variant: {out}");
        assert!(
            out.contains(":host([tone='primary'][size='lg']){box-shadow:var(--shadow-md);}"),
            "compound variant: {out}"
        );
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
