import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

interface Todo { id: number; label: string; due: number; done: boolean }

export default controller<Scope>({
  scope: () => {
    const now = signal(Date.now())
    const todos = signal<Todo[]>([
      { id: 1, label: 'ship the docs',  due: Date.now() - 3600_000, done: false },
      { id: 2, label: 'review the PR',  due: Date.now() + 3600_000, done: false },
      { id: 3, label: 'archive emails', due: Date.now() + 86400_000, done: true },
    ])
    return {
      now, todos,
      toggle: (id: number): void => todos.set(todos().map((t) => t.id === id ? { ...t, done: !t.done } : t)),
    }
  },
})
