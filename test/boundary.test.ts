import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

import { MusicFile } from '../index.js'

import { base } from './const.ts'

const source = new Uint8Array(readFileSync(join(base, 'mp3.mp3')))

describe('numeric boundaries and async errors', () => {
  it('always returns a Promise for buffer loading and rejects invalid data', async () => {
    const result = MusicFile.load(new Uint8Array([1, 2, 3]))

    expect(result).toBeInstanceOf(Promise)
    await expect(result).rejects.toThrow()
  })

  for (const property of [
    'year',
    'trackNumber',
    'discNumber',
    'trackTotal',
    'discsTotal',
  ] as const) {
    it(`rejects invalid ${property} values`, () => {
      const file = MusicFile.loadSync(source)

      for (const value of [
        -1,
        1.5,
        Number.NaN,
        Number.POSITIVE_INFINITY,
        Number.NEGATIVE_INFINITY,
      ]) {
        expect(() => {
          ;(file as any)[property] = value
        }).toThrow()
      }
    })
  }

  it('allows zero for track and disc fields', () => {
    const file = MusicFile.loadSync(source)

    file.trackNumber = 0
    file.discNumber = 0
    file.trackTotal = 0
    file.discsTotal = 0

    expect(file.trackNumber).toBe(0)
    expect(file.discNumber).toBe(0)
    expect(file.trackTotal).toBe(0)
    expect(file.discsTotal).toBe(0)
  })

  it('restricts rating to finite integers from one through five', () => {
    const file = MusicFile.loadSync(source)

    for (const value of [
      0,
      1.5,
      6,
      Number.NaN,
      Number.POSITIVE_INFINITY,
      Number.NEGATIVE_INFINITY,
    ]) {
      expect(() => {
        file.rating = value as any
      }).toThrow()
    }
  })

  it('rejects non-finite ReplayGain values', () => {
    const file = MusicFile.loadSync(source)

    for (const property of [
      'trackReplayGain',
      'trackReplayPeak',
      'albumReplayGain',
      'albumReplayPeak',
    ] as const) {
      for (const value of [Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY]) {
        expect(() => {
          ;(file as any)[property] = value
        }).toThrow()
      }
    }
  })

  it('returns save buffer failures as Promise rejections', async () => {
    const file = MusicFile.loadSync(source)
    const result = file.save(new Uint8Array([1, 2, 3]))

    expect(result).toBeInstanceOf(Promise)
    await expect(result).rejects.toThrow()
  })
})
