//! Build a `taffy::TaffyTree` from a parsed (and style-resolved) element
//! forest, then run `compute_layout`.

use crate::html::Element;
use crate::style::{resolve, ResolvedStyle, Role, Visibility};
use anyhow::Result;
use taffy::prelude::*;

#[derive(Debug)]
pub struct Layout {
    pub root: NodeId,
    pub taffy: TaffyTree,
    pub styles: Vec<ResolvedStyle>,
    pub nodes: Vec<Option<NodeId>>,
    pub elements: Vec<Element>,
    pub viewport: (f32, f32),
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

    let mut taffy: TaffyTree = TaffyTree::new();
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
        let resolved = resolve(elem);
        let style = resolved.style.clone();
        let node = taffy
            .new_leaf(style)
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
            .map_err(|e| anyhow::anyhow!("reparenting {i} → {parent_node:?}: {e:?}"))?;
    }

    let available = Size {
        width: AvailableSpace::Definite(viewport.0),
        height: AvailableSpace::Definite(viewport.1),
    };
    taffy
        .compute_layout(root, available)
        .map_err(|e| anyhow::anyhow!("taffy compute_layout failed: {e:?}"))?;

    Ok(Layout { root, taffy, styles, nodes, elements, viewport })
}
