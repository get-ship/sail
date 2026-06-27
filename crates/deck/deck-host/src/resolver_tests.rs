//! Tests for [`crate::resolver`]. Split out to keep `resolver.rs` under the
//! file-size cap; included via `#[path]` from the resolver module so it shares
//! its `#[cfg(test)]` gate and private surface.

use super::*;
use std::fs;

fn with_tempdir<F: FnOnce(&Path)>(f: F) {
    let tmp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(tmp.path().join(".ship")).unwrap();
    f(tmp.path());
}

/// The deck-native zero-config layout — the single `views/` bucket and the
/// `page.sui` leaf. Deck names no host-private path in its default.
fn layout() -> ViewLayout {
    ViewLayout::conventional()
}

/// A HOST-OVERRIDE layout: the deck-native `views/` bucket plus Ship's
/// `.ship/views`. The override path stays — a host that serves additional
/// buckets supplies them; deck does not bake them into `conventional()`.
fn host_layout() -> ViewLayout {
    ViewLayout {
        view_dirs: vec!["views".into(), ".ship/views".into()],
        view_leaf: "page.sui".into(),
        first_party_root: None,
    }
}

#[test]
fn on_disk_page_sui_resolves_in_overridden_ship_views() {
    // A host that overrides `view_dirs` to add `.ship/views` resolves views
    // from that bucket — the override path the framework default no longer bakes.
    with_tempdir(|root| {
        let view_dir = root.join(".ship/views/widget");
        fs::create_dir_all(&view_dir).unwrap();
        fs::write(
            view_dir.join("page.sui"),
            r#"<state name="x" value="1" /><p>{x()}</p>"#,
        )
        .unwrap();

        let r = resolve(root, &host_layout(), "widget", None).expect("on-disk widget resolves");
        assert!(r.source.contains("name=\"x\""));
        assert!(r.source_path.is_some());
        assert!(r.params.is_empty());
    });
}

#[test]
fn on_disk_param_segment_captures() {
    with_tempdir(|root| {
        let leaf = root.join("views/things/[id]");
        fs::create_dir_all(&leaf).unwrap();
        fs::write(
            leaf.join("page.sui"),
            r#"<state name="x" value="1" /><p>{x()}</p>"#,
        )
        .unwrap();

        let r = resolve(root, &layout(), "things", Some("42")).expect("param dir resolves");
        assert_eq!(r.params.get("id").map(String::as_str), Some("42"));
    });
}

#[test]
fn missing_view_returns_diagnostic() {
    with_tempdir(|root| {
        // With a host-override layout, both configured buckets are named in
        // the diagnostic so an author can see every place deck looked.
        let err = resolve(root, &host_layout(), "ghost", None).unwrap_err();
        assert!(err.message.contains("ghost"));
        assert!(err.message.contains("views/"));
        assert!(err.message.contains(".ship/views/"));
    });
}

#[test]
fn oversized_source_is_rejected() {
    // A pathologically large `.sui` would otherwise be slurped + compiled in
    // process per request. The cap stats the file first and rejects it before
    // any bytes are read, surfacing as a 404 upstream.
    with_tempdir(|root| {
        let view_dir = root.join("views/huge");
        fs::create_dir_all(&view_dir).unwrap();
        let mut big = String::with_capacity((MAX_VIEW_SOURCE_BYTES as usize) + 16);
        big.push_str("<p>");
        while (big.len() as u64) <= MAX_VIEW_SOURCE_BYTES {
            big.push_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        }
        big.push_str("</p>");
        fs::write(view_dir.join("page.sui"), &big).unwrap();

        let err = resolve(root, &layout(), "huge", None).unwrap_err();
        assert!(
            err.message.contains("too large"),
            "oversized source must be rejected; got: {}",
            err.message
        );
    });
}

#[test]
fn read_view_artifact_caps_oversized() {
    with_tempdir(|root| {
        let path = root.join("big.css");
        let big = "a".repeat((MAX_VIEW_SOURCE_BYTES as usize) + 1);
        fs::write(&path, &big).unwrap();
        assert!(read_view_artifact(&path).is_none(), "oversized CSS must be capped to None");

        let small = root.join("small.css");
        fs::write(&small, ".x{}").unwrap();
        assert_eq!(read_view_artifact(&small).as_deref(), Some(".x{}"));
    });
}

#[cfg(unix)]
#[test]
fn symlink_leaf_escaping_root_is_rejected() {
    // A symlink inside `views/<name>` whose target lives outside the
    // views bucket (here: a secret elsewhere on disk) must NOT be followed
    // and served — the containment guard rejects it (404 upstream), even
    // though the view NAME is perfectly valid.
    with_tempdir(|root| {
        let secret_dir = root.join("outside");
        fs::create_dir_all(&secret_dir).unwrap();
        let secret = secret_dir.join("secret.sui");
        fs::write(&secret, "<p>top-secret</p>").unwrap();

        let view_dir = root.join("views/leaky");
        fs::create_dir_all(&view_dir).unwrap();
        // page.sui in the view dir is a symlink pointing OUT of the bucket.
        std::os::unix::fs::symlink(&secret, view_dir.join("page.sui")).unwrap();

        let err = resolve(root, &layout(), "leaky", None).unwrap_err();
        assert!(
            err.message.contains("escapes its root"),
            "symlink escape must be rejected; got: {}",
            err.message
        );
    });
}

#[cfg(unix)]
#[test]
fn symlink_leaf_inside_root_is_allowed() {
    // A symlink that stays WITHIN the views bucket is fine — containment is
    // about escaping the project, not banning symlinks outright.
    with_tempdir(|root| {
        let views = root.join("views");
        let real_dir = views.join("real");
        fs::create_dir_all(&real_dir).unwrap();
        let real_leaf = real_dir.join("page.sui");
        fs::write(&real_leaf, "<p>in-bucket</p>").unwrap();

        let view_dir = views.join("aliased");
        fs::create_dir_all(&view_dir).unwrap();
        std::os::unix::fs::symlink(&real_leaf, view_dir.join("page.sui")).unwrap();

        let r = resolve(root, &layout(), "aliased", None).expect("in-bucket symlink resolves");
        assert!(r.source.contains("in-bucket"));
    });
}

#[test]
fn on_disk_page_sui_resolves_in_top_views() {
    with_tempdir(|root| {
        let view_dir = root.join("views/widget");
        fs::create_dir_all(&view_dir).unwrap();
        fs::write(
            view_dir.join("page.sui"),
            r#"<state name="x" value="1" /><p>{x()}</p>"#,
        )
        .unwrap();

        let r = resolve(root, &layout(), "widget", None).expect("on-disk widget resolves");
        assert!(r.source.contains("name=\"x\""));
        assert!(r.source_path.is_some());
        assert!(r.params.is_empty());
    });
}

#[test]
fn dotdot_segments_in_sub_path_stripped_before_walk() {
    // A sub_path like "../../etc/passwd" must not traverse outside the view
    // root. The ".." segments are filtered during split so they never reach
    // PathBuf::join and cannot build an escaping path even before the
    // canonicalize containment guard fires.
    with_tempdir(|root| {
        // Create a view that would be hit IF ".." didn't escape.
        let view_dir = root.join("views/myview");
        fs::create_dir_all(&view_dir).unwrap();
        fs::write(view_dir.join("page.sui"), r#"<p>safe</p>"#).unwrap();

        // Place a "secret" outside the views bucket.
        let outside = root.join("secret.sui");
        fs::write(&outside, r#"<p>LEAKED</p>"#).unwrap();

        // The traversal sub_path; after stripping ".." this becomes empty,
        // which resolves myview's page.sui (the normal path), NOT the outside file.
        let result = resolve(root, &layout(), "myview", Some("../../secret"));
        // Either resolves the view (traversal stripped → normal resolution) or
        // returns a not-found. Either way it must NOT read the outside secret.
        match &result {
            Ok(r) => {
                assert!(
                    !r.source.contains("LEAKED"),
                    "traversal must not expose the outside file; got: {}",
                    r.source
                );
            }
            Err(_) => {
                // A 404 is also a correct safe response.
            }
        }
    });
}
