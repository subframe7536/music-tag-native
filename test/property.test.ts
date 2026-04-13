import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { join } from 'path'
import { MusicTagger } from '../index'
import { readFileSync } from 'fs'
import { base } from './const'

const VALID_TAG_TYPES = ['ID3V1', 'ID3V2', 'APE', 'VORBIS', 'MP4', 'AIFF', 'RIFF', 'ILST']
const VALID_QUALITIES = ['HQ', 'SQ', 'HiRes']

const samples = [
  { file: 'mp3.mp3', expectedQuality: 'HQ', description: 'MP3' },
  { file: 'flac.flac', description: 'FLAC' },
  { file: 'ogg.opus', expectedQuality: 'HQ', description: 'OGG Opus' },
  { file: 'wav.wav', description: 'WAV' },
] as const

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

  for (const sample of samples) {
    describe(sample.description, () => {
      beforeEach(() => {
        tagger.loadBuffer(readFileSync(join(base, sample.file)))
      })

      it('should have a valid quality classification', () => {
        expect(VALID_QUALITIES).toContain(tagger.quality)
      })

      if ('expectedQuality' in sample) {
        it(`should be ${sample.expectedQuality} quality (lossy format)`, () => {
          expect(tagger.quality).toBe(sample.expectedQuality)
        })
      }

      it('should have bit depth as a number or null', () => {
        const bitDepth = tagger.bitDepth
        expect(bitDepth === null || typeof bitDepth === 'number').toBe(true)
      })

      it('should have bit rate as a number or null', () => {
        const bitRate = tagger.bitRate
        expect(bitRate === null || typeof bitRate === 'number').toBe(true)
      })

      it('should have sample rate as a number or null', () => {
        const sampleRate = tagger.sampleRate
        expect(sampleRate === null || typeof sampleRate === 'number').toBe(true)
      })

      it('should have channels as a number or null', () => {
        const channels = tagger.channels
        expect(channels === null || typeof channels === 'number').toBe(true)
      })

      it('should have a non-negative duration in milliseconds', () => {
        const duration = tagger.duration
        expect(typeof duration).toBe('number')
        expect(duration).toBeGreaterThanOrEqual(0)
      })

      it('should have a recognized tag type', () => {
        const tagType = tagger.tagType
        expect(tagType === null || VALID_TAG_TYPES.includes(tagType!)).toBe(true)
      })
    })
  }
})
