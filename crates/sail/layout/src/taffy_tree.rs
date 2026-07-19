//! Build a `taffy::TaffyTree` from a parsed (and style-resolved) element
//! forest, then run `compute_layout_with_measure`. The measure closure
//! gives text leaves a real intrinsic size from the baked metrics table —
//! without it every `<p>`/`<h1>`/`<small>` measures `0×0` and the
//! diagnostic stream is full of false-positive "collapsed" warnings.

use crate::html::Element;
use crate::metrics::{table as metrics_table, Family, Size as TextSize};
use crate::style::{resolve, ResolvedStyle, Role, Visibility};
use anyhow::Result;
use std::collections::HashMap;
use taffy::prelude::*;

/// Per-leaf context for the taffy measure closure. The element index
/// (`elem_index`) is the position in the source `Vec<Element>` (skips 0,
/// the synthetic root) and is the key the measure closure uses to look up
/// the original `Element` in the side table the closure captures.
#[derive(Debug, Clone, Copy)]
pub struct NodeContext {
    pub elem_index: usize,
}

#[derive(Debug)]
pub struct Layout {
    pub root: NodeId,
    pub taffy: TaffyTree<NodeContext>,
    pub styles: Vec<ResolvedStyle>,
    pub nodes: Vec<Option<NodeId>>,
    pub elements: Vec<Element>,
    pub viewport: (f32, f32),
    /// `unknown_subtree[i]` is true iff element `i` (or any descendant in its
    /// subtree) is a tag the layout pass cannot size (custom element or
    /// replaced element). The diagnostic scan suppresses collapse/zero-size
    /// findings on any element whose `unknown_subtree` is true: we cannot
    /// authoritatively say it collapsed when we cannot see what is inside it.
    ///
    /// Index 0 (the synthetic document root) is always false.
    pub unknown_subtree: Vec<bool>,
}

impl Layout {
    pub fn computed(&self, idx: usize) -> Option<taffy::tree::Layout> {
        self.nodes[idx].and_then(|n| self.taffy.layout(n).ok().cloned())
    }
}

pub fn build<I>(elements: I, viewport: (f32, f32)) -> Result<Layout>
where
    I: IntoIterator<Item = Element>,
{
    let elements: Vec<Element> = elements.into_iter().collect();

    let mut taffy: TaffyTree<NodeContext> = TaffyTree::new();
    let mut styles: Vec<ResolvedStyle> = Vec::with_capacity(elements.len());
    let mut nodes: Vec<Option<NodeId>> = Vec::with_capacity(elements.len());

    let root_style = Style {
        display: Display::Block,
        size: Size {
            width: Dimension::length(viewport.0),
            height: Dimension::length(viewport.1),
        },
        ..Default::default()
    };
    let root = taffy
        .new_with_children(root_style.clone(), &[])
        .map_err(|e| anyhow::anyhow!("creating root failed: {e:?}"))?;

    styles.push(ResolvedStyle {
        style: root_style,
        role: Role::DocumentRoot,
        visibility: Visibility::Visible,
    });
    nodes.push(Some(root));

    for (i, elem) in elements.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let resolved = resolve(elem, &elements, viewport.0);
        let style = resolved.style.clone();
        let node = taffy
            .new_leaf_with_context(style, NodeContext { elem_index: i })
            .map_err(|e| anyhow::anyhow!("creating leaf {i} failed: {e:?}"))?;
        styles.push(resolved);
        nodes.push(Some(node));
    }

    for (i, elem) in elements.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let parent_node = if let Some(p) = elem.parent {
            nodes[p].expect("parent must have a node")
        } else {
            root
        };
        let my_node = nodes[i].unwrap();
        taffy
            .add_child(parent_node, my_node)
            .map_err(|e| anyhow::anyhow!("reparenting {i} -> {parent_node:?}: {e:?}"))?;
    }

    let available = Size {
        width: AvailableSpace::Definite(viewport.0),
        height: AvailableSpace::Definite(viewport.1),
    };

    // Side tables captured by reference for the measure closure. The
    // closure is `FnMut` and Taffy drops it when `compute_layout_with_measure`
    // returns, so this stack-bound borrow is sound.
    let metrics = metrics_table();
    let mut id_to_elem: HashMap<NodeId, usize> = HashMap::with_capacity(nodes.len());
    for (i, slot) in nodes.iter().enumerate() {
        if let Some(id) = slot {
            id_to_elem.insert(*id, i);
        }
    }

    taffy
        .compute_layout_with_measure(
            root,
            available,
            |known_dimensions, available_space, node_id, ctx, _style| {
                let Some(_ctx) = ctx else { return Size::ZERO };
                let Some(&elem_index) = id_to_elem.get(&node_id) else { return Size::ZERO };
                let elem = &elements[elem_index];
                if elem.text.is_empty() {
                    return Size::ZERO;
                }
                let (family, size) = resolve_family_size(elem_index, &elements);
                let m = match metrics.get(&(family, size)) {
                    Some(m) => m,
                    None => return Size::ZERO,
                };
                measure_text(&elem.text, m, known_dimensions, available_space)
            },
        )
        .map_err(|e| anyhow::anyhow!("taffy compute_layout failed: {e:?}"))?;

    // Mark every element whose subtree contains an unknown element. Walk
    // leaves-first so a parent's flag folds in every descendant's
    // contribution in one pass. The synthetic root (index 0) is excluded.
    let mut unknown_subtree = vec![false; elements.len()];
    for i in (1..elements.len()).rev() {
        let elem = &elements[i];
        let self_unknown = elem.is_unknown();
        let descendants_unknown = elem
            .children
            .iter()
            .any(|&ci| unknown_subtree.get(ci).copied().unwrap_or(false));
        unknown_subtree[i] = self_unknown || descendants_unknown;
    }

    Ok(Layout {
        root,
        taffy,
        styles,
        nodes,
        elements,
        viewport,
        unknown_subtree,
    })
}

/// Resolve the (family, size) for a text leaf by walking:
///   1. The element's own `font-*` and `text-*` classes.
///   2. The element's tag — `h1` -> 4xl, `h2` -> 2xl, etc.
///   3. Ancestors' classes (closest wins).
///   4. Default body / base.
fn resolve_family_size(elem_index: usize, all: &[Element]) -> (Family, TextSize) {
    let mut family: Option<Family> = None;
    let mut size: Option<TextSize> = None;

    let mut cursor = Some(elem_index);
    while (family.is_none() || size.is_none()) && cursor.is_some() {
        let i = cursor.unwrap();
        let node = &all[i];
        for c in node.class.split_whitespace() {
            if family.is_none() {
                family = match c {
                    "font-mono" => Some(Family::Mono),
                    "font-display" => Some(Family::Display),
                    "font-press-start" => Some(Family::PressStart),
                    _ => None,
                };
            }
            if size.is_none() {
                if let Some(tok) = c.strip_prefix("text-") {
                    size = Some(TextSize::from_class_token(tok));
                }
            }
        }
        if size.is_none() {
            size = tag_default_size(&node.tag);
        }
        cursor = node.parent;
    }

    (family.unwrap_or(Family::Body), size.unwrap_or(TextSize::Base))
}

fn tag_default_size(tag: &str) -> Option<TextSize> {
    match tag {
        "h1" => Some(TextSize::Xxxxl),
        "h2" => Some(TextSize::Xxl),
        "h3" => Some(TextSize::Xl),
        "h4" => Some(TextSize::Lg),
        "h5" => Some(TextSize::Md),
        "h6" => Some(TextSize::Sm),
        "small" => Some(TextSize::Sm),
        "code" => Some(TextSize::Sm),
        _ => None,
    }
}

/// Compute the measured size of a string of text given a font metric and
/// the available space. Word-wraps on overflow, stacks `line_height` per line.
fn measure_text(
    text: &str,
    m: &crate::metrics::Metrics,
    known: Size<Option<f32>>,
    available: Size<AvailableSpace>,
) -> Size<f32> {
    use taffy::geometry::AbsoluteAxis;
    let inline = AbsoluteAxis::Horizontal;
    let block = inline.other_axis();

    if text.trim().is_empty() {
        return Size::ZERO;
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return Size::ZERO;
    }

    let space_width = crate::metrics::char_width(m, ' ');
    let word_widths: Vec<f32> = words
        .iter()
        .map(|w| crate::metrics::word_width(m, w))
        .collect();
    let max_line_width = word_widths.iter().sum::<f32>()
        + (words.len().saturating_sub(1) as f32) * space_width;
    let min_line_width = *word_widths
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let inline_size = known.get_abs(inline).unwrap_or_else(|| {
        match available.get_abs(inline) {
            AvailableSpace::Definite(w) => w.min(max_line_width).max(min_line_width),
            AvailableSpace::MaxContent => max_line_width,
            AvailableSpace::MinContent => min_line_width,
        }
    });

    let block_size = known.get_abs(block).unwrap_or_else(|| {
        let mut lines = 1usize;
        let mut line_len = 0.0f32;
        for (i, w) in word_widths.iter().enumerate() {
            if i == 0 {
                line_len = *w;
            } else if line_len + space_width + w > inline_size {
                lines += 1;
                line_len = *w;
            } else {
                line_len += space_width + w;
            }
        }
        lines as f32 * m.line_height_px
    });

    Size {
        width: inline_size,
        height: block_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iframe_wrapper_is_unknown() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col sx-grow".to_string(),
                children: vec![2],
                parent: Some(0),
                ..Default::default()
            },
            Element {
                tag: "iframe".to_string(),
                class: String::new(),
                children: vec![],
                parent: Some(1),
                ..Default::default()
            },
        ];
        let tree = build(elements, (1440.0, 900.0)).unwrap();
        assert!(tree.unknown_subtree[2], "iframe itself is unknown");
        assert!(tree.unknown_subtree[1], "iframe wrapper propagates unknown up");
    }

    #[test]
    fn img_wrapper_is_known() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col sx-grow".to_string(),
                children: vec![2],
                parent: Some(0),
                ..Default::default()
            },
            Element {
                tag: "img".to_string(),
                class: String::new(),
                children: vec![],
                parent: Some(1),
                ..Default::default()
            },
        ];
        let tree = build(elements, (1440.0, 900.0)).unwrap();
        assert!(!tree.unknown_subtree[2], "img itself is known");
        assert!(!tree.unknown_subtree[1], "img wrapper is known");
    }

    #[test]
    fn custom_element_propagates_unknown_upward() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col sx-grow".to_string(),
                children: vec![2],
                parent: Some(0),
                ..Default::default()
            },
            Element {
                tag: "sail-code-editor".to_string(),
                class: String::new(),
                children: vec![],
                parent: Some(1),
                ..Default::default()
            },
        ];
        let tree = build(elements, (1440.0, 900.0)).unwrap();
        assert!(tree.unknown_subtree[2], "custom element itself is unknown");
        assert!(tree.unknown_subtree[1], "parent of unknown is unknown_subtree");
    }

    #[test]
    fn text_leaf_measures_non_zero() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "p".to_string(),
                class: "text-base".to_string(),
                text: "Hello".to_string(),
                children: vec![],
                parent: Some(0),
                ..Default::default()
            },
        ];
        let tree = build(elements, (1440.0, 900.0)).unwrap();
        let layout = tree.computed(1).unwrap();
        assert!(layout.size.height > 0.0, "text height should be non-zero: {:?}", layout.size);
    }

    #[test]
    fn measure_text_single_line_hello() {
        let m = &crate::metrics::table()[
            &(crate::metrics::Family::Body, crate::metrics::Size::Base)
        ];
        let known = Size::<Option<f32>>::NONE;
        let available = Size {
            width: AvailableSpace::Definite(1000.0),
            height: AvailableSpace::Definite(1000.0),
        };
        let measured = measure_text("Hello", m, known, available);
        assert!(measured.width > 0.0, "non-zero width: {measured:?}");
        assert!(
            (measured.height - m.line_height_px).abs() < 0.01,
            "single-line height should be one line height: {measured:?}"
        );
    }

    #[test]
    fn measure_text_wraps_to_two_lines() {
        let m = &crate::metrics::table()[
            &(crate::metrics::Family::Body, crate::metrics::Size::Base)
        ];
        let hello_w = crate::metrics::word_width(m, "Hello");
        let world_w = crate::metrics::word_width(m, "World");
        let space_w = crate::metrics::char_width(m, ' ');
        // Available width forces a wrap after "Hello".
        let available_width = hello_w + space_w + 1.0;
        let known = Size::<Option<f32>>::NONE;
        let available = Size {
            width: AvailableSpace::Definite(available_width),
            height: AvailableSpace::Definite(1000.0),
        };
        let measured = measure_text("Hello World", m, known, available);
        let expected_height = 2.0 * m.line_height_px;
        assert!(
            (measured.height - expected_height).abs() < 0.01,
            "wrapped text should be two lines: {measured:?}"
        );
        let longest = hello_w.max(world_w);
        assert!(
            measured.width >= longest - 0.1 && measured.width <= longest + 0.1,
            "wrapped width should be the longest word: {measured:?}"
        );
    }

    #[test]
    fn measure_text_empty_returns_zero() {
        let m = &crate::metrics::table()[
            &(crate::metrics::Family::Body, crate::metrics::Size::Base)
        ];
        let known = Size::<Option<f32>>::NONE;
        let available = Size {
            width: AvailableSpace::Definite(1000.0),
            height: AvailableSpace::Definite(1000.0),
        };
        assert_eq!(measure_text("", m, known, available), Size::ZERO);
        assert_eq!(measure_text("   ", m, known, available), Size::ZERO);
    }

    #[test]
    fn sibling_of_unknown_is_known() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col sx-grow".to_string(),
                children: vec![],
                parent: Some(0),
                ..Default::default()
            },
            Element {
                tag: "sail-code-editor".to_string(),
                class: String::new(),
                children: vec![],
                parent: Some(0),
                ..Default::default()
            },
        ];
        let tree = build(elements, (1440.0, 900.0)).unwrap();
        assert!(tree.unknown_subtree[2], "sail-code-editor is unknown");
        assert!(!tree.unknown_subtree[1], "sibling column is known");
    }
}
