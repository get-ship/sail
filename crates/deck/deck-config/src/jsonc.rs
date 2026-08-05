//! Std-only JSONC stripper for the deck-config crate.
//!
//! A self-contained concern: turn a JSONC document (`//` + `/* */` comments,
//! trailing commas) into valid JSON, without depending on `ship-substrate`.
//! Kept in its own module so the config types stay focused on schema.

/// Strip JSONC extensions from a string: `//` line comments, `/* */` block
/// comments, and trailing commas before `]` or `}`. The result is valid JSON.
/// Respects string literals so URLs like `"http://"` are not mangled.
///
/// Public helper for hosts (and tests) that need to parse a JSONC document
/// without depending on `ship-substrate` — the deck-config crate is the
/// std-only JSONC parser the deck-core resolver chain requires. The
/// `deck-config::DeckConfig` type no longer reads any file itself; the
/// helper stays so a host that DOES want JSONC parsing for its own
/// config (e.g. `apps.jsonc`) can build a `DeckConfig` via
/// `serde_json::from_str(&strip_jsonc_comments(&raw))` and pass it in. The
/// per-app serve parser (`AppServeConfig::parse_deck_jsonc`) calls it
/// internally — one stripper, no second copy.
///
/// NOTE: this is a deliberate second copy of the JSONC stripper inlined
/// into the deck-config crate to drop the `ship-substrate` dependency
/// edge from deck-core's resolver chain. The two copies must stay in
/// sync — if the algorithm changes, update both.
pub fn strip_jsonc_comments(input: &str) -> String {
    // Phase 1: strip comments.
    let mut stage = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;
    while let Some(c) = chars.next() {
        if in_string {
            stage.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            stage.push(c);
            continue;
        }
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    while let Some(&nc) = chars.peek() {
                        chars.next();
                        if nc == '\n' {
                            stage.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                    continue;
                }
                _ => {}
            }
        }
        stage.push(c);
    }
    // Phase 2: remove trailing commas (`,` followed only by whitespace then
    // `]` or `}`). Walk as bytes for cheap peek-ahead, but copy non-ASCII
    // bytes verbatim — `bytes[i] as char` would mangle multi-byte UTF-8
    // (e.g. an emdash 0xE2 0x80 0x94 turns into three U+00E2/U+0080/U+0094
    // chars when push'd to a String).
    let src = stage.as_bytes();
    let mut cleaned: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    let mut in_string2 = false;
    let mut escape2 = false;
    while i < src.len() {
        let b = src[i];
        if in_string2 {
            cleaned.push(b);
            if escape2 {
                escape2 = false;
            } else if b == b'\\' {
                escape2 = true;
            } else if b == b'"' {
                in_string2 = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string2 = true;
            cleaned.push(b);
            i += 1;
            continue;
        }
        if b == b',' {
            let mut j = i + 1;
            while j < src.len()
                && (src[j] == b' ' || src[j] == b'\t' || src[j] == b'\r' || src[j] == b'\n')
            {
                j += 1;
            }
            if j < src.len() && (src[j] == b']' || src[j] == b'}') {
                i += 1;
                continue;
            }
        }
        cleaned.push(b);
        i += 1;
    }
    // Safe: we only dropped ASCII commas at valid char boundaries; every
    // other byte was copied verbatim from the original UTF-8 String.
    String::from_utf8(cleaned)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}
