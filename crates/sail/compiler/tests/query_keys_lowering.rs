//! End-to-end compiler-level proof for `check::query_keys` — the static
//! cache-key/invalidation-prefix derivation pass for `query()`/`mutation()`/
//! `invalidate()` (see `crates/sail/compiler/src/check/query_keys.rs` and
//! `packages/sail/runtime/src/query.ts`'s module header for the contract).
//!
//! Modeled on `sfc_controller_scope.rs`: an SFC's inline `<controller>`
//! block is extracted with `parse::preprocess::extract_controller_block`,
//! then run through `check::query_keys::rewrite` to get the emitted JS, and
//! separately through `compile()` to prove the SAME refusal surfaces at the
//! `compile()` boundary a real build goes through (not just the resolver in
//! isolation).

use std::path::PathBuf;

use sail_compiler::check::query_keys;
use sail_compiler::parse::preprocess::extract_controller_block;
use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

fn opts(name: &str) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: Some(format!("./{name}.sail.controller.ts")),
        mode: CompileMode::Component,
        source_path: Some(PathBuf::from(format!("/x/{name}.sail"))),
        component_meta: Some(ComponentMeta {
            name: name.to_string(),
        }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn controller_body(sfc: &str) -> String {
    extract_controller_block(sfc)
        .1
        .expect("has a <controller> block")
}

const QUERY_SFC: &str = r#"<controller>
import { controller, query } from '@ship/sail-runtime'
import { crm } from '../clients/crm'

export default controller((ctx) => {
  const stage = () => 'open'
  const deals = query(() => crm.deals.list({ stage: stage() }))
  return { deals }
})
</controller>

<div>{deals().kind}</div>
"#;

#[test]
fn a_query_fixture_lowers_to_the_exact_compiler_derived_key_fn() {
    let body = controller_body(QUERY_SFC);
    let out = query_keys::rewrite(&body).expect("statically resolvable");
    assert!(
        out.contains(
            r#"query(() => crm.deals.list({ stage: stage() }), undefined, () => ["crm", "deals", "list", { stage: stage() }])"#
        ),
        "unexpected lowering:\n{out}"
    );
    // The fixture also compiles cleanly through the real `compile()` entry —
    // the query_keys check is wired into `check::run`, not just callable in
    // isolation.
    compile(QUERY_SFC, opts("query-demo")).expect("valid query() usage compiles");
}

const MUTATION_SFC: &str = r#"<controller>
import { controller, mutation } from '@ship/sail-runtime'
import { crm } from '../clients/crm'

export default controller((ctx) => {
  const draft = () => ({ id: '1', stage: 'closed' })
  const save = mutation(() => crm.deals.update(draft()))
  return { save }
})
</controller>

<button on:click={save.mutate}>save</button>
"#;

#[test]
fn a_mutation_fixture_lowers_the_auto_invalidate_namespace() {
    let body = controller_body(MUTATION_SFC);
    let out = query_keys::rewrite(&body).expect("statically resolvable");
    assert!(
        out.contains(r#"mutation(() => crm.deals.update(draft()), undefined, ["crm", "deals"])"#),
        "unexpected lowering:\n{out}"
    );
    compile(MUTATION_SFC, opts("mutation-demo")).expect("valid mutation() usage compiles");
}

const MUTATION_INVALIDATES_SFC: &str = r#"<controller>
import { controller, mutation } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
import { billing } from '../clients/billing'

export default controller((ctx) => {
  const draft = () => ({ id: '1', stage: 'closed' })
  const save = mutation(() => crm.deals.update(draft()), { invalidates: [crm.deals.list, billing.summary] })
  return { save }
})
</controller>

<button on:click={save.mutate}>save</button>
"#;

#[test]
fn a_mutation_fixture_lowers_both_the_namespace_and_the_explicit_invalidates() {
    let body = controller_body(MUTATION_INVALIDATES_SFC);
    let out = query_keys::rewrite(&body).expect("statically resolvable");
    assert!(
        out.contains(
            r#"mutation(() => crm.deals.update(draft()), undefined, ["crm", "deals"], [["crm", "deals", "list"], ["billing", "summary"]])"#
        ),
        "unexpected lowering:\n{out}"
    );
    assert!(
        !out.contains("invalidates:"),
        "invalidates: must be stripped from the emitted opts"
    );
    compile(MUTATION_INVALIDATES_SFC, opts("mutation-invalidates-demo"))
        .expect("valid mutation() usage with invalidates compiles");
}

const INVALIDATE_METHOD_SFC: &str = r#"<controller>
import { controller, invalidate } from '@ship/sail-runtime'
import { crm } from '../clients/crm'

export default controller((ctx) => {
  const refresh = () => invalidate(crm.deals.list)
  return { refresh }
})
</controller>

<button on:click={refresh}>refresh</button>
"#;

#[test]
fn invalidate_of_an_exact_method_lowers_to_dunder_invalidate() {
    let body = controller_body(INVALIDATE_METHOD_SFC);
    let out = query_keys::rewrite(&body).expect("statically resolvable");
    assert!(
        out.contains(r#"__invalidate(["crm", "deals", "list"])"#),
        "unexpected lowering:\n{out}"
    );
    compile(INVALIDATE_METHOD_SFC, opts("invalidate-method-demo"))
        .expect("valid invalidate() usage compiles");
}

const INVALIDATE_NAMESPACE_SFC: &str = r#"<controller>
import { controller, invalidate } from '@ship/sail-runtime'
import { crm } from '../clients/crm'

export default controller((ctx) => {
  const refresh = () => invalidate(crm.deals)
  return { refresh }
})
</controller>

<button on:click={refresh}>refresh</button>
"#;

#[test]
fn invalidate_of_a_namespace_lowers_to_dunder_invalidate() {
    let body = controller_body(INVALIDATE_NAMESPACE_SFC);
    let out = query_keys::rewrite(&body).expect("statically resolvable");
    assert!(
        out.contains(r#"__invalidate(["crm", "deals"])"#),
        "unexpected lowering:\n{out}"
    );
    compile(INVALIDATE_NAMESPACE_SFC, opts("invalidate-namespace-demo"))
        .expect("valid invalidate() usage compiles");
}

const INVALIDATE_CLIENT_SFC: &str = r#"<controller>
import { controller, invalidate } from '@ship/sail-runtime'
import { crm } from '../clients/crm'

export default controller((ctx) => {
  const refresh = () => invalidate(crm)
  return { refresh }
})
</controller>

<button on:click={refresh}>refresh</button>
"#;

#[test]
fn invalidate_of_a_whole_client_lowers_to_dunder_invalidate() {
    let body = controller_body(INVALIDATE_CLIENT_SFC);
    let out = query_keys::rewrite(&body).expect("statically resolvable");
    assert!(
        out.contains(r#"__invalidate(["crm"])"#),
        "unexpected lowering:\n{out}"
    );
    compile(INVALIDATE_CLIENT_SFC, opts("invalidate-client-demo"))
        .expect("valid invalidate() usage compiles");
}

// ---------------------------------------------------------------------
// Refusals — asserting a genuine compile ERROR (never a panic, never a
// silent pass-through) at the `compile()` boundary.
// ---------------------------------------------------------------------

const COMPUTED_MEMBER_SFC: &str = r#"<controller>
import { controller, query } from '@ship/sail-runtime'
import { crm } from '../clients/crm'

export default controller((ctx) => {
  const method = 'list'
  const deals = query(() => crm.deals[method]({ stage: 'open' }))
  return { deals }
})
</controller>

<div>{deals().kind}</div>
"#;

#[test]
fn a_computed_member_thunk_is_a_compile_error() {
    let err = compile(COMPUTED_MEMBER_SFC, opts("computed-demo"))
        .expect_err("computed member access must refuse, not silently emit something");
    assert!(
        err.message.contains("computed member access"),
        "{}",
        err.message
    );
}

const NON_IMPORTED_CLIENT_SFC: &str = r#"<controller>
import { controller, query } from '@ship/sail-runtime'

export default controller((ctx) => {
  const crm = { deals: { list: () => Promise.resolve({ items: [] }) } }
  const deals = query(() => crm.deals.list())
  return { deals }
})
</controller>

<div>{deals().kind}</div>
"#;

#[test]
fn a_thunk_closing_over_a_non_imported_binding_is_a_compile_error() {
    let err = compile(NON_IMPORTED_CLIENT_SFC, opts("non-imported-demo"))
        .expect_err("a non-imported local can't be a client, even if it looks client-shaped");
    assert!(
        err.message.contains("not a client imported"),
        "{}",
        err.message
    );
}

const INVALIDATE_LITERAL_SFC: &str = r#"<controller>
import { controller, invalidate } from '@ship/sail-runtime'

export default controller((ctx) => {
  const refresh = () => invalidate('crm.deals.list')
  return { refresh }
})
</controller>

<button on:click={refresh}>refresh</button>
"#;

#[test]
fn invalidate_with_a_string_literal_is_a_compile_error() {
    let err = compile(INVALIDATE_LITERAL_SFC, opts("invalidate-literal-demo"))
        .expect_err("a string literal is a handwritten key — must refuse");
    assert!(
        err.message.contains("never a string or array literal"),
        "{}",
        err.message
    );
}

const NO_CALL_THUNK_SFC: &str = r#"<controller>
import { controller, query } from '@ship/sail-runtime'
import { crm } from '../clients/crm'

export default controller((ctx) => {
  const deals = query(() => crm.deals.list)
  return { deals }
})
</controller>

<div>{deals().kind}</div>
"#;

#[test]
fn a_thunk_that_calls_nothing_is_a_compile_error() {
    let err = compile(NO_CALL_THUNK_SFC, opts("no-call-demo"))
        .expect_err("a thunk with no call has nothing to derive a key from");
    assert!(
        err.message.contains("doesn't call anything"),
        "{}",
        err.message
    );
}
