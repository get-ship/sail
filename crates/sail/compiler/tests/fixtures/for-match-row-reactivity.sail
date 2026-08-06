<controller>
// Keyed <for> + nested <match> fixture. `toggle(id)` replaces the row object
// (same `id` key, so bindFor's keyed-reuse path fires — no remount) but flips
// `status`, which should re-select the <match> arm WITHOUT changing which
// row is mounted.
import { controller, signal } from '@ship/sail-runtime'

export interface Task {
  id: string
  label: string
  status: 'todo' | 'done'
}

export default controller(() => {
  const tasks = signal<Task[]>([
    { id: 'a', label: 'Alpha', status: 'todo' },
    { id: 'b', label: 'Bravo', status: 'done' },
  ])
  const toggle = (id: string): void => {
    tasks.set(
      tasks().map((t) => (t.id === id ? { ...t, status: t.status === 'todo' ? 'done' : 'todo' } : t)),
    )
  }
  return { tasks, toggle }
})
</controller>

<column gap="2">
  <for {tasks() as task} key={task.id}>
  <row gap="2" data-test="row" data-id={task.id}>
    <span>{task.label}</span>
    <match {task.status}>
      {'todo' => <em data-test="status">todo</em>}{'done' => <strong data-test="status">done</strong>}</match>
    <button data-test="toggle" on:click={() => toggle(task.id)}>toggle</button>
  </row>
  </for>
</column>
