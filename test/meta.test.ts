import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { join } from 'path'
import { MusicTagger, MetaPicture } from '../index'
import { base } from './const'
import { readFileSync } from 'fs'

describe.sequential('Metadata Getters and Setters', () => {
  let tagger: MusicTagger
  const path = join(base, 'mp3.mp3')
  const buffer = readFileSync(path)

  afterEach(() => {
    if (!tagger.isDisposed()) {
      tagger.dispose()
    }
  })

  beforeEach(() => {
    tagger = new MusicTagger()
    tagger.loadBuffer(buffer)
  })

  describe('title', () => {
    it('should get title', () => {
      const title = tagger.title
      expect(title === null || typeof title === 'string').toBe(true)
    })

    it('should set title', () => {
      tagger.title = 'Test Title'
      expect(tagger.title).toBe('Test Title')
    })

    it('should remove title when set to null', () => {
      tagger.title = 'Test Title'
      tagger.title = null
      expect(tagger.title).toBeNull()
    })
  })

  describe('artist', () => {
    it('should get artist', () => {
      const artist = tagger.artist
      expect(artist === null || typeof artist === 'string').toBe(true)
    })

    it('should set artist', () => {
      tagger.artist = 'Test Artist'
      expect(tagger.artist).toBe('Test Artist')
    })

    it('should remove artist when set to null', () => {
      tagger.artist = 'Test Artist'
      tagger.artist = null
      expect(tagger.artist).toBeNull()
    })
  })

  describe('album', () => {
    it('should get album', () => {
      const album = tagger.album
      expect(album === null || typeof album === 'string').toBe(true)
    })

    it('should set album', () => {
      tagger.album = 'Test Album'
      expect(tagger.album).toBe('Test Album')
    })

    it('should remove album when set to null', () => {
      tagger.album = 'Test Album'
      tagger.album = null
      expect(tagger.album).toBeNull()
    })
  })

  describe('year', () => {
    it('should get year', () => {
      const year = tagger.year
      expect(year === null || typeof year === 'number').toBe(true)
    })

    it('should set year', () => {
      tagger.year = 2024
      expect(tagger.year).toBe(2024)
    })

    it('should remove year when set to null', () => {
      tagger.year = 2024
      tagger.year = null
      expect(tagger.year).toBeNull()
    })
  })

  describe('genre', () => {
    it('should get genre', () => {
      const genre = tagger.genre
      expect(genre === null || typeof genre === 'string').toBe(true)
    })

    it('should set genre', () => {
      tagger.genre = 'Rock'
      expect(tagger.genre).toBe('Rock')
    })

    it('should remove genre when set to null', () => {
      tagger.genre = 'Rock'
      tagger.genre = null
      expect(tagger.genre).toBeNull()
    })
  })

  describe('trackNumber', () => {
    it('should get track number', () => {
      const trackNumber = tagger.trackNumber
      expect(trackNumber === null || typeof trackNumber === 'number').toBe(true)
    })

    it('should set track number', () => {
      tagger.trackNumber = 1
      expect(tagger.trackNumber).toBe(1)
    })

    it('should remove track number when set to null', () => {
      tagger.trackNumber = 1
      tagger.trackNumber = null
      expect(tagger.trackNumber).toBeNull()
    })
  })

  describe('discNumber', () => {
    it('should get disc number', () => {
      const discNumber = tagger.discNumber
      expect(discNumber === null || typeof discNumber === 'number').toBe(true)
    })

    it('should set disc number', () => {
      tagger.discNumber = 1
      expect(tagger.discNumber).toBe(1)
    })

    it('should remove disc number when set to null', () => {
      tagger.discNumber = 1
      tagger.discNumber = null
      expect(tagger.discNumber).toBeNull()
    })
  })

  describe('trackTotal', () => {
    it('should get track total', () => {
      const trackTotal = tagger.trackTotal
      expect(trackTotal === null || typeof trackTotal === 'number').toBe(true)
    })

    it('should set track total', () => {
      tagger.trackTotal = 12
      expect(tagger.trackTotal).toBe(12)
    })

    it('should remove track total when set to null', () => {
      tagger.trackTotal = 12
      tagger.trackTotal = null
      expect(tagger.trackTotal).toBeNull()
    })
  })

  describe('discsTotal', () => {
    it('should get discs total', () => {
      const discsTotal = tagger.discsTotal
      expect(discsTotal === null || typeof discsTotal === 'number').toBe(true)
    })

    it('should set discs total', () => {
      tagger.discsTotal = 2
      expect(tagger.discsTotal).toBe(2)
    })

    it('should remove discs total when set to null', () => {
      tagger.discsTotal = 2
      tagger.discsTotal = null
      expect(tagger.discsTotal).toBeNull()
    })
  })

  describe('comment', () => {
    it('should get comment', () => {
      const comment = tagger.comment
      expect(comment === null || typeof comment === 'string').toBe(true)
    })

    it('should set comment', () => {
      tagger.comment = 'Test comment'
      expect(tagger.comment).toBe('Test comment')
    })

    it('should remove comment when set to null', () => {
      tagger.comment = 'Test comment'
      tagger.comment = null
      expect(tagger.comment).toBeNull()
    })
  })

  describe('albumArtist', () => {
    it('should get album artist', () => {
      const albumArtist = tagger.albumArtist
      expect(albumArtist === null || typeof albumArtist === 'string').toBe(true)
    })

    it('should set album artist', () => {
      tagger.albumArtist = 'Album Artist'
      expect(tagger.albumArtist).toBe('Album Artist')
    })

    it('should remove album artist when set to null', () => {
      tagger.albumArtist = 'Album Artist'
      tagger.albumArtist = null
      expect(tagger.albumArtist).toBeNull()
    })
  })

  describe('composer', () => {
    it('should get composer', () => {
      const composer = tagger.composer
      expect(composer === null || typeof composer === 'string').toBe(true)
    })

    it('should set composer', () => {
      tagger.composer = 'Composer Name'
      expect(tagger.composer).toBe('Composer Name')
    })

    it('should remove composer when set to null', () => {
      tagger.composer = 'Composer Name'
      tagger.composer = null
      expect(tagger.composer).toBeNull()
    })
  })

  describe('conductor', () => {
    it('should get conductor', () => {
      const conductor = tagger.conductor
      expect(conductor === null || typeof conductor === 'string').toBe(true)
    })

    it('should set conductor', () => {
      tagger.conductor = 'Conductor Name'
      expect(tagger.conductor).toBe('Conductor Name')
    })

    it('should remove conductor when set to null', () => {
      tagger.conductor = 'Conductor Name'
      tagger.conductor = null
      expect(tagger.conductor).toBeNull()
    })
  })

  describe('lyricist', () => {
    it('should get lyricist', () => {
      const lyricist = tagger.lyricist
      expect(lyricist === null || typeof lyricist === 'string').toBe(true)
    })

    it('should set lyricist', () => {
      tagger.lyricist = 'Lyricist Name'
      expect(tagger.lyricist).toBe('Lyricist Name')
    })

    it('should remove lyricist when set to null', () => {
      tagger.lyricist = 'Lyricist Name'
      tagger.lyricist = null
      expect(tagger.lyricist).toBeNull()
    })
  })

  describe('publisher', () => {
    it('should get publisher', () => {
      const publisher = tagger.publisher
      expect(publisher === null || typeof publisher === 'string').toBe(true)
    })

    it('should set publisher', () => {
      tagger.publisher = 'Publisher Name'
      expect(tagger.publisher).toBe('Publisher Name')
    })

    it('should remove publisher when set to null', () => {
      tagger.publisher = 'Publisher Name'
      tagger.publisher = null
      expect(tagger.publisher).toBeNull()
    })
  })

  describe('lyrics', () => {
    it('should get lyrics', () => {
      const lyrics = tagger.lyrics
      expect(lyrics === null || typeof lyrics === 'string').toBe(true)
    })

    it('should set lyrics', () => {
      tagger.lyrics = 'Test lyrics\nLine 2'
      expect(tagger.lyrics).toBe('Test lyrics\nLine 2')
    })

    it('should remove lyrics when set to null', () => {
      tagger.lyrics = 'Test lyrics'
      tagger.lyrics = null
      expect(tagger.lyrics).toBeNull()
    })
  })

  describe('copyright', () => {
    it('should get copyright', () => {
      const copyright = tagger.copyright
      expect(copyright === null || typeof copyright === 'string').toBe(true)
    })

    it('should set copyright', () => {
      tagger.copyright = '© 2024 Test'
      expect(tagger.copyright).toBe('© 2024 Test')
    })

    it('should remove copyright when set to null', () => {
      tagger.copyright = '© 2024 Test'
      tagger.copyright = null
      expect(tagger.copyright).toBeNull()
    })
  })

  describe('replay gain', () => {
    it('should get track replay gain', () => {
      const gain = tagger.trackReplayGain
      expect(gain === null || typeof gain === 'number').toBe(true)
    })

    it('should set track replay gain', () => {
      tagger.trackReplayGain = -5.5
      expect(tagger.trackReplayGain).toBe(-5.5)
    })

    it('should remove track replay gain when set to null', () => {
      tagger.trackReplayGain = -5.5
      tagger.trackReplayGain = null
      expect(tagger.trackReplayGain).toBeNull()
    })

    it('should get track replay peak', () => {
      const peak = tagger.trackReplayPeak
      expect(peak === null || typeof peak === 'number').toBe(true)
    })

    it('should set track replay peak', () => {
      tagger.trackReplayPeak = 0.95
      expect(tagger.trackReplayPeak).toBe(0.95)
    })

    it('should remove track replay peak when set to null', () => {
      tagger.trackReplayPeak = 0.95
      tagger.trackReplayPeak = null
      expect(tagger.trackReplayPeak).toBeNull()
    })

    it('should get album replay gain', () => {
      const gain = tagger.albumReplayGain
      expect(gain === null || typeof gain === 'number').toBe(true)
    })

    it('should set album replay gain', () => {
      tagger.albumReplayGain = -6.0
      expect(tagger.albumReplayGain).toBe(-6.0)
    })

    it('should remove album replay gain when set to null', () => {
      tagger.albumReplayGain = -6.0
      tagger.albumReplayGain = null
      expect(tagger.albumReplayGain).toBeNull()
    })

    it('should get album replay peak', () => {
      const peak = tagger.albumReplayPeak
      expect(peak === null || typeof peak === 'number').toBe(true)
    })

    it('should set album replay peak', () => {
      tagger.albumReplayPeak = 0.98
      expect(tagger.albumReplayPeak).toBe(0.98)
    })

    it('should remove album replay peak when set to null', () => {
      tagger.albumReplayPeak = 0.98
      tagger.albumReplayPeak = null
      expect(tagger.albumReplayPeak).toBeNull()
    })
  })

  describe('pictures', () => {
    it('should get pictures', () => {
      const pictures = tagger.pictures
      expect(pictures === null || Array.isArray(pictures)).toBe(true)
    })

    it('should set pictures', () => {
      const pictureData = new Uint8Array([1, 2, 3, 4, 5])
      const picture = new MetaPicture('image/jpeg', pictureData, 'Cover')
      tagger.pictures = [picture]

      const pictures = tagger.pictures
      expect(pictures).not.toBeNull()
      expect(pictures!.length).toBe(1)
      expect(pictures![0].mimeType).toBe('image/jpeg')
    })

    it('should remove pictures when set to null', () => {
      const pictureData = new Uint8Array([1, 2, 3])
      const picture = new MetaPicture('image/png', pictureData)
      tagger.pictures = [picture]
      tagger.pictures = null
      expect(tagger.pictures).toBeNull()
    })
  })
})
