import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import { describe, it, expect, beforeEach } from 'vitest'

import { MetaPicture, MusicFile } from '../index.js'

import { base } from './const.ts'

const samples = [
  { file: 'mp3.mp3', description: 'MP3' },
  { file: 'flac.flac', description: 'FLAC' },
  { file: 'ogg.opus', description: 'OGG Opus' },
  { file: 'wav.wav', description: 'WAV' },
] as const

for (const sample of samples) {
  describe(`Metadata Getters and Setters [${sample.description}]`, { concurrent: false }, () => {
    let musicFile: MusicFile
    const buffer = readFileSync(join(base, sample.file))

    beforeEach(() => {
      musicFile = MusicFile.loadSync(buffer)
    })

    describe('title', () => {
      it('should get title', () => {
        const title = musicFile.title
        expect(title === null || typeof title === 'string').toBe(true)
      })

      it('should set title', () => {
        musicFile.title = 'Test Title'
        expect(musicFile.title).toBe('Test Title')
      })

      it('should remove title when set to null', () => {
        musicFile.title = 'Test Title'
        musicFile.title = null
        expect(musicFile.title).toBeNull()
      })
    })

    describe('artist', () => {
      it('should get artist', () => {
        const artist = musicFile.artist
        expect(artist === null || typeof artist === 'string').toBe(true)
      })

      it('should set artist', () => {
        musicFile.artist = 'Test Artist'
        expect(musicFile.artist).toBe('Test Artist')
      })

      it('should remove artist when set to null', () => {
        musicFile.artist = 'Test Artist'
        musicFile.artist = null
        expect(musicFile.artist).toBeNull()
      })
    })

    describe('album', () => {
      it('should get album', () => {
        const album = musicFile.album
        expect(album === null || typeof album === 'string').toBe(true)
      })

      it('should set album', () => {
        musicFile.album = 'Test Album'
        expect(musicFile.album).toBe('Test Album')
      })

      it('should remove album when set to null', () => {
        musicFile.album = 'Test Album'
        musicFile.album = null
        expect(musicFile.album).toBeNull()
      })
    })

    describe('year', () => {
      it('should get year', () => {
        const year = musicFile.year
        expect(year === null || typeof year === 'number').toBe(true)
      })

      it('should set year', () => {
        musicFile.year = 2024
        expect(musicFile.year).toBe(2024)
      })

      it('should remove year when set to null', () => {
        musicFile.year = 2024
        musicFile.year = null
        expect(musicFile.year).toBeNull()
      })
    })

    describe('genre', () => {
      it('should get genre', () => {
        const genre = musicFile.genre
        expect(genre === null || typeof genre === 'string').toBe(true)
      })

      it('should set genre', () => {
        musicFile.genre = 'Rock'
        expect(musicFile.genre).toBe('Rock')
      })

      it('should remove genre when set to null', () => {
        musicFile.genre = 'Rock'
        musicFile.genre = null
        expect(musicFile.genre).toBeNull()
      })
    })

    describe('trackNumber', () => {
      it('should get track number', () => {
        const trackNumber = musicFile.trackNumber
        expect(trackNumber === null || typeof trackNumber === 'number').toBe(true)
      })

      it('should set track number', () => {
        musicFile.trackNumber = 1
        expect(musicFile.trackNumber).toBe(1)
      })

      it('should remove track number when set to null', () => {
        musicFile.trackNumber = 1
        musicFile.trackNumber = null
        expect(musicFile.trackNumber).toBeNull()
      })
    })

    describe('discNumber', () => {
      it('should get disc number', () => {
        const discNumber = musicFile.discNumber
        expect(discNumber === null || typeof discNumber === 'number').toBe(true)
      })

      it('should set disc number', () => {
        musicFile.discNumber = 1
        expect(musicFile.discNumber).toBe(1)
      })

      it('should remove disc number when set to null', () => {
        musicFile.discNumber = 1
        musicFile.discNumber = null
        expect(musicFile.discNumber).toBeNull()
      })
    })

    describe('trackTotal', () => {
      it('should get track total', () => {
        const trackTotal = musicFile.trackTotal
        expect(trackTotal === null || typeof trackTotal === 'number').toBe(true)
      })

      it('should set track total', () => {
        musicFile.trackTotal = 12
        expect(musicFile.trackTotal).toBe(12)
      })

      it('should remove track total when set to null', () => {
        musicFile.trackTotal = 12
        musicFile.trackTotal = null
        expect(musicFile.trackTotal).toBeNull()
      })
    })

    describe('discsTotal', () => {
      it('should get discs total', () => {
        const discsTotal = musicFile.discsTotal
        expect(discsTotal === null || typeof discsTotal === 'number').toBe(true)
      })

      it('should set discs total', () => {
        musicFile.discsTotal = 2
        expect(musicFile.discsTotal).toBe(2)
      })

      it('should remove discs total when set to null', () => {
        musicFile.discsTotal = 2
        musicFile.discsTotal = null
        expect(musicFile.discsTotal).toBeNull()
      })
    })

    describe('comment', () => {
      it('should get comment', () => {
        const comment = musicFile.comment
        expect(comment === null || typeof comment === 'string').toBe(true)
      })

      it('should set comment', () => {
        musicFile.comment = 'Test comment'
        expect(musicFile.comment).toBe('Test comment')
      })

      it('should remove comment when set to null', () => {
        musicFile.comment = 'Test comment'
        musicFile.comment = null
        expect(musicFile.comment).toBeNull()
      })
    })

    describe('albumArtist', () => {
      it('should get album artist', () => {
        const albumArtist = musicFile.albumArtist
        expect(albumArtist === null || typeof albumArtist === 'string').toBe(true)
      })

      it('should set album artist', () => {
        musicFile.albumArtist = 'Album Artist'
        expect(musicFile.albumArtist).toBe('Album Artist')
      })

      it('should remove album artist when set to null', () => {
        musicFile.albumArtist = 'Album Artist'
        musicFile.albumArtist = null
        expect(musicFile.albumArtist).toBeNull()
      })
    })

    describe('composer', () => {
      it('should get composer', () => {
        const composer = musicFile.composer
        expect(composer === null || typeof composer === 'string').toBe(true)
      })

      it('should set composer', () => {
        musicFile.composer = 'Composer Name'
        expect(musicFile.composer).toBe('Composer Name')
      })

      it('should remove composer when set to null', () => {
        musicFile.composer = 'Composer Name'
        musicFile.composer = null
        expect(musicFile.composer).toBeNull()
      })
    })

    describe('conductor', () => {
      it('should get conductor', () => {
        const conductor = musicFile.conductor
        expect(conductor === null || typeof conductor === 'string').toBe(true)
      })

      it('should set conductor', () => {
        musicFile.conductor = 'Conductor Name'
        expect(musicFile.conductor).toBe('Conductor Name')
      })

      it('should remove conductor when set to null', () => {
        musicFile.conductor = 'Conductor Name'
        musicFile.conductor = null
        expect(musicFile.conductor).toBeNull()
      })
    })

    describe('lyricist', () => {
      it('should get lyricist', () => {
        const lyricist = musicFile.lyricist
        expect(lyricist === null || typeof lyricist === 'string').toBe(true)
      })

      it('should set lyricist', () => {
        musicFile.lyricist = 'Lyricist Name'
        expect(musicFile.lyricist).toBe('Lyricist Name')
      })

      it('should remove lyricist when set to null', () => {
        musicFile.lyricist = 'Lyricist Name'
        musicFile.lyricist = null
        expect(musicFile.lyricist).toBeNull()
      })
    })

    describe('publisher', () => {
      it('should get publisher', () => {
        const publisher = musicFile.publisher
        expect(publisher === null || typeof publisher === 'string').toBe(true)
      })

      it('should set publisher', () => {
        musicFile.publisher = 'Publisher Name'
        expect(musicFile.publisher).toBe('Publisher Name')
      })

      it('should remove publisher when set to null', () => {
        musicFile.publisher = 'Publisher Name'
        musicFile.publisher = null
        expect(musicFile.publisher).toBeNull()
      })
    })

    describe('lyrics', () => {
      it('should get lyrics', () => {
        const lyrics = musicFile.lyrics
        expect(lyrics === null || typeof lyrics === 'string').toBe(true)
      })

      it('should set lyrics', () => {
        musicFile.lyrics = 'Test lyrics\nLine 2'
        expect(musicFile.lyrics).toBe('Test lyrics\nLine 2')
      })

      it('should remove lyrics when set to null', () => {
        musicFile.lyrics = 'Test lyrics'
        musicFile.lyrics = null
        expect(musicFile.lyrics).toBeNull()
      })
    })

    describe('copyright', () => {
      it('should get copyright', () => {
        const copyright = musicFile.copyright
        expect(copyright === null || typeof copyright === 'string').toBe(true)
      })

      it('should set copyright', () => {
        musicFile.copyright = '© 2024 Test'
        expect(musicFile.copyright).toBe('© 2024 Test')
      })

      it('should remove copyright when set to null', () => {
        musicFile.copyright = '© 2024 Test'
        musicFile.copyright = null
        expect(musicFile.copyright).toBeNull()
      })
    })

    describe('rating', () => {
      it('should get rating', () => {
        const rating = musicFile.rating
        expect(rating === null || typeof rating === 'number').toBe(true)
      })

      it('should set rating', () => {
        musicFile.rating = 3
        expect(musicFile.rating).toBe(3)
      })

      it('should remove rating when set to null', () => {
        musicFile.rating = 3
        musicFile.rating = null
        expect(musicFile.rating).toBeNull()
      })

      it('should throw for out-of-range rating', () => {
        expect(() => {
          musicFile.rating = 6 as any
        }).toThrow()
      })
    })

    describe('replay gain', () => {
      it('should get track replay gain', () => {
        const gain = musicFile.trackReplayGain
        expect(gain === null || typeof gain === 'number').toBe(true)
      })

      it('should set track replay gain', () => {
        musicFile.trackReplayGain = -5.5
        expect(musicFile.trackReplayGain).toBe(-5.5)
      })

      it('should remove track replay gain when set to null', () => {
        musicFile.trackReplayGain = -5.5
        musicFile.trackReplayGain = null
        expect(musicFile.trackReplayGain).toBeNull()
      })

      it('should get track replay peak', () => {
        const peak = musicFile.trackReplayPeak
        expect(peak === null || typeof peak === 'number').toBe(true)
      })

      it('should set track replay peak', () => {
        musicFile.trackReplayPeak = 0.95
        expect(musicFile.trackReplayPeak).toBe(0.95)
      })

      it('should remove track replay peak when set to null', () => {
        musicFile.trackReplayPeak = 0.95
        musicFile.trackReplayPeak = null
        expect(musicFile.trackReplayPeak).toBeNull()
      })

      it('should get album replay gain', () => {
        const gain = musicFile.albumReplayGain
        expect(gain === null || typeof gain === 'number').toBe(true)
      })

      it('should set album replay gain', () => {
        musicFile.albumReplayGain = -6.0
        expect(musicFile.albumReplayGain).toBe(-6.0)
      })

      it('should remove album replay gain when set to null', () => {
        musicFile.albumReplayGain = -6.0
        musicFile.albumReplayGain = null
        expect(musicFile.albumReplayGain).toBeNull()
      })

      it('should get album replay peak', () => {
        const peak = musicFile.albumReplayPeak
        expect(peak === null || typeof peak === 'number').toBe(true)
      })

      it('should set album replay peak', () => {
        musicFile.albumReplayPeak = 0.98
        expect(musicFile.albumReplayPeak).toBe(0.98)
      })

      it('should remove album replay peak when set to null', () => {
        musicFile.albumReplayPeak = 0.98
        musicFile.albumReplayPeak = null
        expect(musicFile.albumReplayPeak).toBeNull()
      })
    })

    describe('pictures', () => {
      it('should get pictures', () => {
        const pictures = musicFile.pictures
        expect(pictures === null || Array.isArray(pictures)).toBe(true)
      })

      it('should set pictures', () => {
        const pictureData = new Uint8Array([1, 2, 3, 4, 5])
        const picture = new MetaPicture('image/jpeg', pictureData, 'Cover')
        musicFile.pictures = [picture]

        const pictures = musicFile.pictures
        expect(pictures).not.toBeNull()
        expect(pictures!.length).toBe(1)
        expect(pictures![0]!.mimeType).toBe('image/jpeg')
      })

      it('should remove pictures when set to null', () => {
        const pictureData = new Uint8Array([1, 2, 3])
        const picture = new MetaPicture('image/png', pictureData)
        musicFile.pictures = [picture]
        musicFile.pictures = null
        expect(musicFile.pictures).toBeNull()
      })
    })
  })
}

/** Build a 128-byte ID3v1 tag */
function id3v1Tag(fields: { title: string; artist: string }): Uint8Array {
  const tag = new Uint8Array(128)
  const encoder = new TextEncoder()
  tag.set(encoder.encode('TAG'), 0)
  tag.set(encoder.encode(fields.title).subarray(0, 30), 3)
  tag.set(encoder.encode(fields.artist).subarray(0, 30), 33)
  tag[127] = 255 // no genre
  return tag
}

describe('MP3 with only an ID3v1 tag', () => {
  const audio = readFileSync(join(base, 'mp3-no-tags.mp3'))
  const source = new Uint8Array(audio.length + 128)
  source.set(audio, 0)
  source.set(id3v1Tag({ title: 'V1 Title', artist: 'V1 Artist' }), audio.length)

  it('reads the ID3v1 tag', () => {
    const musicFile = MusicFile.loadSync(source)
    expect(musicFile.tagType).toBe('ID3V1')
    expect(musicFile.title).toBe('V1 Title')
  })

  it('writes fields ID3v1 cannot store into a new ID3v2 tag', () => {
    const musicFile = MusicFile.loadSync(source)
    musicFile.composer = 'Composer Name'
    musicFile.lyrics = 'Lyrics line'

    expect(musicFile.tagType).toBe('ID3V2')
    expect(musicFile.composer).toBe('Composer Name')
    expect(musicFile.lyrics).toBe('Lyrics line')
    // Existing values are carried over from the ID3v1 tag
    expect(musicFile.title).toBe('V1 Title')
    expect(musicFile.artist).toBe('V1 Artist')
  })

  it('persists the new ID3v2 tag through save and reload', () => {
    const musicFile = MusicFile.loadSync(source)
    musicFile.composer = 'Composer Name'

    const reloaded = MusicFile.loadSync(musicFile.saveSync(source.slice()) as Uint8Array)

    expect(reloaded.tagType).toBe('ID3V2')
    expect(reloaded.composer).toBe('Composer Name')
    expect(reloaded.title).toBe('V1 Title')
  })

  it('persists pictures through save and reload', () => {
    const musicFile = MusicFile.loadSync(source)
    musicFile.pictures = [new MetaPicture('image/png', new Uint8Array([1, 2, 3, 4]))]

    const reloaded = MusicFile.loadSync(musicFile.saveSync(source.slice()) as Uint8Array)

    expect(reloaded.tagType).toBe('ID3V2')
    expect(reloaded.pictures).toHaveLength(1)
    expect(reloaded.title).toBe('V1 Title')
  })

  it('does not truncate a title longer than 30 bytes', () => {
    const title = 'A title that is longer than thirty bytes'
    const musicFile = MusicFile.loadSync(source)
    musicFile.title = title

    const reloaded = MusicFile.loadSync(musicFile.saveSync(source.slice()) as Uint8Array)

    expect(reloaded.tagType).toBe('ID3V2')
    expect(reloaded.title).toBe(title)
  })

  it('updates the ID3v1 tag too', () => {
    const musicFile = MusicFile.loadSync(source)
    musicFile.title = 'New Title'

    const saved = musicFile.saveSync(source.slice()) as Uint8Array
    const v1Title = new TextDecoder()
      .decode(saved.subarray(saved.length - 125, saved.length - 95))
      .replace(/\0+$/, '')

    expect(v1Title).toBe('New Title')
    expect(MusicFile.loadSync(saved).title).toBe('New Title')
  })

  it('keeps a removed field removed after save and reload', () => {
    const musicFile = MusicFile.loadSync(source)
    musicFile.title = null

    const reloaded = MusicFile.loadSync(musicFile.saveSync(source.slice()) as Uint8Array)

    expect(reloaded.title).toBeNull()
    expect(reloaded.artist).toBe('V1 Artist')
  })

  it('does not add an ID3v2 tag when only clearing fields', () => {
    const musicFile = MusicFile.loadSync(source)
    musicFile.rating = null
    musicFile.composer = null
    musicFile.pictures = null
    musicFile.title = null

    expect(musicFile.tagType).toBe('ID3V1')
    const saved = musicFile.saveSync(source.slice()) as Uint8Array
    expect(new TextDecoder().decode(saved.subarray(0, 3))).not.toBe('ID3')
    expect(MusicFile.loadSync(saved).artist).toBe('V1 Artist')
  })

  it('updates the ID3v1 year too', () => {
    const musicFile = MusicFile.loadSync(source)
    musicFile.year = 2024

    const saved = musicFile.saveSync(source.slice()) as Uint8Array
    const v1Year = new TextDecoder().decode(saved.subarray(saved.length - 35, saved.length - 31))

    expect(v1Year).toBe('2024')
    expect(MusicFile.loadSync(saved).year).toBe(2024)
  })
})

/** Build an APEv2 tag (footer only) from key/value pairs */
function apeTag(items: [key: string, value: string][]): Uint8Array {
  const encoder = new TextEncoder()
  const parts = items.map(([key, value]) => {
    const keyBytes = encoder.encode(`${key}\0`)
    const valueBytes = encoder.encode(value)
    const item = new Uint8Array(8 + keyBytes.length + valueBytes.length)
    new DataView(item.buffer).setUint32(0, valueBytes.length, true)
    item.set(keyBytes, 8)
    item.set(valueBytes, 8 + keyBytes.length)
    return item
  })
  const bodyLength = parts.reduce((sum, part) => sum + part.length, 0)
  const tag = new Uint8Array(bodyLength + 32)
  let offset = 0
  for (const part of parts) {
    tag.set(part, offset)
    offset += part.length
  }
  const footer = new DataView(tag.buffer, bodyLength)
  tag.set(encoder.encode('APETAGEX'), bodyLength)
  footer.setUint32(8, 2000, true) // version
  footer.setUint32(12, bodyLength + 32, true) // tag size without header
  footer.setUint32(16, items.length, true)
  return tag
}

describe('MP3 with ID3v1 and APE tags but no ID3v2', () => {
  const audio = readFileSync(join(base, 'mp3-no-tags.mp3'))
  const ape = apeTag([
    ['Title', 'APE Title'],
    ['Composer', 'APE Composer'],
  ])
  const v1 = id3v1Tag({ title: 'V1 Title', artist: 'V1 Artist' })
  const source = new Uint8Array(audio.length + ape.length + v1.length)
  source.set(audio, 0)
  source.set(ape, audio.length)
  source.set(v1, audio.length + ape.length)

  it('reads the APE tag instead of ID3v1', () => {
    const musicFile = MusicFile.loadSync(source)

    expect(musicFile.tagType).toBe('APE')
    expect(musicFile.title).toBe('APE Title')
    expect(musicFile.composer).toBe('APE Composer')
  })

  it('writes to the APE tag when it can store the field', () => {
    const musicFile = MusicFile.loadSync(source)
    musicFile.title = 'New Title'

    const reloaded = MusicFile.loadSync(musicFile.saveSync(source.slice()) as Uint8Array)

    expect(reloaded.tagType).toBe('APE')
    expect(reloaded.title).toBe('New Title')
  })

  it('seeds a new ID3v2 tag from the APE tag', () => {
    const musicFile = MusicFile.loadSync(source)
    // APE cannot store a rating, so an ID3v2 tag is created.
    musicFile.rating = 4

    const reloaded = MusicFile.loadSync(musicFile.saveSync(source.slice()) as Uint8Array)

    expect(reloaded.tagType).toBe('ID3V2')
    expect(reloaded.rating).toBe(4)
    expect(reloaded.title).toBe('APE Title')
    expect(reloaded.composer).toBe('APE Composer')
  })
})
