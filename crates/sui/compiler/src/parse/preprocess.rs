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
    let (s, sx_css) = lower_sx(&s);
    let (s, layout_css) = lower_layout(&s);
    let (s, variants_css) = lower_variants(&s);
    let (s, theme_css) = lower_theme(&s);
    let css = format!("{theme_css}{sx_css}{layout_css}{variants_css}");
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
    // Channels declared by this file's `<theme>` blocks — legal sx values that
    // lower to `var(--channel)` (the theme fills them per prop value).
    let channels = sui_lint::rules::sx::source_channels(source);
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
                    out.push_str(&lower_sx_value(val, &mut css, &mut seen, &channels));
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
    channels: &[String],
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
        let (prefix, pseudo): (String, &str) = match state {
            None => (String::new(), ""),
            Some(s) => {
                if let Some(base) = s.strip_prefix("group-") {
                    match sui_lint::rules::sx::state_pseudo(base) {
                        Some(p) => (format!(".sui-grp{p} "), ""),
                        None => continue,
                    }
                } else {
                    match sui_lint::rules::sx::state_pseudo(s) {
                        Some(p) => (String::new(), p),
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
        // A declared `<theme>` channel lowers to `var(--channel)` on the prop's
        // CSS properties (the theme fills the channel per prop value); otherwise
        // the normal token vocabulary.
        let decls = sui_lint::rules::sx::lower(prop, token).or_else(|| {
            channels
                .iter()
                .any(|c| c == token)
                .then(|| sui_lint::rules::sx::lower_channel(prop, token))
                .flatten()
        });
        if let Some(decls) = decls {
            if seen.insert(class.clone()) {
                let body = decls
                    .iter()
                    .map(|(p, v)| format!("{p}:{v}"))
                    .collect::<Vec<_>>()
                    .join(";");
                css.push_str(&format!("{prefix}.{class}{pseudo}{{{body}}}"));
            }
        }
        classes.push(class);
    }
    classes.join(" ")
}

/// After the `sx` name: optional ws, `=`, optional ws, then a `"`.
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

        for line in body.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some((sel_part, sx_part)) = line.split_once(':') else { continue };
            let Some(selector) = variant_selector(sel_part) else { continue };
            let decls = lower_variant_sx(sx_part);
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
