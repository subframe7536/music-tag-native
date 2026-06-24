import { copyFileSync, readFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { describe, it, expect } from 'vitest'

import { TaggedFile } from '../index'

import { base } from './const'

const isWasi = process.env.NAPI_RS_FORCE_WASI === '1'

describe('TaggedFile', () => {
  describe.skipIf(isWasi)('load', () => {
    it('should load an MP3 file', async () => {
      const path = join(base, 'mp3.mp3')
      const taggedFile = await TaggedFile.load(path)

      expect(taggedFile.tagType).toBeTruthy()
    })

    it('should load a FLAC file', async () => {
      const path = join(base, 'flac.flac')
      const taggedFile = await TaggedFile.load(path)

      expect(taggedFile.tagType).toBeTruthy()
    })

    it('should load an OGG/Opus file', async () => {
      const path = join(base, 'ogg.opus')
      const taggedFile = await TaggedFile.load(path)

      expect(taggedFile.tagType).toBeTruthy()
    })

    it('should load a WAV file', async () => {
      const path = join(base, 'wav.wav')
      const taggedFile = await TaggedFile.load(path)

      expect(taggedFile.tagType).toBeTruthy()
    })

    it('should throw error for non-existent file', async () => {
      await expect(async () => {
        await TaggedFile.load('non-existent-file.mp3')
      }).rejects.toThrow()
    })

    it('should throw error for invalid audio file', async () => {
      await expect(async () => {
        await TaggedFile.load(__filename) // Try to load the test file itself
      }).rejects.toThrow()
    })
  })

  describe.skipIf(isWasi)('loadSync', () => {
    it('should load an MP3 file', () => {
      const path = join(base, 'mp3.mp3')
      const taggedFile = TaggedFile.loadSync(path)

      expect(taggedFile.tagType).toBeTruthy()
    })

    it('should load a FLAC file', () => {
      const path = join(base, 'flac.flac')
      const taggedFile = TaggedFile.loadSync(path)

      expect(taggedFile.tagType).toBeTruthy()
    })

    it('should load an OGG/Opus file', () => {
      const path = join(base, 'ogg.opus')
      const taggedFile = TaggedFile.loadSync(path)

      expect(taggedFile.tagType).toBeTruthy()
    })

    it('should load a WAV file', () => {
      const path = join(base, 'wav.wav')
      const taggedFile = TaggedFile.loadSync(path)

      expect(taggedFile.tagType).toBeTruthy()
    })

    it('should throw error for non-existent file', () => {
      expect(() => {
        TaggedFile.loadSync('non-existent-file.mp3')
      }).toThrow()
    })

    it('should throw error for invalid audio file', () => {
      expect(() => {
        TaggedFile.loadSync(__filename) // Try to load the test file itself
      }).toThrow()
    })
  })

  describe('loadSync', () => {
    it('should load from buffer', () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      const taggedFile = TaggedFile.loadSync(uint8Array)

      expect(taggedFile.tagType).toBeTruthy()
    })

    it('should throw error for invalid buffer', () => {
      const invalidBuffer = new Uint8Array([1, 2, 3, 4, 5])

      expect(() => {
        TaggedFile.loadSync(invalidBuffer)
      }).toThrow()
    })
  })

  describe.sequential('save', () => {
    it('should save buffer after loading from buffer', async () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      const taggedFile = TaggedFile.loadSync(uint8Array)
      taggedFile.title = 'New Title'
      const newBuffer = (await taggedFile.save(uint8Array)) as Uint8Array

      expect(newBuffer).toBeInstanceOf(Uint8Array)
      expect(newBuffer.length).toBeGreaterThan(0)
    })
  })

  describe.sequential('saveSync', () => {
    it('should save buffer after loading from buffer', () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      const taggedFile = TaggedFile.loadSync(uint8Array)
      taggedFile.title = 'New Title'
      const newBuffer = taggedFile.saveSync(uint8Array) as Uint8Array

      expect(newBuffer).toBeInstanceOf(Uint8Array)
      expect(newBuffer.length).toBeGreaterThan(0)
    })
  })

  describe.skipIf(isWasi)('savePath', () => {
    it('should save to original path', async () => {
      const sourcePath = join(base, 'mp3.mp3')
      const path = join(tmpdir(), `music-tag-native-save-original-${Date.now()}.mp3`)
      copyFileSync(sourcePath, path)

      try {
        const taggedFile = await TaggedFile.load(path)
        taggedFile.title = 'Saved Title'

        await taggedFile.save()

        const verifyTaggedFile = await TaggedFile.load(path)
        expect(verifyTaggedFile.title).toBe('Saved Title')
      } finally {
        rmSync(path, { force: true })
      }
    })

    it('should save to custom path', async () => {
      const path = join(base, 'mp3.mp3')
      const targetPath = join(tmpdir(), `music-tag-native-save-${Date.now()}.mp3`)
      const taggedFile = await TaggedFile.load(path)
      taggedFile.title = 'Saved Custom Path Title'

      await taggedFile.save(targetPath)

      try {
        const newTaggedFile = await TaggedFile.load(targetPath)
        expect(newTaggedFile.title).toBe('Saved Custom Path Title')
      } finally {
        rmSync(targetPath, { force: true })
      }
    })
  })

  describe.skipIf(isWasi)('savePathSync', () => {
    it('should save to original path', () => {
      const sourcePath = join(base, 'mp3.mp3')
      const path = join(tmpdir(), `music-tag-native-save-original-${Date.now()}.mp3`)
      copyFileSync(sourcePath, path)

      try {
        const taggedFile = TaggedFile.loadSync(path)
        taggedFile.title = 'Saved Title'

        taggedFile.saveSync()

        const verifyTaggedFile = TaggedFile.loadSync(path)
        expect(verifyTaggedFile.title).toBe('Saved Title')
      } finally {
        rmSync(path, { force: true })
      }
    })

    it('should save to custom path', () => {
      const path = join(base, 'mp3.mp3')
      const targetPath = join(tmpdir(), `music-tag-native-save-${Date.now()}.mp3`)
      const taggedFile = TaggedFile.loadSync(path)
      taggedFile.title = 'Saved Custom Path Title'

      taggedFile.saveSync(targetPath)

      try {
        const newTaggedFile = TaggedFile.loadSync(targetPath)
        expect(newTaggedFile.title).toBe('Saved Custom Path Title')
      } finally {
        rmSync(targetPath, { force: true })
      }
    })
  })

  describe('Integration tests', () => {
    it('async should load, modify, and save metadata', async () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      const taggedFile = TaggedFile.loadSync(uint8Array)

      // Modify metadata
      taggedFile.title = 'Modified Title'
      taggedFile.artist = 'Modified Artist'
      taggedFile.year = 2024
      taggedFile.genre = 'Test Genre'

      // Verify changes
      expect(taggedFile.title).toBe('Modified Title')
      expect(taggedFile.artist).toBe('Modified Artist')
      expect(taggedFile.year).toBe(2024)
      expect(taggedFile.genre).toBe('Test Genre')

      // Save and reload
      const newBuffer = (await taggedFile.save(uint8Array)) as Uint8Array

      // Create new tagger and load the modified buffer
      const newTaggedFile = TaggedFile.loadSync(newBuffer)

      // Verify persisted changes
      expect(newTaggedFile.title).toBe('Modified Title')
      expect(newTaggedFile.artist).toBe('Modified Artist')
      expect(newTaggedFile.year).toBe(2024)
      expect(newTaggedFile.genre).toBe('Test Genre')
    })

    it('sync should load, modify, and save metadata', () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      const taggedFile = TaggedFile.loadSync(uint8Array)

      // Modify metadata
      taggedFile.title = 'Modified Title'
      taggedFile.artist = 'Modified Artist'
      taggedFile.year = 2024
      taggedFile.genre = 'Test Genre'

      // Verify changes
      expect(taggedFile.title).toBe('Modified Title')
      expect(taggedFile.artist).toBe('Modified Artist')
      expect(taggedFile.year).toBe(2024)
      expect(taggedFile.genre).toBe('Test Genre')

      // Save and reload
      const newBuffer = taggedFile.saveSync(uint8Array) as Uint8Array

      // Create new tagger and load the modified buffer
      const newTaggedFile = TaggedFile.loadSync(newBuffer)

      // Verify persisted changes
      expect(newTaggedFile.title).toBe('Modified Title')
      expect(newTaggedFile.artist).toBe('Modified Artist')
      expect(newTaggedFile.year).toBe(2024)
      expect(newTaggedFile.genre).toBe('Test Genre')
    })

    it.skipIf(isWasi)('async should handle multiple file formats', async () => {
      const formats = ['mp3.mp3', 'flac.flac', 'ogg.opus', 'wav.wav']

      for (const file of formats) {
        const path = join(base, file)

        const taggedFile = await TaggedFile.load(path)
        expect(taggedFile.tagType).toBeTruthy()
        expect(taggedFile.duration).toBeGreaterThanOrEqual(0)
      }
    })

    it.skipIf(isWasi)('sync should handle multiple file formats', () => {
      const formats = ['mp3.mp3', 'flac.flac', 'ogg.opus', 'wav.wav']

      for (const file of formats) {
        const path = join(base, file)

        const taggedFile = TaggedFile.loadSync(path)
        expect(taggedFile.tagType).toBeTruthy()
        expect(taggedFile.duration).toBeGreaterThanOrEqual(0)
      }
    })
  })
})
