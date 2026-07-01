//! Unit tests for `expr_subset::check`. Split into a sibling file to
//! keep the implementation under the 300-line cap.

use super::*;
use crate::diagnostics::Span;
use crate::ir::expr::Expr;

fn ok(src: &str) {
    check(&Expr::new(src, Span::new(0, src.len()), false))
        .unwrap_or_else(|e| panic!("expected `{src}` to pass: {}", e.message));
}

fn err(src: &str) -> String {
    check(&Expr::new(src, Span::new(0, src.len()), false))
        .map_err(|e| e.message)
        .unwrap_err()
}

// ---------- positive cases (one per allowed rule) ----------

#[test]
fn ident_read() {
    ok("count");
}

#[test]
fn literals() {
    ok("'hi'");
    ok("42");
    ok("true");
    ok("null");
    ok("undefined");
}

#[test]
fn property_access_and_optional_chain() {
    ok("a.b.c");
    ok("a?.b?.c");
}

#[test]
fn nullish_coalescing_and_logical() {
    ok("a ?? b");
    ok("a && b || c");
}

#[test]
fn arithmetic_compare_ternary_unary() {
    ok("a + b * 2 - 1");
    ok("a >= 0 && a < 10");
    ok("ok ? a : b");
    ok("!flag");
}

#[test]
fn template_literal_with_interpolation() {
    ok("`hello ${name}`");
    ok("`x=${a + b}`");
}

#[test]
fn array_and_object_literals() {
    ok("[a, b, c]");
    ok("{ k: v, n: 1 }");
    ok("[...rest]");
    ok("{ ...rest, k: 1 }");
}

#[test]
fn zero_arg_call() {
    ok("count()");
    ok("agents().value");
    ok("a.b.c()");
}

#[test]
fn allowlisted_method_calls() {
    ok("xs.slice(0, 4)");
    ok("xs.map(toName)");
    ok("xs.filter(isActive)");
    ok("xs.includes(3)");
    ok("v.toString()");
    ok("'hello'.length");
    // `.toFixed(n)` is the natural shape for number formatting in
    // templates. Agents reach for it unprompted because it's standard
    // JS. Used to reject; now admitted.
    ok("count.toFixed(2)");
    ok("price.toFixed(0)");
    // `.has()` / `.get()` — read-only Set / Map accessors. `selectedSet().has(id)`
    // is the natural canvas idiom (sui-board / sui-draw); used to reject as a
    // bare `expr` CompileError (#263-B), now admitted alongside includes/keys/…
    ok("selectedSet().has(n.id)");
    ok("incidentSet().has(r.id)");
    ok("byId.get(item.id)");
}

#[test]
fn pure_method_allowlist_broadening() {
    // String read-only predicates / search.
    ok("name.startsWith('a')");
    ok("name.endsWith('z')");
    ok("name.includes('mid')");
    ok("s.indexOf(':')");
    ok("s.lastIndexOf(':')");
    // String formatters — no mutation, no I/O.
    ok("name.trim()");
    ok("name.trimStart()");
    ok("name.trimEnd()");
    ok("name.toLowerCase()");
    ok("name.toUpperCase()");
    ok("name.padStart(4, '0')");
    ok("name.padEnd(4, '0')");
    ok("name.repeat(2)");
    ok("name.charAt(0)");
    ok("name.charCodeAt(0)");
    ok("a.concat(b)");
    // Array read-only search / iteration — deterministic callbacks only,
    // same posture as the already-allowlisted `map` / `filter`.
    ok("xs.at(0)");
    ok("xs.join(',')");
    ok("xs.find(isActive)");
    ok("xs.findIndex(isActive)");
    ok("xs.some(isActive)");
    ok("xs.every(isActive)");
    ok("xs.flat()");
}

#[test]
fn mutators_and_forbidden_constructs_still_reject() {
    // Mutators stay out — they would alias the receiver. Use non-empty arg
    // lists so the call reaches the allowlist check; bare `xs.sort()` looks
    // like a zero-arg signal read and is treated on its own path.
    let push = err("xs.push(3)");
    assert!(push.contains("push"), "msg: {push}");
    let splice = err("xs.splice(0, 1)");
    assert!(splice.contains("splice"), "msg: {splice}");
    let fill = err("xs.fill(0)");
    assert!(fill.contains("fill"), "msg: {fill}");
    let copy = err("xs.copyWithin(0)");
    assert!(copy.contains("copyWithin"), "msg: {copy}");
    // Forbidden bare keywords stay out regardless of method-allowlist
    // changes — these are independent rejection paths.
    let new_kw = err("new Date()");
    assert!(new_kw.contains("`new`"), "msg: {new_kw}");
    let this_kw = err("this.value");
    assert!(this_kw.contains("`this`"), "msg: {this_kw}");
}

#[test]
fn agents_view_real_expressions() {
    // Real expressions lifted from views/agents/view.sui.
    ok("total()");
    ok("total() === 1 ? '' : 's'");
    ok("agents().kind === 'ok'");
    ok("items().length === 0");
    ok("a.description || 'No description'");
    ok("'/agents/' + a.id");
    ok("a.harnesses && a.harnesses.length > 0");
    ok("a.harnesses.slice(0, 4)");
    ok("a.harnesses.length - 4");
    ok("(a.tags || []).slice(0, 2)");
    ok("(a.tags || []).length > 2");
}

// ---------- negative cases (one per rejected rule) ----------

#[test]
fn rejects_arrow() {
    let m = err("(x) => x + 1");
    assert!(m.contains("arrow"), "{m}");
}

#[test]
fn rejects_new() {
    let m = err("new Date()");
    assert!(m.contains("`new`"), "{m}");
}

#[test]
fn rejects_await() {
    let m = err("await thing()");
    assert!(m.contains("`await`"), "{m}");
}

#[test]
fn rejects_yield() {
    let m = err("yield 1");
    assert!(m.contains("`yield`"), "{m}");
}

#[test]
fn rejects_this() {
    let m = err("this.value");
    assert!(m.contains("`this`"), "{m}");
}

#[test]
fn rejects_assignment() {
    let m = err("a = 1");
    assert!(m.contains("assignment"), "{m}");
}

#[test]
fn rejects_update() {
    let m = err("a++");
    assert!(m.contains("increment"), "{m}");
}

#[test]
fn rejects_regex() {
    let m = err("a / /foo/");
    assert!(m.contains("regex"), "{m}");
}

#[test]
fn rejects_tagged_template() {
    let m = err("html`<p>${x}</p>`");
    assert!(m.contains("tagged template"), "{m}");
}

#[test]
fn rejects_unknown_call_with_args() {
    let m = err("doStuff(a, b)");
    assert!(m.contains("doStuff"), "{m}");
}

#[test]
fn rejects_unknown_method_call() {
    let m = err("xs.push(3)");
    assert!(m.contains("push"), "{m}");
}

#[test]
fn rejects_spread_outside_collection() {
    let m = err("foo(...xs)");
    // Either the function-call check or the spread check fires first.
    assert!(m.contains("spread") || m.contains("foo"), "{m}");
}

#[test]
fn ignores_keywords_inside_strings() {
    ok("'new Date'");
    ok("\"this is fine\"");
    ok("`yield to me`");
}

#[test]
fn template_interpolation_recurses() {
    let m = err("`x=${a = 1}`");
    assert!(m.contains("assignment"), "{m}");
}
