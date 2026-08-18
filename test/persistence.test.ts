import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

import { MetaPicture, MusicFile } from '../index.js'

import { base } from './const.ts'

const samples = ['mp3.mp3', 'mp3-no-tags.mp3', 'flac.flac', 'ogg.opus', 'wav.wav']

describe('Metadata persistence matrix', { concurrent: false }, () => {
  for (const sample of samples) {
    it(`${sample} persists metadata through async save and reload`, async () => {
      const source = new Uint8Array(readFileSync(join(base, sample)))
      const file = await MusicFile.load(source)

      file.title = 'Persisted title'
      file.artist = 'Persisted artist'
      file.album = 'Persisted album'
      file.albumArtist = 'Persisted album artist'
      file.genre = 'Persisted genre'
      file.trackNumber = 2
      file.trackTotal = 10
      file.discNumber = 1
      file.discsTotal = 2
      file.lyrics = 'Persisted lyrics\nSecond line'
      file.rating = 4
      file.trackReplayGain = -5.5
      file.trackReplayPeak = 0.95
      file.albumReplayGain = -6
      file.albumReplayPeak = 0.98
      file.pictures = [new MetaPicture('image/jpeg', new Uint8Array([255, 216, 255, 217]), 'Cover')]

      const saved = await file.save(source)
      const reloaded = await MusicFile.load(saved)

      expect({
        title: reloaded.title,
        artist: reloaded.artist,
        album: reloaded.album,
        albumArtist: reloaded.albumArtist,
        genre: reloaded.genre,
        trackNumber: reloaded.trackNumber,
        trackTotal: reloaded.trackTotal,
        discNumber: reloaded.discNumber,
        discsTotal: reloaded.discsTotal,
        lyrics: reloaded.lyrics,
        rating: reloaded.rating,
        trackReplayGain: reloaded.trackReplayGain,
        trackReplayPeak: reloaded.trackReplayPeak,
        albumReplayGain: reloaded.albumReplayGain,
        albumReplayPeak: reloaded.albumReplayPeak,
      }).toEqual({
        title: 'Persisted title',
        artist: 'Persisted artist',
        album: 'Persisted album',
        albumArtist: 'Persisted album artist',
        genre: 'Persisted genre',
        trackNumber: 2,
        trackTotal: 10,
        discNumber: 1,
        discsTotal: 2,
        lyrics: 'Persisted lyrics\nSecond line',
        rating: 4,
        trackReplayGain: -5.5,
        trackReplayPeak: 0.95,
        albumReplayGain: -6,
        albumReplayPeak: 0.98,
      })

      expect(reloaded.pictures).toHaveLength(1)
      expect(reloaded.pictures?.[0]?.mimeType).toBe('image/jpeg')
      expect(reloaded.pictures?.[0]?.description).toBe('Cover')
      expect(reloaded.pictures?.[0]?.data).toEqual(new Uint8Array([255, 216, 255, 217]))
    })
  }
})
