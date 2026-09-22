import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { defineRoutes, matchRoute, setEditMode } from '../src/index'

beforeEach(() => {
  setEditMode(true)
})

afterEach(() => {
  setEditMode(false)
})

describe('matchRoute', () => {
  it('returns the registered shape including head', () => {
    const head = {
      title: 'About',
      metas: [{ name: 'description', content: 'Who we are.' }],
    }
    const shape = { pattern: '/head-survives-uniq', name: 'about', head }
    defineRoutes([shape])
    const matched = matchRoute('/head-survives-uniq')
    expect(matched?.pattern).toBe(shape.pattern)
    expect(matched?.name).toBe(shape.name)
    expect(matched?.head).toBe(head)
  })
})
