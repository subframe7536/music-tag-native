import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import { describe, it, expect, beforeEach } from 'vitest'

import { TaggedFile } from '../index'

import { base } from './const'

const samples = [
  { file: 'mp3.mp3', description: 'MP3' },
  { file: 'flac.flac', description: 'FLAC' },
  { file: 'ogg.opus', description: 'OGG Opus' },
  { file: 'wav.wav', description: 'WAV' },
] as const

describe.sequential('Cross-format Metadata', () => {
  for (const sample of samples) {
    const buf = readFileSync(join(base, sample.file))
    let taggedFile: TaggedFile
    describe(sample.description, () => {
      beforeEach(() => {
        taggedFile = TaggedFile.loadSync(buf)
      })

      it('should read title (string or null)', () => {
        const title = taggedFile.title
        expect(title === null || typeof title === 'string').toBe(true)
      })

      it('should read artist (string or null)', () => {
        const artist = taggedFile.artist
        expect(artist === null || typeof artist === 'string').toBe(true)
      })

      it('should read album (string or null)', () => {
        const album = taggedFile.album
        expect(album === null || typeof album === 'string').toBe(true)
      })

      it('should read year (number or null)', () => {
        const year = taggedFile.year
        expect(year === null || typeof year === 'number').toBe(true)
      })

      it('should read genre (string or null)', () => {
        const genre = taggedFile.genre
        expect(genre === null || typeof genre === 'string').toBe(true)
      })

      it('should read track number (number or null)', () => {
        const trackNumber = taggedFile.trackNumber
        expect(trackNumber === null || typeof trackNumber === 'number').toBe(true)
      })

      it('should read disc number (number or null)', () => {
        const discNumber = taggedFile.discNumber
        expect(discNumber === null || typeof discNumber === 'number').toBe(true)
      })

      it('should read comment (string or null)', () => {
        const comment = taggedFile.comment
        expect(comment === null || typeof comment === 'string').toBe(true)
      })

      it('should read pictures (array or null)', () => {
        const pictures = taggedFile.pictures
        expect(pictures === null || Array.isArray(pictures)).toBe(true)
      })

      it('should set and get title', () => {
        taggedFile.title = 'Cross-format Title'
        expect(taggedFile.title).toBe('Cross-format Title')
      })

      it('should set and get artist', () => {
        taggedFile.artist = 'Cross-format Artist'
        expect(taggedFile.artist).toBe('Cross-format Artist')
      })

      it('should set and get album', () => {
        taggedFile.album = 'Cross-format Album'
        expect(taggedFile.album).toBe('Cross-format Album')
      })

      it('should set and remove title', () => {
        taggedFile.title = 'Temporary Title'
        taggedFile.title = null
        expect(taggedFile.title).toBeNull()
      })

      it('should set and remove artist', () => {
        taggedFile.artist = 'Temporary Artist'
        taggedFile.artist = null
        expect(taggedFile.artist).toBeNull()
      })

      it('should set year and read it back', () => {
        taggedFile.year = 2024
        expect(taggedFile.year).toBe(2024)
      })

      it('should set genre and read it back', () => {
        taggedFile.genre = 'Electronic'
        expect(taggedFile.genre).toBe('Electronic')
      })

      it('should save to buffer and reload with modifications', async () => {
        taggedFile.title = 'Saved Title'
        taggedFile.artist = 'Saved Artist'

        const savedBuffer = (await taggedFile.save(buf)) as Uint8Array
        expect(savedBuffer).toBeInstanceOf(Uint8Array)
        expect(savedBuffer.length).toBeGreaterThan(0)

        const reloadedTaggedFile = TaggedFile.loadSync(savedBuffer)
        expect(reloadedTaggedFile.title).toBe('Saved Title')
        expect(reloadedTaggedFile.artist).toBe('Saved Artist')
      })

      it('should save sync to buffer and reload with modifications', () => {
        taggedFile.title = 'Saved Title'
        taggedFile.artist = 'Saved Artist'

        const savedBuffer = taggedFile.saveSync(buf) as Uint8Array
        expect(savedBuffer).toBeInstanceOf(Uint8Array)
        expect(savedBuffer.length).toBeGreaterThan(0)

        const reloadedTaggedFile = TaggedFile.loadSync(savedBuffer)
        expect(reloadedTaggedFile.title).toBe('Saved Title')
        expect(reloadedTaggedFile.artist).toBe('Saved Artist')
      })
    })
  }
})
