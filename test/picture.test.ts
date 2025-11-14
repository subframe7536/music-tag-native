import { describe, it, expect } from 'vitest'
import { MetaPicture } from '../index'

describe('MetaPicture', () => {
  it('should create a MetaPicture instance', () => {
    const data = new Uint8Array([1, 2, 3, 4, 5])
    const picture = new MetaPicture('image/jpeg', data, 'Test description')

    expect(picture.mimeType).toBe('image/jpeg')
    expect(picture.data).toEqual(data)
    expect(picture.description).toBe('Test description')
    expect(picture.coverType).toBeDefined()
  })

  it('should create a MetaPicture without description', () => {
    const data = new Uint8Array([1, 2, 3])
    const picture = new MetaPicture('image/png', data)

    expect(picture.mimeType).toBe('image/png')
    expect(picture.data).toEqual(data)
    expect(picture.description).toBeNull()
  })

  it('should create a MetaPicture with null description', () => {
    const data = new Uint8Array([1, 2, 3])
    const picture = new MetaPicture('image/gif', data, null)

    expect(picture.mimeType).toBe('image/gif')
    expect(picture.data).toEqual(data)
    expect(picture.description).toBeNull()
  })
})
