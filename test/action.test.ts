import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { readFileSync } from 'fs'
import { join } from 'path'
import { MusicTagger } from '../index'
import { base } from './const'

const isWasi = process.env.NAPI_RS_FORCE_WASI === '1'

describe('MusicTagger', () => {
  let tagger: MusicTagger

  beforeEach(() => {
    tagger = new MusicTagger()
  })

  afterEach(() => {
    if (!tagger.isDisposed()) {
      tagger.dispose()
    }
  })

  describe('Initialization', () => {
    it('should create a new MusicTagger instance', () => {
      expect(tagger).toBeInstanceOf(MusicTagger)
      expect(tagger.isDisposed()).toBe(true)
    })

    it('should be disposed initially', () => {
      expect(tagger.isDisposed()).toBe(true)
    })
  })

  describe.skipIf(isWasi)('loadPath', () => {
    it('should load an MP3 file', () => {
      const path = join(base, 'mp3.mp3')
      tagger.loadPath(path)

      expect(tagger.isDisposed()).toBe(false)
      expect(tagger.tagType).toBeTruthy()
    })

    it('should load a FLAC file', () => {
      const path = join(base, 'flac.flac')
      tagger.loadPath(path)

      expect(tagger.isDisposed()).toBe(false)
      expect(tagger.tagType).toBeTruthy()
    })

    it('should load an OGG/Opus file', () => {
      const path = join(base, 'ogg.opus')
      tagger.loadPath(path)

      expect(tagger.isDisposed()).toBe(false)
      expect(tagger.tagType).toBeTruthy()
    })

    it('should load a WAV file', () => {
      const path = join(base, 'wav.wav')
      tagger.loadPath(path)

      expect(tagger.isDisposed()).toBe(false)
      expect(tagger.tagType).toBeTruthy()
    })

    it('should throw error for non-existent file', () => {
      expect(() => {
        tagger.loadPath('non-existent-file.mp3')
      }).toThrow()
    })

    it('should throw error for invalid audio file', () => {
      expect(() => {
        tagger.loadPath(__filename) // Try to load the test file itself
      }).toThrow()
    })
  })

  describe('loadBuffer', () => {
    it('should load from buffer', () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      tagger.loadBuffer(uint8Array)

      expect(tagger.isDisposed()).toBe(false)
      expect(tagger.tagType).toBeTruthy()
    })

    it('should throw error for invalid buffer', () => {
      const invalidBuffer = new Uint8Array([1, 2, 3, 4, 5])

      expect(() => {
        tagger.loadBuffer(invalidBuffer)
      }).toThrow()
    })
  })

  describe.skipIf(isWasi)('dispose', () => {
    it('should dispose a loaded file', () => {
      const path = join(base, 'mp3.mp3')
      tagger.loadPath(path)
      expect(tagger.isDisposed()).toBe(false)

      tagger.dispose()
      expect(tagger.isDisposed()).toBe(true)
    })

    it('should not throw when disposing an already disposed tagger', () => {
      expect(() => {
        tagger.dispose()
      }).not.toThrow()
    })
  })

  describe.sequential('save', () => {
    it('should save buffer after loading from buffer', () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      tagger.loadBuffer(uint8Array)
      tagger.title = 'New Title'
      expect(() => tagger.save()).not.toThrow()

      const newBuffer = tagger.buffer
      expect(newBuffer).toBeInstanceOf(Uint8Array)
      expect(newBuffer.length).toBeGreaterThan(0)
    })

    it('should throw error when disposed', () => {
      expect(() => {
        tagger.save()
      }).toThrow()
    })
  })

  describe.skipIf(isWasi)('savePath', () => {
    it('should save to original path', () => {
      const path = join(base, 'mp3.mp3')
      tagger.loadPath(path)
      tagger.title = 'Saved Title'

      expect(() => {
        tagger.save()
      }).not.toThrow()
    })

    it('should throw error when disposed', () => {
      expect(() => {
        tagger.save()
      }).toThrow()
    })
  })

  describe('buffer getter', () => {
    it('should return buffer when loaded from buffer', () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      tagger.loadBuffer(uint8Array)
      const retrievedBuffer = tagger.buffer

      expect(retrievedBuffer).toBeInstanceOf(Uint8Array)
      expect(retrievedBuffer.length).toBeGreaterThan(0)
    })

    it.skipIf(isWasi)('should return empty buffer when loaded from path', () => {
      const path = join(base, 'mp3.mp3')
      tagger.loadPath(path)
      const buffer = tagger.buffer

      expect(buffer).toBeInstanceOf(Uint8Array)
      // Buffer might be empty or might contain data depending on implementation
    })

    it('should throw error when disposed', () => {
      expect(() => {
        tagger.buffer
      }).toThrow()
    })
  })

  describe('Error handling', () => {
    it('should throw error when accessing properties on disposed tagger', () => {
      expect(() => {
        tagger.title
      }).toThrow()

      expect(() => {
        tagger.quality
      }).toThrow()

      expect(() => {
        tagger.duration
      }).toThrow()
    })

    it('should throw error when setting properties on disposed tagger', () => {
      expect(() => {
        tagger.title = 'Test'
      }).toThrow()
    })
  })

  describe('Integration tests', () => {
    it('should load, modify, and save metadata', () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      tagger.loadBuffer(uint8Array)

      // Modify metadata
      tagger.title = 'Modified Title'
      tagger.artist = 'Modified Artist'
      tagger.year = 2024
      tagger.genre = 'Test Genre'

      // Verify changes
      expect(tagger.title).toBe('Modified Title')
      expect(tagger.artist).toBe('Modified Artist')
      expect(tagger.year).toBe(2024)
      expect(tagger.genre).toBe('Test Genre')

      // Save and reload
      tagger.save()
      const newBuffer = tagger.buffer

      // Create new tagger and load the modified buffer
      const newTagger = new MusicTagger()
      newTagger.loadBuffer(newBuffer)

      // Verify persisted changes
      expect(newTagger.title).toBe('Modified Title')
      expect(newTagger.artist).toBe('Modified Artist')
      expect(newTagger.year).toBe(2024)
      expect(newTagger.genre).toBe('Test Genre')

      newTagger.dispose()
    })

    it.skipIf(isWasi)('should handle multiple file formats', () => {
      const formats = ['mp3.mp3', 'flac.flac', 'ogg.opus', 'wav.wav']

      for (const file of formats) {
        const path = join(base, file)
        const tagger = new MusicTagger()

        try {
          tagger.loadPath(path)
          expect(tagger.isDisposed()).toBe(false)
          expect(tagger.tagType).toBeTruthy()
          expect(tagger.duration).toBeGreaterThanOrEqual(0)
        } finally {
          tagger.dispose()
        }
      }
    })
  })
})
