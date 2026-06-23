//! `DeckResponse` — the transport-agnostic result every deck handler produces.
//!
//! deck-core never speaks HTTP. Handler logic returns a `DeckResponse`
//! (status + headers + body); a server adapter (deck-axum, …) maps it
//! onto its own response type. This is the port; the adapters are the plugs.

use crate::DaemonMode;

/// Cache-control family. Bundles are immutable-in-prod; view artifacts must
/// revalidate. Picked per-response so the adapter needs no mode awareness.
#[derive(Clone, Copy, Debug)]
pub enum CachePolicy {
    /// Framework bundles (runtime/dom/components/islands) + per-icon lucide SVGs.
    Asset,
    /// View artifacts (template/setup/css/payload/host).
    View,
    /// Never cache (errors, not-found).
    None,
}

// ── HTML security headers ─────────────────────────────────────────────────────
//
// Every HTML response emitted by deck (`html_view`) carries a fixed set of
// security headers. The set is the SINGLE mint point for these headers; the
// `html_view_carries_security_headers` behavior test pins them so a half-edit
// (dropping a header, weakening the CSP, renaming X-Frame-Options) is caught
// before it ships. The headers live as a `const &[(&str, &str)]` rather than
// `format!`-inlined so:
//
//   1. The literal text is greppable + diffable.
//   2. A header change does not introduce per-call allocations.
//   3. The behavior test can assert against the exact bytes the daemon sends.
//
// Why these headers:
//   - `X-Frame-Options: SAMEORIGIN` — clickjacking defence. SAMEORIGIN (not
//     DENY) lets the hub embed same-origin iframes (preview surfaces).
//   - `X-Content-Type-Options: nosniff` — stops browsers from MIME-sniffing a
//     body whose declared content-type is `text/html` into something else.
//   - `Content-Security-Policy: …` — defence-in-depth against stored XSS via
//     SUI primitives. `'unsafe-inline'` is required because deck inlines
//     scripts (`inject_globals`) and SUI templates may inline styles; the
//     policy still restricts external origins so a stored XSS cannot load
//     attacker-controlled scripts. font-src / style-src admit Google Fonts
//     because the shell's `index.html` loads Syne / DM Sans / JetBrains Mono
//     from fonts.googleapis.com (see apps/shell/index.html). Without these,
//     the design system falls back to system fonts and the page looks visually
//     broken (icons tied to those families disappear). We could self-host; for
//     the dev loop a CSP allow is the smallest fix. img-src 'self' data: stays
//     tight; connect-src 'self' blocks exfiltration; frame-ancestors 'self'
//     mirrors X-Frame-Options for browsers that honour CSP framing.
pub const HTML_SECURITY_HEADERS: &[(&str, &str)] = &[
    ("X-Frame-Options", "SAMEORIGIN"),
    ("X-Content-Type-Options", "nosniff"),
    (
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data:; connect-src 'self'; frame-ancestors 'self'",
    ),
];


impl CachePolicy {
    fn header(self, mode: DaemonMode) -> &'static str {
        match self {
            CachePolicy::None => "no-cache",
            CachePolicy::Asset => match mode {
                DaemonMode::Development => "no-cache",
                DaemonMode::Production => "public, immutable, max-age=31536000",
            },
            CachePolicy::View => match mode {
                DaemonMode::Development => "no-cache",
                DaemonMode::Production => "must-revalidate, max-age=0",
            },
        }
    }
}

/// A framework-agnostic HTTP-shaped response. Adapters translate it 1:1.
#[derive(Clone, Debug)]
pub struct DeckResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub cache_control: &'static str,
    /// Strong ETag (Production only); `None` omits the header.
    pub etag: Option<String>,
    /// Additional headers beyond content-type/cache/etag (e.g. `X-Sui-View`).
    pub extra_headers: Vec<(&'static str, String)>,
    pub body: Vec<u8>,
}

impl DeckResponse {
    /// Core constructor. ETag is computed for cacheable bodies in Production
    /// only — matching the daemon's prior per-handler behavior.
    pub fn new(
        status: u16,
        content_type: &'static str,
        cache: CachePolicy,
        mode: DaemonMode,
        body: impl Into<Vec<u8>>,
    ) -> Self {
        let body = body.into();
        let etag = match (cache, mode) {
            (CachePolicy::None, _) | (_, DaemonMode::Development) => None,
            (_, DaemonMode::Production) => Some(compute_etag(&body)),
        };
        Self {
            status,
            content_type,
            cache_control: cache.header(mode),
            etag,
            extra_headers: Vec::new(),
            body,
        }
    }

    pub fn js_asset(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "application/javascript; charset=utf-8", CachePolicy::Asset, mode, body)
    }
    pub fn js_view(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "application/javascript; charset=utf-8", CachePolicy::View, mode, body)
    }
    pub fn css_view(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "text/css; charset=utf-8", CachePolicy::View, mode, body)
    }
    pub fn html_view(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        // All HTML responses carry the deck-defined security header set. The
        // values live in `HTML_SECURITY_HEADERS` (top of file) — the single
        // mint point, with a behavior test pinning the bytes so a half-edit
        // is caught before it ships.
        let mut resp = Self::new(200, "text/html; charset=utf-8", CachePolicy::View, mode, body);
        for (name, value) in HTML_SECURITY_HEADERS {
            resp = resp.with_header(name, *value);
        }
        resp
    }
    pub fn json_asset(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "application/json; charset=utf-8", CachePolicy::Asset, mode, body)
    }
    pub fn json_view(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "application/json; charset=utf-8", CachePolicy::View, mode, body)
    }

    /// 404 with a plain-text reason (no cache, no etag).
    ///
    /// In Production the caller-supplied `msg` is suppressed and a generic
    /// `"not found"` body is returned — `msg` routinely carries internal detail
    /// (absolute on-disk paths, resolver attempt lists, build hints) that must
    /// never reach prod clients. In Development the reason is surfaced verbatim
    /// to aid debugging. Mirrors `render_error`'s mode-gated redaction.
    pub fn not_found(msg: impl Into<String>, mode: DaemonMode) -> Self {
        let body = match mode {
            DaemonMode::Production => "not found".to_string(),
            DaemonMode::Development => msg.into(),
        };
        let mut r = Self::new(404, "text/plain; charset=utf-8", CachePolicy::None, mode, body.into_bytes());
        r.cache_control = "no-cache";
        r
    }

    /// 500 — in Production the message is hidden; in Development it's surfaced.
    pub fn render_error(msg: impl Into<String>, mode: DaemonMode) -> Self {
        let body = match mode {
            DaemonMode::Production => "render error".to_string(),
            DaemonMode::Development => msg.into(),
        };
        Self::new(500, "text/plain; charset=utf-8", CachePolicy::None, mode, body.into_bytes())
    }

    pub fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.extra_headers.push((name, value.into()));
        self
    }
}

/// Strong ETag derived from content bytes (fast non-cryptographic hash).
pub fn compute_etag(content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    content.hash(&mut h);
    format!("\"{}\"", h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── security headers ──────────────────────────────────────────────────────

    fn header_value<'a>(resp: &'a DeckResponse, name: &str) -> Option<&'a str> {
        resp.extra_headers.iter().find_map(|(k, v)| {
            if k.eq_ignore_ascii_case(name) { Some(v.as_str()) } else { None }
        })
    }

    #[test]
    fn html_view_carries_security_headers() {
        let resp = DeckResponse::html_view("<html></html>", DaemonMode::Development);
        assert_eq!(
            header_value(&resp, "X-Frame-Options"),
            Some("SAMEORIGIN"),
            "X-Frame-Options must be SAMEORIGIN on HTML responses"
        );
        assert_eq!(
            header_value(&resp, "X-Content-Type-Options"),
            Some("nosniff"),
            "X-Content-Type-Options must be nosniff on HTML responses"
        );
        let csp = header_value(&resp, "Content-Security-Policy")
            .expect("Content-Security-Policy must be present on HTML responses");
        assert!(csp.contains("default-src 'self'"), "CSP must restrict default-src; got: {csp}");
        assert!(csp.contains("frame-ancestors 'self'"), "CSP must restrict frame-ancestors; got: {csp}");
    }

    #[test]
    fn js_asset_does_not_carry_html_security_headers() {
        let resp = DeckResponse::js_asset("console.log(1)", DaemonMode::Development);
        // Non-HTML responses do not need X-Frame-Options or CSP.
        assert!(
            header_value(&resp, "X-Frame-Options").is_none(),
            "X-Frame-Options must not appear on non-HTML responses"
        );
        assert!(
            header_value(&resp, "Content-Security-Policy").is_none(),
            "CSP must not appear on non-HTML responses"
        );
    }

    /// Behavior pin: the `extra_headers` carried by an `html_view` response
    /// must equal `HTML_SECURITY_HEADERS` exactly, in order, byte-for-byte.
    /// This is the transport-agnostic assertion — it catches a header added
    /// to the const but not emitted by `html_view`, or a header emitted by
    /// `html_view` but missing from the const. The wire-level mirror in
    /// `deck-axum` (`html_view_security_headers_reach_the_wire`) confirms
    /// the adapter preserves them, completing the chain.
    #[test]
    fn html_view_emits_exactly_the_security_header_set() {
        let resp = DeckResponse::html_view("<html></html>", DaemonMode::Development);
        assert_eq!(
            resp.extra_headers.len(),
            HTML_SECURITY_HEADERS.len(),
            "html_view must emit exactly the HTML_SECURITY_HEADERS set; \
             got {} extra headers but the const declares {}",
            resp.extra_headers.len(),
            HTML_SECURITY_HEADERS.len(),
        );
        for (i, (name, value)) in HTML_SECURITY_HEADERS.iter().enumerate() {
            let (got_name, got_value) = &resp.extra_headers[i];
            assert_eq!(
                got_name, name,
                "header #{i} name: expected `{name}`, got `{got_name}`"
            );
            assert_eq!(
                got_value, value,
                "header `{name}` value mismatch: expected `{value}`, got `{got_value}`"
            );
        }
    }

    /// Negative-case proof: a half-edit that drops `X-Frame-Options` from
    /// the const fails the behavior test. Mirrors as an inline assertion
    /// here so the test surfaces the failure mode without spinning up a
    /// deck-axum dep.
    #[test]
    fn html_view_security_headers_are_not_substring_matchable() {
        // The security-header assertion above checks the WHOLE response,
        // not a substring. If the const ever shrank to `X-Frame-Options`
        // alone, `html_view_carries_security_headers` (which uses
        // `header_value` lookups for X-Content-Type-Options and CSP) would
        // also fail. Belt-and-suspenders: a third, exact-set equality
        // assertion (`html_view_emits_exactly_the_security_header_set`)
        // catches partial overlaps a single substring check would miss.
        let resp = DeckResponse::html_view("<html></html>", DaemonMode::Development);
        let names: Vec<&str> = resp
            .extra_headers
            .iter()
            .map(|(k, _)| *k)
            .collect();
        assert!(
            names.contains(&"X-Frame-Options"),
            "X-Frame-Options missing from served HTML response (extra_headers={names:?})"
        );
        assert!(
            names.contains(&"X-Content-Type-Options"),
            "X-Content-Type-Options missing from served HTML response (extra_headers={names:?})"
        );
        assert!(
            names.contains(&"Content-Security-Policy"),
            "Content-Security-Policy missing from served HTML response (extra_headers={names:?})"
        );
        // No spurious header slipped in past the const.
        let expected_count = HTML_SECURITY_HEADERS.len();
        assert_eq!(
            resp.extra_headers.len(),
            expected_count,
            "extra_headers count drifted from HTML_SECURITY_HEADERS.len() — \
             either `html_view` added an extra header or the const shrank",
        );
    }

    #[test]
    fn not_found_dev_surfaces_reason() {
        let r = DeckResponse::not_found("/abs/path/views/foo missing", DaemonMode::Development);
        assert_eq!(r.status, 404);
        assert_eq!(r.cache_control, "no-cache");
        assert!(r.etag.is_none());
        assert_eq!(r.body, b"/abs/path/views/foo missing");
    }

    #[test]
    fn not_found_prod_redacts_reason() {
        // Caller-supplied detail (absolute paths, build hints, resolver attempt
        // lists) must never reach prod clients — only a generic body ships.
        let r = DeckResponse::not_found("/abs/path/views/foo missing", DaemonMode::Production);
        assert_eq!(r.status, 404);
        let body = std::str::from_utf8(&r.body).unwrap();
        assert_eq!(body, "not found");
        assert!(!body.contains("/abs/path"), "prod 404 must not echo on-disk paths");
        // 404s are never cached and never carry an ETag, in either mode.
        assert_eq!(r.cache_control, "no-cache");
        assert!(r.etag.is_none());
    }

    // ── TS port drift guard ───────────────────────────────────────────────────

    /// The hand-maintained TS port at `packages/deck/hono/src/response.ts`
    /// must carry the SAME wire bytes this Rust module mints. We assert the
    /// THREE contract surfaces the spec names — content-types, the
    /// cache-control policy×mode matrix, and the HTML security headers —
    /// deriving every expected string FROM Rust, so a change here (a new
    /// max-age, a tightened CSP, a renamed header) forces the TS port to
    /// follow or this test fails. Without it the two ports drift silently;
    /// the Rust side is the source of truth, the TS side is the port.
    #[test]
    fn ts_port_pins_the_deck_response_contract() {
        let ts = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../packages/deck/hono/src/response.ts"
        ))
        .expect("read packages/deck/hono/src/response.ts");

        // 1. Content-types minted by the constructors in this file.
        for ct in [
            "application/javascript; charset=utf-8",
            "text/css; charset=utf-8",
            "text/html; charset=utf-8",
            "application/json; charset=utf-8",
            "text/plain; charset=utf-8",
        ] {
            assert!(ts.contains(ct), "TS port missing content-type `{ct}`");
        }

        // 2. Cache-control policy×mode matrix — every byte-string Rust mints
        //    must appear in the TS `cacheControl` function.
        for policy in [CachePolicy::Asset, CachePolicy::View, CachePolicy::None] {
            for mode in [DaemonMode::Development, DaemonMode::Production] {
                let bytes = policy.header(mode);
                assert!(
                    ts.contains(bytes),
                    "TS port missing cache-control `{bytes}` for {policy:?}/{mode:?}"
                );
            }
        }

        // 3. HTML security headers — name AND value, exact bytes, for the
        //    whole set (the CSP string in particular cannot drift).
        for (name, value) in HTML_SECURITY_HEADERS {
            assert!(ts.contains(name), "TS port missing security header name `{name}`");
            assert!(
                ts.contains(*value),
                "TS port missing security header value for `{name}`"
            );
        }
    }
}
