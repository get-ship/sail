use std::path::Path;

/// The ONE canonical view-name validator — the single source of truth for the
/// whole render→serve contract. The SSR/payload path (deck-core `view`/
/// `view_payload`) and the SSE/mutation handlers (shipd) both resolve to this
/// via the `render_helpers` re-export chain, so a name the render path accepts
/// (and mints a `__subscribe__`/mutation-action URL for) is exactly a name the
/// SSE/mutation path accepts. Lives in deck-host (the lowest layer) so deck-core
/// can delegate down to it.
///
/// Accepts `[A-Za-z0-9_-]` segments, optionally `/`-nested (≤8 segments, ≤64
/// chars each). Rejects traversal and dotfiles: a segment that is empty, `..`,
/// or starts with `.` or `-`.
pub fn is_valid_view_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let segments: Vec<&str> = s.split('/').collect();
    if segments.len() > 8 {
        return false;
    }
    segments.iter().all(|seg| {
        !seg.is_empty()
            && seg.len() <= 64
            && *seg != ".."
            && !seg.starts_with('.')
            && !seg.starts_with('-')
            && seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    })
}

pub fn render_view_source(project_root: &Path, name: &str) -> Option<String> {
    let path = project_root.join("views").join(name).join("page.sail");
    std::fs::read_to_string(&path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_validation_rejects_traversal_accepts_render_path_names() {
        // Traversal + dotfiles rejected.
        assert!(!is_valid_view_name(""));
        assert!(!is_valid_view_name("../etc"));
        assert!(!is_valid_view_name(".hidden"));
        assert!(!is_valid_view_name("-leading"));
        assert!(!is_valid_view_name("a/../b"));
        // Every name the render/payload path accepts must validate here too —
        // digits, uppercase, underscore, and `/`-nesting (the divergence this
        // SSOT collapse fixes).
        assert!(is_valid_view_name("agents"));
        assert!(is_valid_view_name("agent-detail"));
        assert!(is_valid_view_name("v2"));
        assert!(is_valid_view_name("MyView"));
        assert!(is_valid_view_name("my_view"));
        assert!(is_valid_view_name("studio/agents"));
    }
}
