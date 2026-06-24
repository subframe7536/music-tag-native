import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import { describe, it, expect, beforeEach } from 'vitest'

import { MusicFile } from '../index'

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
    let musicFile: MusicFile
    describe(sample.description, () => {
      beforeEach(() => {
        musicFile = MusicFile.loadSync(buf)
      })

      it('should read title (string or null)', () => {
        const title = musicFile.title
        expect(title === null || typeof title === 'string').toBe(true)
      })

      it('should read artist (string or null)', () => {
        const artist = musicFile.artist
        expect(artist === null || typeof artist === 'string').toBe(true)
      })

      it('should read album (string or null)', () => {
        const album = musicFile.album
        expect(album === null || typeof album === 'string').toBe(true)
      })

      it('should read year (number or null)', () => {
        const year = musicFile.year
        expect(year === null || typeof year === 'number').toBe(true)
      })

      it('should read genre (string or null)', () => {
        const genre = musicFile.genre
        expect(genre === null || typeof genre === 'string').toBe(true)
      })

      it('should read track number (number or null)', () => {
        const trackNumber = musicFile.trackNumber
        expect(trackNumber === null || typeof trackNumber === 'number').toBe(true)
      })

      it('should read disc number (number or null)', () => {
        const discNumber = musicFile.discNumber
        expect(discNumber === null || typeof discNumber === 'number').toBe(true)
      })

      it('should read comment (string or null)', () => {
        const comment = musicFile.comment
        expect(comment === null || typeof comment === 'string').toBe(true)
      })

      it('should read pictures (array or null)', () => {
        const pictures = musicFile.pictures
        expect(pictures === null || Array.isArray(pictures)).toBe(true)
      })

      it('should set and get title', () => {
        musicFile.title = 'Cross-format Title'
        expect(musicFile.title).toBe('Cross-format Title')
      })

      it('should set and get artist', () => {
        musicFile.artist = 'Cross-format Artist'
        expect(musicFile.artist).toBe('Cross-format Artist')
      })

      it('should set and get album', () => {
        musicFile.album = 'Cross-format Album'
        expect(musicFile.album).toBe('Cross-format Album')
      })

      it('should set and remove title', () => {
        musicFile.title = 'Temporary Title'
        musicFile.title = null
        expect(musicFile.title).toBeNull()
      })

      it('should set and remove artist', () => {
        musicFile.artist = 'Temporary Artist'
        musicFile.artist = null
        expect(musicFile.artist).toBeNull()
      })

      it('should set year and read it back', () => {
        musicFile.year = 2024
        expect(musicFile.year).toBe(2024)
      })

      it('should set genre and read it back', () => {
        musicFile.genre = 'Electronic'
        expect(musicFile.genre).toBe('Electronic')
      })

      it('should save to buffer and reload with modifications', async () => {
        musicFile.title = 'Saved Title'
        musicFile.artist = 'Saved Artist'

        const savedBuffer = (await musicFile.save(buf)) as Uint8Array
        expect(savedBuffer).toBeInstanceOf(Uint8Array)
        expect(savedBuffer.length).toBeGreaterThan(0)

        const reloadedMusicFile = MusicFile.loadSync(savedBuffer)
        expect(reloadedMusicFile.title).toBe('Saved Title')
        expect(reloadedMusicFile.artist).toBe('Saved Artist')
      })

      it('should save sync to buffer and reload with modifications', () => {
        musicFile.title = 'Saved Title'
        musicFile.artist = 'Saved Artist'

        const savedBuffer = musicFile.saveSync(buf) as Uint8Array
        expect(savedBuffer).toBeInstanceOf(Uint8Array)
        expect(savedBuffer.length).toBeGreaterThan(0)

        const reloadedMusicFile = MusicFile.loadSync(savedBuffer)
        expect(reloadedMusicFile.title).toBe('Saved Title')
        expect(reloadedMusicFile.artist).toBe('Saved Artist')
      })
    })
  }
})
