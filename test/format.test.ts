import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { join } from 'path'
import { MusicTagger } from '../index'
import { readFileSync } from 'fs'
import { base } from './const'

const samples = [
  { file: 'mp3.mp3', description: 'MP3' },
  { file: 'flac.flac', description: 'FLAC' },
  { file: 'ogg.opus', description: 'OGG Opus' },
  { file: 'wav.wav', description: 'WAV' },
] as const

describe.sequential('Cross-format Metadata', () => {
  let tagger: MusicTagger

  afterEach(() => {
    if (!tagger.isDisposed()) {
      tagger.dispose()
    }
  })

  for (const sample of samples) {
    describe(sample.description, () => {
      beforeEach(() => {
        tagger = new MusicTagger()
        tagger.loadBuffer(readFileSync(join(base, sample.file)))
      })

      it('should load the file successfully', () => {
        expect(tagger.isDisposed()).toBe(false)
        expect(tagger.tagType).toBeTruthy()
      })

      it('should read title (string or null)', () => {
        const title = tagger.title
        expect(title === null || typeof title === 'string').toBe(true)
      })

      it('should read artist (string or null)', () => {
        const artist = tagger.artist
        expect(artist === null || typeof artist === 'string').toBe(true)
      })

      it('should read album (string or null)', () => {
        const album = tagger.album
        expect(album === null || typeof album === 'string').toBe(true)
      })

      it('should read year (number or null)', () => {
        const year = tagger.year
        expect(year === null || typeof year === 'number').toBe(true)
      })

      it('should read genre (string or null)', () => {
        const genre = tagger.genre
        expect(genre === null || typeof genre === 'string').toBe(true)
      })

      it('should read track number (number or null)', () => {
        const trackNumber = tagger.trackNumber
        expect(trackNumber === null || typeof trackNumber === 'number').toBe(true)
      })

      it('should read disc number (number or null)', () => {
        const discNumber = tagger.discNumber
        expect(discNumber === null || typeof discNumber === 'number').toBe(true)
      })

      it('should read comment (string or null)', () => {
        const comment = tagger.comment
        expect(comment === null || typeof comment === 'string').toBe(true)
      })

      it('should read pictures (array or null)', () => {
        const pictures = tagger.pictures
        expect(pictures === null || Array.isArray(pictures)).toBe(true)
      })

      it('should set and get title', () => {
        tagger.title = 'Cross-format Title'
        expect(tagger.title).toBe('Cross-format Title')
      })

      it('should set and get artist', () => {
        tagger.artist = 'Cross-format Artist'
        expect(tagger.artist).toBe('Cross-format Artist')
      })

      it('should set and get album', () => {
        tagger.album = 'Cross-format Album'
        expect(tagger.album).toBe('Cross-format Album')
      })

      it('should set and remove title', () => {
        tagger.title = 'Temporary Title'
        tagger.title = null
        expect(tagger.title).toBeNull()
      })

      it('should set and remove artist', () => {
        tagger.artist = 'Temporary Artist'
        tagger.artist = null
        expect(tagger.artist).toBeNull()
      })

      it('should set year and read it back', () => {
        tagger.year = 2024
        expect(tagger.year).toBe(2024)
      })

      it('should set genre and read it back', () => {
        tagger.genre = 'Electronic'
        expect(tagger.genre).toBe('Electronic')
      })

      it('should save to buffer and reload with modifications', () => {
        tagger.title = 'Saved Title'
        tagger.artist = 'Saved Artist'
        tagger.save()

        const savedBuffer = tagger.buffer
        expect(savedBuffer).toBeInstanceOf(Uint8Array)
        expect(savedBuffer.length).toBeGreaterThan(0)

        const reloadedTagger = new MusicTagger()
        try {
          reloadedTagger.loadBuffer(savedBuffer)
          expect(reloadedTagger.title).toBe('Saved Title')
          expect(reloadedTagger.artist).toBe('Saved Artist')
        } finally {
          reloadedTagger.dispose()
        }
      })
    })
  }
})
