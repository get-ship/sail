//! `sail-controller-no-ambient-authority` — data flows through `ship`, not
//! raw network/eval.
//!
//! Flags, anywhere inside a `*.controller.ts` file, use of ambient authority:
//! - `fetch(...)`, `navigator.sendBeacon(...)`
//! - `new XMLHttpRequest()`, `new WebSocket(...)`, `new EventSource(...)`
//! - `eval(...)`, dynamic `import(...)`
//! - `document.cookie`
//!
//! Rationale (ADR unified-deck-router's sibling, the secure-execution model):
//! route code is untrusted (agent-authored) and runs on the highest-privilege
//! surface. All I/O must go through the `ship` SDK (the only sanctioned,
//! policy-gated path); ambient network/eval is exactly what the Deck SES
//! Compartment denies at runtime (proven in #165 — confined routes run with
//! `fetch`/`window` undefined). This rule is the author-time mirror.
//!
//! **Severity: Error** (security). CONFINED BY DEFAULT: every inline
//! `<controller>` runs this gate, path-independent — an app controller is
//! app code running under the deck's SES Compartment regardless of which
//! directory it lives in. A package opts a controller OUT of the gate by
//! declaring genuine unconfined chrome in its `sail.config.json`
//! (`"unconfined": true`), the whole-hog escape wired through
//! `crates/sail/compiler/src/check/policy.rs`'s `Policy`/`PolicyResolution`
//! — the same capability-vocabulary path the `"network"` per-origin
//! allowlist already uses. This is NOT a path heuristic: two files in the
//! same directory can resolve differently depending on which package's
//! `sail.config.json` owns them.
//!
//! Promotion to Error was gated on every confined-surface route controller being
//! migrated off raw ambient authority — now true (zero violations across the
//! confined surface, verified by the repo-wide scan). Error hard-fails compile
//! on any regression. Note `fetch` is flagged only as a bare call or on a global
//! (`window`/`globalThis`/`self`); `ship.git.fetch({})` is a sanctioned SDK op,
//! not ambient authority.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sail-controller-no-ambient-authority";

/// Banned regardless of the `"network"` allowlist — not part of the network
/// capability.
const ALWAYS_BANNED_CALLS: &[&str] = &["eval"];
/// Networking calls whose target is checked against `"network"` — see
/// [`check_confined`].
const NETWORK_GATED_CALLS: &[&str] = &["fetch"];
/// Networking constructors whose first arg (the target) is checked against
/// `"network"`.
const NETWORK_GATED_CTORS: &[&str] = &["WebSocket", "EventSource"];

/// The `"network"`/`"unconfined"`-config-blind entry — the generic
/// `CONTROLLER_RULES` table (`rules::RuleFn`'s fixed `(path, source) ->
/// Vec<Finding>` shape) calls this. Consumers reached through that table (the
/// LSP, `sail_lint` CLI, agent tooling) don't have a resolved
/// `sail.config.json` in hand, so this defaults to confined with an empty
/// network allowlist — today's full refusal, unchanged. The compiler's
/// `check::mod::collect` is config-aware and calls [`check_confined`] directly
/// instead (splicing out this rule's table-driven findings first), the same
/// "specially invoked" pattern `css_token` uses.
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    check_confined(path, source, &[], false)
}

/// The config-aware entry. `network` is the resolved `"network"` per-origin
/// allowlist (`Policy::network_for`) — empty means no grant (today's default).
/// `unconfined` is the resolved `"unconfined"` capability
/// (`Policy::unconfined_for`) — `true` grants the whole-hog opt-out and this
/// rule is skipped entirely; `false` (the default, including when no config
/// exists) keeps the controller confined and every check below applies.
///
/// - `eval`, dynamic `import()`, `document.cookie`, `navigator.sendBeacon` are
///   ALWAYS banned, regardless of `network`. They are not part of this
///   capability.
/// - `fetch(...)` / `window.fetch(...)` / `new WebSocket(...)` /
///   `new EventSource(...)`: the first argument, when a STATIC STRING
///   LITERAL, is allowed iff its origin is in `network` (or it's a
///   relative/same-origin path and `network` contains `"self"`) — flagged
///   otherwise. A DYNAMIC (non-literal) argument is allowed only when
///   `network` is non-empty (we can't statically verify a dynamic target;
///   the runtime fetch wrapper — `packages/deck/host/src/route-runner.ts` —
///   is the actual enforcement for that case) — still banned when `network`
///   is empty/absent.
/// - `new XMLHttpRequest()`: the constructor call itself carries no URL (that
///   arrives later via `.open()`, which this rule does not track — tracking
///   call-site URL flow through `.open()` is real static-analysis work this
///   rule intentionally does not take on). So the ctor is simply no longer
///   flagged once ANY non-empty `network` list is granted; it stays flagged
///   when `network` is empty, same as before.
pub fn check_confined(
    path: &Path,
    source: &str,
    network: &[String],
    unconfined: bool,
) -> Vec<Finding> {
    // Confined by default. Only a package that explicitly grants
    // `"unconfined": true` may use ambient authority freely.
    if unconfined {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = AmbientVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
        network,
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

/// Whether a static string literal `url` targets an origin permitted by
/// `network`. `"self"` in `network` permits a relative/same-origin `url`
/// (no `scheme://` prefix). An absolute `url` is compared by its
/// `scheme://host[:port]` authority against each allowlist entry's authority
/// (an entry's path, if any, is ignored — the grant is per-origin).
fn literal_target_allowed(url: &str, network: &[String]) -> bool {
    match authority_of(url) {
        Some(authority) => network
            .iter()
            .any(|entry| authority_of(entry).as_deref() == Some(&authority)),
        None => network.iter().any(|entry| entry == "self"),
    }
}

/// Extract the `scheme://host[:port]` authority from an absolute URL-ish
/// string (`https://api.example.com/x` -> `https://api.example.com`). `None`
/// for a relative/schemeless string (`/api/x`, `./x`, `api/x`) — those are
/// same-origin, matched against `"self"` instead.
fn authority_of(s: &str) -> Option<String> {
    let scheme_end = s.find("://")?;
    let after = &s[scheme_end + 3..];
    let end = after.find(['/', '?', '#']).unwrap_or(after.len());
    Some(format!("{}://{}", &s[..scheme_end], &after[..end]))
}

/// Classify a call/ctor argument as a static string literal (Some(text)) or
/// dynamic (None) — a template literal with substitutions, an identifier, a
/// call expression, etc. all count as dynamic; we cannot statically verify
/// their target.
fn static_string_arg<'a>(
    args: &'a oxc_allocator::Vec<'a, Argument<'a>>,
) -> Option<Option<&'a str>> {
    let first = args.first()?;
    let expr = first.as_expression()?;
    match expr {
        Expression::StringLiteral(s) => Some(Some(s.value.as_str())),
        _ => Some(None),
    }
}

struct AmbientVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
    network: &'src [String],
}

impl AmbientVisitor<'_> {
    fn report(&mut self, span: Span, what: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "controllers must not use ambient authority (`{what}`). All I/O goes \
                 through the `ship` SDK — the only sanctioned, policy-gated path. Raw \
                 network/eval is denied at runtime by the Deck secure-execution Compartment."
            ),
            suggest: Some("use an injected client method".to_string()),
        });
    }

    fn report_off_allowlist(&mut self, span: Span, what: &str, target: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "controllers must not use ambient authority (`{what}` to `{target}`) — \
                 its origin is not in this package's `sail.config.json` \"network\" allowlist. \
                 All I/O goes through an injected client, or grant the origin explicitly."
            ),
            suggest: Some(
                "add the origin to \"network\" in sail.config.json, or use an injected client method"
                    .to_string(),
            ),
        });
    }

    /// Whether a networking call's first argument clears the `network`
    /// allowlist — the shared decision for `fetch`/`WebSocket`/`EventSource`.
    /// `None` args (e.g. bare `fetch()`) are treated as dynamic.
    fn arg_target_allowed<'a>(
        &self,
        args: &'a oxc_allocator::Vec<'a, Argument<'a>>,
    ) -> (bool, Option<String>) {
        match static_string_arg(args) {
            // Static literal: verify its origin against the allowlist.
            Some(Some(url)) => (
                literal_target_allowed(url, self.network),
                Some(url.to_string()),
            ),
            // Dynamic target (or missing arg): allowed only when SOME grant
            // exists — we can't statically verify it, the runtime fetch
            // wrapper is the actual wall for this case.
            Some(None) | None => (!self.network.is_empty(), None),
        }
    }
}

impl<'a> Visit<'a> for AmbientVisitor<'_> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        match &expr.callee {
            // bare `eval(...)` — always banned, not part of the network capability.
            Expression::Identifier(id) if ALWAYS_BANNED_CALLS.contains(&id.name.as_str()) => {
                self.report(expr.span, id.name.as_str())
            }
            // bare `fetch(...)` — allowed when its target clears `network`.
            Expression::Identifier(id) if NETWORK_GATED_CALLS.contains(&id.name.as_str()) => {
                let (allowed, target) = self.arg_target_allowed(&expr.arguments);
                if !allowed {
                    match target {
                        Some(t) => self.report_off_allowlist(expr.span, id.name.as_str(), &t),
                        None => self.report(expr.span, id.name.as_str()),
                    }
                }
            }
            // `window.fetch(...)` / `navigator.sendBeacon(...)` — but ONLY when the
            // object is a global. A member `.fetch` on something else is a
            // domain method, not ambient authority — `ship.git.fetch({})` is a
            // sanctioned SDK op that happens to be named "fetch".
            Expression::StaticMemberExpression(m) => {
                let p = m.property.name.as_str();
                let on_global = matches!(
                    &m.object,
                    Expression::Identifier(o)
                        if matches!(o.name.as_str(), "window" | "globalThis" | "self" | "navigator")
                );
                if on_global && p == "sendBeacon" {
                    // Not part of the network capability — always banned.
                    self.report(expr.span, p)
                } else if on_global && p == "fetch" {
                    let (allowed, target) = self.arg_target_allowed(&expr.arguments);
                    if !allowed {
                        match target {
                            Some(t) => self.report_off_allowlist(expr.span, p, &t),
                            None => self.report(expr.span, p),
                        }
                    }
                }
            }
            _ => {}
        }
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e)
            }
        }
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        // dynamic import(...) — always banned, not part of the network capability.
        self.report(expr.span, "import()");
        self.visit_expression(&expr.source);
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        if let Expression::Identifier(id) = &expr.callee {
            let name = id.name.as_str();
            if name == "XMLHttpRequest" {
                // No URL at construction time (it arrives via `.open()`, which
                // this rule does not track) — no longer flagged once ANY
                // non-empty `network` grant exists; unchanged (flagged) when
                // `network` is empty.
                if self.network.is_empty() {
                    self.report(expr.span, name);
                }
            } else if NETWORK_GATED_CTORS.contains(&name) {
                let (allowed, target) = self.arg_target_allowed(&expr.arguments);
                if !allowed {
                    match target {
                        Some(t) => self.report_off_allowlist(expr.span, name, &t),
                        None => self.report(expr.span, name),
                    }
                }
            }
        }
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e)
            }
        }
    }

    fn visit_static_member_expression(&mut self, m: &StaticMemberExpression<'a>) {
        // document.cookie
        if m.property.name.as_str() == "cookie" {
            if let Expression::Identifier(obj) = &m.object
                && obj.name.as_str() == "document"
            {
                self.report(m.span, "document.cookie");
            }
        }
        self.visit_expression(&m.object);
    }
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    let mut i = 0;
    while i < cap {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // A confined-surface path (route controller) — where the rule applies.
    fn n(src: &str) -> usize {
        check(Path::new("views/git/controller.ts"), src).len()
    }

    #[test]
    fn flags_fetch() {
        assert_eq!(n("fetch('/api/v1/x')"), 1);
    }

    #[test]
    fn flags_new_xhr_and_websocket() {
        assert_eq!(n("const x = new XMLHttpRequest()"), 1);
        assert_eq!(n("const w = new WebSocket('wss://x')"), 1);
    }

    #[test]
    fn flags_dynamic_import_and_eval() {
        assert_eq!(n("const m = import('./x.js')"), 1);
        assert_eq!(n("eval('2+2')"), 1);
    }

    #[test]
    fn flags_document_cookie() {
        assert_eq!(n("const c = document.cookie"), 1);
    }

    #[test]
    fn allows_ship_calls() {
        assert_eq!(n("ship.git.overview()"), 0);
    }

    #[test]
    fn error_severity_on_confined_surface() {
        let f = check(Path::new("views/git/controller.ts"), "fetch('/x')");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::Error);
    }

    #[test]
    fn allows_sdk_methods_named_fetch() {
        // `ship.git.fetch({})` is a domain op, NOT ambient authority — only bare
        // `fetch(` or a fetch on a global (window/globalThis/self) is flagged.
        assert_eq!(n("const f = () => ship.git.fetch({})"), 0);
        assert_eq!(n("repo.fetch()"), 0);
        // …but the real ambient forms still fire.
        assert_eq!(n("window.fetch('/x')"), 1);
        assert_eq!(n("globalThis.fetch('/x')"), 1);
        assert_eq!(n("navigator.sendBeacon('/x')"), 1);
    }

    #[test]
    fn confined_by_default_regardless_of_path() {
        // No path heuristic — an inline controller anywhere is confined
        // unless its package's `sail.config.json` grants `"unconfined"`.
        let chrome = "apps/shell/src/components/sail-x/sail-x.controller.ts";
        assert_eq!(
            check(Path::new(chrome), "fetch('/x'); import('./y.js')").len(),
            2
        );
        let pack = "packages/components/ui-components/src/sail-y/sail-y.controller.ts";
        assert_eq!(check(Path::new(pack), "fetch('/z')").len(), 1);
    }

    #[test]
    fn unconfined_capability_skips_the_gate_path_independent() {
        // `unconfined: true` is a whole-hog escape from `check_confined`'s
        // `unconfined` param — it applies regardless of the file's path.
        let f = check_confined(
            Path::new("packages/components/sail-x/sail-x.controller.ts"),
            "fetch('/x'); import('./y.js')",
            &[],
            true,
        );
        assert_eq!(f.len(), 0);
    }

    // ------------------------------------------------------------------
    // `"network"` allowlist — check_confined
    // ------------------------------------------------------------------

    fn nc(src: &str, network: &[&str]) -> usize {
        let net: Vec<String> = network.iter().map(|s| s.to_string()).collect();
        check_confined(Path::new("views/git/controller.ts"), src, &net, false).len()
    }

    #[test]
    fn default_still_refuses_with_no_key_or_empty_list() {
        // No config / empty list — the network-blind `check()` entry (the
        // generic table's default) and an explicit empty allowlist behave
        // identically: still banned.
        assert_eq!(n("fetch('https://api.example.com/x')"), 1);
        assert_eq!(nc("fetch('https://api.example.com/x')", &[]), 1);
    }

    #[test]
    fn granted_origin_allows_a_matching_literal_fetch() {
        assert_eq!(
            nc(
                "fetch('https://api.example.com/x')",
                &["https://api.example.com"]
            ),
            0
        );
    }

    #[test]
    fn granted_origin_still_refuses_a_different_origin() {
        assert_eq!(
            nc("fetch('https://evil.com/x')", &["https://api.example.com"]),
            1,
            "an origin not on the allowlist must still be flagged"
        );
    }

    #[test]
    fn self_grants_relative_targets_only() {
        assert_eq!(nc("fetch('/api/x')", &["self"]), 0);
        assert_eq!(
            nc("fetch('https://evil.com/x')", &["self"]),
            1,
            "\"self\" does not grant an absolute cross-origin target"
        );
        assert_eq!(
            nc("fetch('/api/x')", &["https://api.example.com"]),
            1,
            "a relative target needs \"self\" specifically, not just any grant"
        );
    }

    #[test]
    fn dynamic_target_allowed_only_when_a_grant_exists() {
        assert_eq!(
            nc("fetch(url)", &[]),
            1,
            "a dynamic target with no grant at all stays banned"
        );
        assert_eq!(
            nc("fetch(url)", &["https://api.example.com"]),
            0,
            "a dynamic target is allowed once ANY grant exists — the runtime \
             fetch wrapper is the actual enforcement for it"
        );
    }

    #[test]
    fn websocket_and_eventsource_follow_the_same_allowlist() {
        assert_eq!(
            nc(
                "new WebSocket('wss://api.example.com')",
                &["wss://api.example.com"]
            ),
            0
        );
        assert_eq!(
            nc(
                "new WebSocket('wss://evil.com')",
                &["wss://api.example.com"]
            ),
            1
        );
        assert_eq!(
            nc(
                "new EventSource('https://api.example.com/stream')",
                &["https://api.example.com"]
            ),
            0
        );
        assert_eq!(
            nc(
                "new EventSource('https://evil.com/stream')",
                &["https://api.example.com"]
            ),
            1
        );
    }

    #[test]
    fn xhr_ctor_clears_once_any_grant_exists_no_arg_to_check() {
        // No URL at construction (arrives later via `.open()`, untracked) — the
        // bare ctor is simply unflagged once ANY non-empty grant exists.
        assert_eq!(nc("new XMLHttpRequest()", &[]), 1);
        assert_eq!(nc("new XMLHttpRequest()", &["https://api.example.com"]), 0);
    }

    #[test]
    fn eval_and_cookie_stay_banned_even_with_a_grant() {
        // Not part of the network capability — a grant never loosens these.
        assert_eq!(nc("eval('2+2')", &["https://api.example.com"]), 1);
        assert_eq!(
            nc("const c = document.cookie", &["https://api.example.com"]),
            1
        );
        assert_eq!(
            nc("const m = import('./x.js')", &["https://api.example.com"]),
            1
        );
        assert_eq!(
            nc("navigator.sendBeacon('/x')", &["https://api.example.com"]),
            1
        );
    }

    #[test]
    fn window_fetch_follows_the_allowlist_too() {
        assert_eq!(
            nc(
                "window.fetch('https://api.example.com/x')",
                &["https://api.example.com"]
            ),
            0
        );
        assert_eq!(
            nc(
                "globalThis.fetch('https://evil.com/x')",
                &["https://api.example.com"]
            ),
            1
        );
    }
}
