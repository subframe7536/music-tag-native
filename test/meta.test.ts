import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import { describe, it, expect, beforeEach } from 'vitest'

import { MetaPicture, TaggedFile } from '../index'

import { base } from './const'

const samples = [
  { file: 'mp3.mp3', description: 'MP3' },
  { file: 'flac.flac', description: 'FLAC' },
  { file: 'ogg.opus', description: 'OGG Opus' },
  { file: 'wav.wav', description: 'WAV' },
] as const

for (const sample of samples) {
  describe.sequential(`Metadata Getters and Setters [${sample.description}]`, () => {
    let taggedFile: TaggedFile
    const buffer = readFileSync(join(base, sample.file))

    beforeEach(() => {
      taggedFile = TaggedFile.loadSync(buffer)
    })

    describe('title', () => {
      it('should get title', () => {
        const title = taggedFile.title
        expect(title === null || typeof title === 'string').toBe(true)
      })

      it('should set title', () => {
        taggedFile.title = 'Test Title'
        expect(taggedFile.title).toBe('Test Title')
      })

      it('should remove title when set to null', () => {
        taggedFile.title = 'Test Title'
        taggedFile.title = null
        expect(taggedFile.title).toBeNull()
      })
    })

    describe('artist', () => {
      it('should get artist', () => {
        const artist = taggedFile.artist
        expect(artist === null || typeof artist === 'string').toBe(true)
      })

      it('should set artist', () => {
        taggedFile.artist = 'Test Artist'
        expect(taggedFile.artist).toBe('Test Artist')
      })

      it('should remove artist when set to null', () => {
        taggedFile.artist = 'Test Artist'
        taggedFile.artist = null
        expect(taggedFile.artist).toBeNull()
      })
    })

    describe('album', () => {
      it('should get album', () => {
        const album = taggedFile.album
        expect(album === null || typeof album === 'string').toBe(true)
      })

      it('should set album', () => {
        taggedFile.album = 'Test Album'
        expect(taggedFile.album).toBe('Test Album')
      })

      it('should remove album when set to null', () => {
        taggedFile.album = 'Test Album'
        taggedFile.album = null
        expect(taggedFile.album).toBeNull()
      })
    })

    describe('year', () => {
      it('should get year', () => {
        const year = taggedFile.year
        expect(year === null || typeof year === 'number').toBe(true)
      })

      it('should set year', () => {
        taggedFile.year = 2024
        expect(taggedFile.year).toBe(2024)
      })

      it('should remove year when set to null', () => {
        taggedFile.year = 2024
        taggedFile.year = null
        expect(taggedFile.year).toBeNull()
      })
    })

    describe('genre', () => {
      it('should get genre', () => {
        const genre = taggedFile.genre
        expect(genre === null || typeof genre === 'string').toBe(true)
      })

      it('should set genre', () => {
        taggedFile.genre = 'Rock'
        expect(taggedFile.genre).toBe('Rock')
      })

      it('should remove genre when set to null', () => {
        taggedFile.genre = 'Rock'
        taggedFile.genre = null
        expect(taggedFile.genre).toBeNull()
      })
    })

    describe('trackNumber', () => {
      it('should get track number', () => {
        const trackNumber = taggedFile.trackNumber
        expect(trackNumber === null || typeof trackNumber === 'number').toBe(true)
      })

      it('should set track number', () => {
        taggedFile.trackNumber = 1
        expect(taggedFile.trackNumber).toBe(1)
      })

      it('should remove track number when set to null', () => {
        taggedFile.trackNumber = 1
        taggedFile.trackNumber = null
        expect(taggedFile.trackNumber).toBeNull()
      })
    })

    describe('discNumber', () => {
      it('should get disc number', () => {
        const discNumber = taggedFile.discNumber
        expect(discNumber === null || typeof discNumber === 'number').toBe(true)
      })

      it('should set disc number', () => {
        taggedFile.discNumber = 1
        expect(taggedFile.discNumber).toBe(1)
      })

      it('should remove disc number when set to null', () => {
        taggedFile.discNumber = 1
        taggedFile.discNumber = null
        expect(taggedFile.discNumber).toBeNull()
      })
    })

    describe('trackTotal', () => {
      it('should get track total', () => {
        const trackTotal = taggedFile.trackTotal
        expect(trackTotal === null || typeof trackTotal === 'number').toBe(true)
      })

      it('should set track total', () => {
        taggedFile.trackTotal = 12
        expect(taggedFile.trackTotal).toBe(12)
      })

      it('should remove track total when set to null', () => {
        taggedFile.trackTotal = 12
        taggedFile.trackTotal = null
        expect(taggedFile.trackTotal).toBeNull()
      })
    })

    describe('discsTotal', () => {
      it('should get discs total', () => {
        const discsTotal = taggedFile.discsTotal
        expect(discsTotal === null || typeof discsTotal === 'number').toBe(true)
      })

      it('should set discs total', () => {
        taggedFile.discsTotal = 2
        expect(taggedFile.discsTotal).toBe(2)
      })

      it('should remove discs total when set to null', () => {
        taggedFile.discsTotal = 2
        taggedFile.discsTotal = null
        expect(taggedFile.discsTotal).toBeNull()
      })
    })

    describe('comment', () => {
      it('should get comment', () => {
        const comment = taggedFile.comment
        expect(comment === null || typeof comment === 'string').toBe(true)
      })

      it('should set comment', () => {
        taggedFile.comment = 'Test comment'
        expect(taggedFile.comment).toBe('Test comment')
      })

      it('should remove comment when set to null', () => {
        taggedFile.comment = 'Test comment'
        taggedFile.comment = null
        expect(taggedFile.comment).toBeNull()
      })
    })

    describe('albumArtist', () => {
      it('should get album artist', () => {
        const albumArtist = taggedFile.albumArtist
        expect(albumArtist === null || typeof albumArtist === 'string').toBe(true)
      })

      it('should set album artist', () => {
        taggedFile.albumArtist = 'Album Artist'
        expect(taggedFile.albumArtist).toBe('Album Artist')
      })

      it('should remove album artist when set to null', () => {
        taggedFile.albumArtist = 'Album Artist'
        taggedFile.albumArtist = null
        expect(taggedFile.albumArtist).toBeNull()
      })
    })

    describe('composer', () => {
      it('should get composer', () => {
        const composer = taggedFile.composer
        expect(composer === null || typeof composer === 'string').toBe(true)
      })

      it('should set composer', () => {
        taggedFile.composer = 'Composer Name'
        expect(taggedFile.composer).toBe('Composer Name')
      })

      it('should remove composer when set to null', () => {
        taggedFile.composer = 'Composer Name'
        taggedFile.composer = null
        expect(taggedFile.composer).toBeNull()
      })
    })

    describe('conductor', () => {
      it('should get conductor', () => {
        const conductor = taggedFile.conductor
        expect(conductor === null || typeof conductor === 'string').toBe(true)
      })

      it('should set conductor', () => {
        taggedFile.conductor = 'Conductor Name'
        expect(taggedFile.conductor).toBe('Conductor Name')
      })

      it('should remove conductor when set to null', () => {
        taggedFile.conductor = 'Conductor Name'
        taggedFile.conductor = null
        expect(taggedFile.conductor).toBeNull()
      })
    })

    describe('lyricist', () => {
      it('should get lyricist', () => {
        const lyricist = taggedFile.lyricist
        expect(lyricist === null || typeof lyricist === 'string').toBe(true)
      })

      it('should set lyricist', () => {
        taggedFile.lyricist = 'Lyricist Name'
        expect(taggedFile.lyricist).toBe('Lyricist Name')
      })

      it('should remove lyricist when set to null', () => {
        taggedFile.lyricist = 'Lyricist Name'
        taggedFile.lyricist = null
        expect(taggedFile.lyricist).toBeNull()
      })
    })

    describe('publisher', () => {
      it('should get publisher', () => {
        const publisher = taggedFile.publisher
        expect(publisher === null || typeof publisher === 'string').toBe(true)
      })

      it('should set publisher', () => {
        taggedFile.publisher = 'Publisher Name'
        expect(taggedFile.publisher).toBe('Publisher Name')
      })

      it('should remove publisher when set to null', () => {
        taggedFile.publisher = 'Publisher Name'
        taggedFile.publisher = null
        expect(taggedFile.publisher).toBeNull()
      })
    })

    describe('lyrics', () => {
      it('should get lyrics', () => {
        const lyrics = taggedFile.lyrics
        expect(lyrics === null || typeof lyrics === 'string').toBe(true)
      })

      it('should set lyrics', () => {
        taggedFile.lyrics = 'Test lyrics\nLine 2'
        expect(taggedFile.lyrics).toBe('Test lyrics\nLine 2')
      })

      it('should remove lyrics when set to null', () => {
        taggedFile.lyrics = 'Test lyrics'
        taggedFile.lyrics = null
        expect(taggedFile.lyrics).toBeNull()
      })
    })

    describe('copyright', () => {
      it('should get copyright', () => {
        const copyright = taggedFile.copyright
        expect(copyright === null || typeof copyright === 'string').toBe(true)
      })

      it('should set copyright', () => {
        taggedFile.copyright = '© 2024 Test'
        expect(taggedFile.copyright).toBe('© 2024 Test')
      })

      it('should remove copyright when set to null', () => {
        taggedFile.copyright = '© 2024 Test'
        taggedFile.copyright = null
        expect(taggedFile.copyright).toBeNull()
      })
    })

    describe('rating', () => {
      it('should get rating', () => {
        const rating = taggedFile.rating
        expect(rating === null || typeof rating === 'number').toBe(true)
      })

      it('should set rating', () => {
        taggedFile.rating = 3
        expect(taggedFile.rating).toBe(3)
      })

      it('should remove rating when set to null', () => {
        taggedFile.rating = 3
        taggedFile.rating = null
        expect(taggedFile.rating).toBeNull()
      })

      it('should throw for out-of-range rating', () => {
        expect(() => {
          taggedFile.rating = 6 as any
        }).toThrow()
      })
    })

    describe('replay gain', () => {
      it('should get track replay gain', () => {
        const gain = taggedFile.trackReplayGain
        expect(gain === null || typeof gain === 'number').toBe(true)
      })

      it('should set track replay gain', () => {
        taggedFile.trackReplayGain = -5.5
        expect(taggedFile.trackReplayGain).toBe(-5.5)
      })

      it('should remove track replay gain when set to null', () => {
        taggedFile.trackReplayGain = -5.5
        taggedFile.trackReplayGain = null
        expect(taggedFile.trackReplayGain).toBeNull()
      })

      it('should get track replay peak', () => {
        const peak = taggedFile.trackReplayPeak
        expect(peak === null || typeof peak === 'number').toBe(true)
      })

      it('should set track replay peak', () => {
        taggedFile.trackReplayPeak = 0.95
        expect(taggedFile.trackReplayPeak).toBe(0.95)
      })

      it('should remove track replay peak when set to null', () => {
        taggedFile.trackReplayPeak = 0.95
        taggedFile.trackReplayPeak = null
        expect(taggedFile.trackReplayPeak).toBeNull()
      })

      it('should get album replay gain', () => {
        const gain = taggedFile.albumReplayGain
        expect(gain === null || typeof gain === 'number').toBe(true)
      })

      it('should set album replay gain', () => {
        taggedFile.albumReplayGain = -6.0
        expect(taggedFile.albumReplayGain).toBe(-6.0)
      })

      it('should remove album replay gain when set to null', () => {
        taggedFile.albumReplayGain = -6.0
        taggedFile.albumReplayGain = null
        expect(taggedFile.albumReplayGain).toBeNull()
      })

      it('should get album replay peak', () => {
        const peak = taggedFile.albumReplayPeak
        expect(peak === null || typeof peak === 'number').toBe(true)
      })

      it('should set album replay peak', () => {
        taggedFile.albumReplayPeak = 0.98
        expect(taggedFile.albumReplayPeak).toBe(0.98)
      })

      it('should remove album replay peak when set to null', () => {
        taggedFile.albumReplayPeak = 0.98
        taggedFile.albumReplayPeak = null
        expect(taggedFile.albumReplayPeak).toBeNull()
      })
    })

    describe('pictures', () => {
      it('should get pictures', () => {
        const pictures = taggedFile.pictures
        expect(pictures === null || Array.isArray(pictures)).toBe(true)
      })

      it('should set pictures', () => {
        const pictureData = new Uint8Array([1, 2, 3, 4, 5])
        const picture = new MetaPicture('image/jpeg', pictureData, 'Cover')
        taggedFile.pictures = [picture]

        const pictures = taggedFile.pictures
        expect(pictures).not.toBeNull()
        expect(pictures!.length).toBe(1)
        expect(pictures![0]!.mimeType).toBe('image/jpeg')
      })

      it('should remove pictures when set to null', () => {
        const pictureData = new Uint8Array([1, 2, 3])
        const picture = new MetaPicture('image/png', pictureData)
        taggedFile.pictures = [picture]
        taggedFile.pictures = null
        expect(taggedFile.pictures).toBeNull()
      })
    })
  })
}
