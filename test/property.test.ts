import { describe, it, expect, beforeEach } from 'vitest'
import { join } from 'path'
import { TaggedFile } from '../index'
import { readFileSync } from 'fs'
import { base } from './const'

const samples = [
  { file: 'mp3.mp3', expectedQuality: "HQ", description: 'MP3' },
  { file: 'flac.flac', description: 'FLAC' },
  { file: 'ogg.opus', expectedQuality: "HQ", description: 'OGG Opus' },
  { file: 'wav.wav', description: 'WAV' },
] as const

describe('Audio Properties', () => {
  for (const sample of samples) {
    const buf = readFileSync(join(base, sample.file));
    let taggedFile: TaggedFile
    describe(sample.description, () => {
      beforeEach(() => {
        taggedFile = TaggedFile.loadSync(buf);
      })

      if ('expectedQuality' in sample) {
        it(`should be ${sample.expectedQuality} quality (lossy format)`, () => {
          expect(taggedFile.quality).toBe(sample.expectedQuality)
        })
      }

      it('should have bit depth as a number or null', () => {
        const bitDepth = taggedFile.bitDepth
        expect(bitDepth === null || typeof bitDepth === 'number').toBe(true)
      })

      it('should have bit rate as a number or null', () => {
        const bitRate = taggedFile.bitRate
        expect(bitRate === null || typeof bitRate === 'number').toBe(true)
      })

      it('should have sample rate as a number or null', () => {
        const sampleRate = taggedFile.sampleRate
        expect(sampleRate === null || typeof sampleRate === 'number').toBe(true)
      })

      it('should have channels as a number or null', () => {
        const channels = taggedFile.channels
        expect(channels === null || typeof channels === 'number').toBe(true)
      })

      it('should have a non-negative duration in milliseconds', () => {
        const duration = taggedFile.duration
        expect(typeof duration).toBe('number')
        expect(duration).toBeGreaterThanOrEqual(0)
      })
    })
  }
})
