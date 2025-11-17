import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { join } from 'path'
import { MusicTagger } from '../index'
import { readFileSync } from 'fs'

const base = 'samples'

describe('Audio Properties', () => {
  let tagger: MusicTagger

  beforeEach(() => {
    tagger = new MusicTagger()
  })

  afterEach(() => {
    if (!tagger.isDisposed()) {
      tagger.dispose()
    }
  })
  beforeEach(() => {
    const path = join(base, 'mp3.mp3')
    tagger.loadBuffer(readFileSync(path))
  })

  it('should have a quality classification', () => {
    const quality = tagger.quality
    expect(['HQ', 'SQ', 'HiRes']).toContain(quality)
  })

  it('should have bit depth or null', () => {
    const bitDepth = tagger.bitDepth
    expect(bitDepth === null || typeof bitDepth === 'number').toBe(true)
  })

  it('should have bit rate or null', () => {
    const bitRate = tagger.bitRate
    expect(bitRate === null || typeof bitRate === 'number').toBe(true)
  })

  it('should have sample rate or null', () => {
    const sampleRate = tagger.sampleRate
    expect(sampleRate === null || typeof sampleRate === 'number').toBe(true)
  })

  it('should have channels or null', () => {
    const channels = tagger.channels
    expect(channels === null || typeof channels === 'number').toBe(true)
  })

  it('should have duration', () => {
    const duration = tagger.duration
    expect(typeof duration).toBe('number')
    expect(duration).toBeGreaterThanOrEqual(0)
  })

  it('should have tag type or null', () => {
    const tagType = tagger.tagType
    expect(
      tagType === null || ['ID3V1', 'ID3V2', 'APE', 'VORBIS', 'MP4', 'AIFF', 'RIFF', 'ILST'].includes(tagType),
    ).toBe(true)
  })
})
