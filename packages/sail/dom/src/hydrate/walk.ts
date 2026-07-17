// Walk a child-index path from `root` to a descendant node.
//
// The compiler emits `walk(root, [0, 2, 1])` to address a node by its
// position in the SSR'd subtree. Each entry indexes into `childNodes`
// (so it crosses both element and text/comment children, matching the
// position the SSR walker emitted).

export function walk(root: Node, path: readonly number[]): Node {
  let n: Node = root
  for (let i = 0; i < path.length; i++) {
    const next = n.childNodes[path[i]!]
    if (!next) {
      throw new Error(
        `[ship-set/hydrate] walk: path ${JSON.stringify(path)} fell off at step ${i} ` +
          `(parent <${(n as Element).tagName?.toLowerCase?.() ?? n.nodeName}> has ${n.childNodes.length} children)`,
      )
    }
    n = next
  }
  return n
}
