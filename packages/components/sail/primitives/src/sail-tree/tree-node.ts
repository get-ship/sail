/**
 * Floor shape for hierarchical tree rows (formerly sail-tree-view controller).
 * Identity props + optional children; extra fields ride via intersection.
 */
export interface TreeNode {
  id: string
  label?: string
  children?: TreeNode[]
}
