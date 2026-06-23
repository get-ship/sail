//! Shared boundary-marker helpers used by every control-flow renderer.
//!
//! Two shapes:
//!
//! - **Active branch (rendered, visible)** — `<sui-{kind} key="bN"
//!   style="display:contents">…body…</sui-{kind}>`. The `display:contents`
//!   wrapper transparently passes its children through to layout, so the
//!   author's HTML renders exactly as if the wrapper weren't there. The
//!   `key` attribute is the binding key the attach runtime walks.
//! - **Placeholder (un-rendered, invisible)** — `<sui-{kind} key="bN"
//!   hidden></sui-{kind}>`. The `hidden` attribute suppresses display.
//!   When the runtime decides this branch should activate, it swaps the
//!   placeholder for a populated active wrapper.

use super::walker::Walker;

pub(super) fn open_active(walker: &mut Walker<'_>, kind: &str, key: &str) {
    walker.push_raw(&format!(
        "<sui-{kind} key=\"{key}\" style=\"display:contents\">"
    ));
}

/// Open an active match-branch wrapper with an explicit `branch` attribute
/// so the runtime knows which case was taken.
pub(super) fn open_active_with_branch(
    walker: &mut Walker<'_>,
    kind: &str,
    key: &str,
    branch: &str,
) {
    walker.push_raw(&format!(
        "<sui-{kind} key=\"{key}\" branch=\"{branch}\" style=\"display:contents\">"
    ));
}

pub(super) fn close_active(walker: &mut Walker<'_>, kind: &str) {
    walker.push_raw(&format!("</sui-{kind}>"));
}

pub(super) fn empty_template(walker: &mut Walker<'_>, kind: &str, key: &str) {
    walker.push_raw(&format!("<sui-{kind} key=\"{key}\" hidden></sui-{kind}>"));
}

pub(super) fn open_for_item(walker: &mut Walker<'_>, key_value: &str) {
    walker.push_raw(&format!(
        "<div key=\"{key_value}\" style=\"display:contents\">"
    ));
}

pub(super) fn close_for_item(walker: &mut Walker<'_>) {
    walker.push_raw("</div>");
}
