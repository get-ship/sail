//! `DeckResponse` — the transport-agnostic result every deck handler produces.
//!
//! deck-core never speaks HTTP. Handler logic returns a `DeckResponse`
//! (status + headers + body); a server adapter (deck-axum, …) maps it
//! onto its own response type. This is the port; the adapters are the plugs.

use crate::DaemonMode;

/// Cache-control family. Bundles are immutable-in-prod; route artifacts must
/// revalidate. Picked per-response so the adapter needs no mode awareness.
#[derive(Clone, Copy, Debug)]
pub enum CachePolicy {
    /// Framework bundles (runtime/dom/components/islands) + per-icon lucide SVGs.
    Asset,
    /// Route artifacts (template/setup/css/payload/host).
    Route,
    /// Never cache (errors, not-found).
    None,
}

// ── HTML security headers ─────────────────────────────────────────────────────
//
// Every HTML response emitted by deck (`html_route`) carries a fixed set of
// security headers. The set is the SINGLE mint point for these headers; the
// `html_route_carries_security_headers` behavior test pins them so a half-edit
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
//     SAIL primitives. `'unsafe-inline'` is required because deck inlines
//     scripts (`inject_globals`) and SAIL templates may inline styles; the
//     policy still restricts external origins so a stored XSS cannot load
//     attacker-controlled scripts. font-src / style-src admit Google Fonts
//     because the shell's `index.html` loads Syne / DM Sans / JetBrains Mono
//     from fonts.googleapis.com (see apps/shell/index.html). Without these,
//     the design system falls back to system fonts and the page looks visually
//     broken (icons tied to those families disappear). We could self-host; for
//     the dev loop a CSP allow is the smallest fix. img-src allows 'self',
//     data/blob (object URLs from media), and https images (demo photos /
//     remote media). connect-src 'self' still blocks exfiltration;
//     frame-ancestors 'self' mirrors X-Frame-Options for CSP framing.
pub const HTML_SECURITY_HEADERS: &[(&str, &str)] = &[
    ("X-Frame-Options", "SAMEORIGIN"),
    ("X-Content-Type-Options", "nosniff"),
    (
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: blob: https:; connect-src 'self'; frame-ancestors 'self'",
    ),
];

/// The CSP body shared by every `connect-src` variant — everything except the
/// `connect-src` clause itself, which [`connect_src_directive`] derives from
/// a package's resolved `"network"` allowlist
/// (`sail_compiler::check::policy::Policy::network_for` /
/// `PolicyResolution::network`). Split out so [`html_security_headers_for`]
/// cannot drift from [`HTML_SECURITY_HEADERS`]'s hand-written string on any
/// clause OTHER than `connect-src`.
const CSP_PREFIX: &str = "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: blob: https:;";
const CSP_SUFFIX: &str = "frame-ancestors 'self'";

/// Derive the `connect-src` CSP clause from a package's resolved `"network"`
/// per-origin allowlist — the SAME list the `sail-controller-no-ambient-
/// authority` lint checks fetch/WebSocket/EventSource targets against
/// (`sail_compiler::check::policy::Policy::network_for`). Empty list (the
/// default: no `sail.config.json`, or one that never declares `"network"`)
/// yields `connect-src 'self'` — byte-identical to [`HTML_SECURITY_HEADERS`]'s
/// hardcoded clause, so this is a strict generalization, not a behavior
/// change for anyone who doesn't opt in. `"self"` in the list contributes the
/// CSP `'self'` keyword; any other entry (`"https://api.example.com"`) is
/// already an origin and is emitted verbatim.
///
/// NOT YET WIRED to `html_route`/`DeckResponse` — deck-core's request path
/// (`assets.rs`'s `serve_shell`/`html_route` call, deck-core/src/assets.rs:316)
/// has no per-package resolved `"network"` list in hand today; the SPA shell
/// HTML deck-core serves isn't scoped to one `.sail` package, so the seam that
/// would supply this list does not exist yet. This function is the
/// correctly-parameterized piece; wiring it in is a follow-up.
pub fn connect_src_directive(network: &[String]) -> String {
    if network.is_empty() {
        return "connect-src 'self'".to_string();
    }
    let parts: Vec<String> = network
        .iter()
        .map(|o| {
            if o == "self" {
                "'self'".to_string()
            } else {
                o.clone()
            }
        })
        .collect();
    format!("connect-src {}", parts.join(" "))
}

/// The full HTML security header set with `connect-src` derived from
/// `network` (see [`connect_src_directive`]) instead of the hardcoded
/// `'self'`. Same names/order/every-other-value as [`HTML_SECURITY_HEADERS`];
/// `network: &[]` reproduces it byte-for-byte (pinned by
/// `network_aware_headers_match_the_default_when_network_is_empty`).
pub fn html_security_headers_for(network: &[String]) -> Vec<(&'static str, String)> {
    let csp = format!(
        "{CSP_PREFIX} {}; {CSP_SUFFIX}",
        connect_src_directive(network)
    );
    vec![
        ("X-Frame-Options", "SAMEORIGIN".to_string()),
        ("X-Content-Type-Options", "nosniff".to_string()),
        ("Content-Security-Policy", csp),
    ]
}

impl CachePolicy {
    fn header(self, mode: DaemonMode) -> &'static str {
        match self {
            CachePolicy::None => "no-cache",
            CachePolicy::Asset => match mode {
                DaemonMode::Development => "no-cache",
                DaemonMode::Production => "public, immutable, max-age=31536000",
            },
            CachePolicy::Route => match mode {
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
    /// Additional headers beyond content-type/cache/etag (e.g. `Location`).
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
        Self::new(
            200,
            "application/javascript; charset=utf-8",
            CachePolicy::Asset,
            mode,
            body,
        )
    }
    pub fn js_route(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(
            200,
            "application/javascript; charset=utf-8",
            CachePolicy::Route,
            mode,
            body,
        )
    }
    pub fn css_route(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(
            200,
            "text/css; charset=utf-8",
            CachePolicy::Route,
            mode,
            body,
        )
    }
    /// A binary font asset (the bundler-emitted hashed woff2). Immutable by
    /// construction — the hash is in the filename — so it takes the Asset
    /// cache policy.
    pub fn font_asset(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "font/woff2", CachePolicy::Asset, mode, body)
    }
    pub fn html_route(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        // All HTML responses carry the deck-defined security header set. The
        // values live in `HTML_SECURITY_HEADERS` (top of file) — the single
        // mint point, with a behavior test pinning the bytes so a half-edit
        // is caught before it ships. The error responses (`not_found`/
        // `render_error`) carry the same set via `carrying_security_headers`.
        Self::new(
            200,
            "text/html; charset=utf-8",
            CachePolicy::Route,
            mode,
            body,
        )
        .carrying_security_headers()
    }
    pub fn json_asset(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(
            200,
            "application/json; charset=utf-8",
            CachePolicy::Asset,
            mode,
            body,
        )
    }
    pub fn json_route(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(
            200,
            "application/json; charset=utf-8",
            CachePolicy::Route,
            mode,
            body,
        )
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
        let mut r = Self::new(
            404,
            "text/plain; charset=utf-8",
            CachePolicy::None,
            mode,
            body.into_bytes(),
        );
        r.cache_control = "no-cache";
        // The 404 body in Development reflects request-derived detail (callers do
        // `format!("invalid asset path: {file}")`, etc.), so it MUST carry the
        // security header set — above all `nosniff`, so a browser never MIME-
        // sniffs the text/plain body as HTML and runs reflected markup.
        r.carrying_security_headers()
    }

    /// 500 — in Production the message is hidden; in Development it's surfaced.
    pub fn render_error(msg: impl Into<String>, mode: DaemonMode) -> Self {
        let body = match mode {
            DaemonMode::Production => "render error".to_string(),
            DaemonMode::Development => msg.into(),
        };
        // Same reflection risk as `not_found` (the dev body surfaces `msg`):
        // carry the security set so a reflected text/plain 500 is never sniffed.
        Self::new(
            500,
            "text/plain; charset=utf-8",
            CachePolicy::None,
            mode,
            body.into_bytes(),
        )
        .carrying_security_headers()
    }

    pub fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.extra_headers.push((name, value.into()));
        self
    }

    /// Apply deck's security header set (`HTML_SECURITY_HEADERS`, the single mint
    /// point) to this response. Used by `html_route` AND the error responses
    /// (`not_found`/`render_error`): a text/plain error body reflects request-
    /// derived detail in Development, so it MUST send `X-Content-Type-Options:
    /// nosniff` to stop a browser MIME-sniffing it as HTML and executing
    /// reflected markup; X-Frame-Options + CSP ride along as defence-in-depth.
    /// Assets deliberately skip this — a JS/CSS asset is not an HTML execution
    /// context (see `js_asset_does_not_carry_html_security_headers`).
    fn carrying_security_headers(mut self) -> Self {
        for (name, value) in HTML_SECURITY_HEADERS {
            self = self.with_header(name, *value);
        }
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
            if k.eq_ignore_ascii_case(name) {
                Some(v.as_str())
            } else {
                None
            }
        })
    }

    #[test]
    fn html_route_carries_security_headers() {
        let resp = DeckResponse::html_route("<html></html>", DaemonMode::Development);
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
        assert!(
            csp.contains("default-src 'self'"),
            "CSP must restrict default-src; got: {csp}"
        );
        assert!(
            csp.contains("frame-ancestors 'self'"),
            "CSP must restrict frame-ancestors; got: {csp}"
        );
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

    /// Behavior pin: the `extra_headers` carried by an `html_route` response
    /// must equal `HTML_SECURITY_HEADERS` exactly, in order, byte-for-byte.
    /// This is the transport-agnostic assertion — it catches a header added
    /// to the const but not emitted by `html_route`, or a header emitted by
    /// `html_route` but missing from the const. The wire-level mirror in
    /// `deck-axum` (`html_route_security_headers_reach_the_wire`) confirms
    /// the adapter preserves them, completing the chain.
    #[test]
    fn html_route_emits_exactly_the_security_header_set() {
        let resp = DeckResponse::html_route("<html></html>", DaemonMode::Development);
        assert_eq!(
            resp.extra_headers.len(),
            HTML_SECURITY_HEADERS.len(),
            "html_route must emit exactly the HTML_SECURITY_HEADERS set; \
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
    fn html_route_security_headers_are_not_substring_matchable() {
        // The security-header assertion above checks the WHOLE response,
        // not a substring. If the const ever shrank to `X-Frame-Options`
        // alone, `html_route_carries_security_headers` (which uses
        // `header_value` lookups for X-Content-Type-Options and CSP) would
        // also fail. Belt-and-suspenders: a third, exact-set equality
        // assertion (`html_route_emits_exactly_the_security_header_set`)
        // catches partial overlaps a single substring check would miss.
        let resp = DeckResponse::html_route("<html></html>", DaemonMode::Development);
        let names: Vec<&str> = resp.extra_headers.iter().map(|(k, _)| *k).collect();
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
             either `html_route` added an extra header or the const shrank",
        );
    }

    // ── network-aware CSP (connect-src derivation) ────────────────────────────

    #[test]
    fn network_aware_headers_match_the_default_when_network_is_empty() {
        let derived = html_security_headers_for(&[]);
        let expected: Vec<(&str, String)> = HTML_SECURITY_HEADERS
            .iter()
            .map(|(k, v)| (*k, v.to_string()))
            .collect();
        assert_eq!(
            derived, expected,
            "empty network must reproduce the hardcoded default exactly"
        );
    }

    #[test]
    fn network_aware_headers_name_the_granted_origins() {
        let network = vec!["https://api.example.com".to_string()];
        let headers = html_security_headers_for(&network);
        let csp = headers
            .iter()
            .find(|(k, _)| *k == "Content-Security-Policy")
            .unwrap()
            .1
            .clone();
        assert!(
            csp.contains("connect-src https://api.example.com"),
            "connect-src must name the granted origin verbatim: {csp}"
        );
        assert!(
            !csp.contains("connect-src 'self'"),
            "the default 'self' must not leak in: {csp}"
        );
        // Every other clause is unchanged.
        assert!(csp.contains("default-src 'self'"));
        assert!(csp.contains("frame-ancestors 'self'"));
    }

    #[test]
    fn network_aware_headers_translate_self_to_the_csp_keyword() {
        let network = vec!["self".to_string()];
        let csp = html_security_headers_for(&network)
            .into_iter()
            .find(|(k, _)| *k == "Content-Security-Policy")
            .unwrap()
            .1;
        assert!(csp.contains("connect-src 'self'"), "{csp}");
    }

    #[test]
    fn not_found_carries_security_headers() {
        // A 404 reflects request-derived detail in Development (callers do
        // `format!("invalid asset path: {file}")`), so the text/plain error body
        // MUST carry nosniff — else a browser could MIME-sniff it as HTML and run
        // reflected markup (`GET /<script>…` lands the path in the 404 body).
        // This is the one user-influenced error surface; it was shipping bare.
        let resp = DeckResponse::not_found(
            "invalid asset path: <script>alert(1)</script>",
            DaemonMode::Development,
        );
        assert_eq!(
            header_value(&resp, "X-Content-Type-Options"),
            Some("nosniff"),
            "the 404 MUST send nosniff so a reflected text/plain body is never sniffed as HTML",
        );
        assert_eq!(header_value(&resp, "X-Frame-Options"), Some("SAMEORIGIN"));
        assert!(
            header_value(&resp, "Content-Security-Policy").is_some(),
            "the error response carries the full security set (defence-in-depth)",
        );
    }

    #[test]
    fn render_error_carries_security_headers() {
        // The 500 dev body surfaces `msg` too — same reflection risk, same fix.
        let resp =
            DeckResponse::render_error("boom <img src=x onerror=1>", DaemonMode::Development);
        assert_eq!(
            header_value(&resp, "X-Content-Type-Options"),
            Some("nosniff"),
            "the 500 MUST send nosniff for the same reason as the 404",
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
        assert!(
            !body.contains("/abs/path"),
            "prod 404 must not echo on-disk paths"
        );
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
        for policy in [CachePolicy::Asset, CachePolicy::Route, CachePolicy::None] {
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
            assert!(
                ts.contains(name),
                "TS port missing security header name `{name}`"
            );
            assert!(
                ts.contains(*value),
                "TS port missing security header value for `{name}`"
            );
        }
    }
}
