import { copyFileSync, readFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { describe, it, expect } from 'vitest'

import { MusicFile } from '../index'

import { base } from './const'

const isWasi = process.env.NAPI_RS_FORCE_WASI === '1'

describe('MusicFile', () => {
  describe.skipIf(isWasi)('load', () => {
    it('should load an MP3 file', async () => {
      const path = join(base, 'mp3.mp3')
      const musicFile = await MusicFile.load(path)

      expect(musicFile.tagType).toBeTruthy()
    })

    it('should load a FLAC file', async () => {
      const path = join(base, 'flac.flac')
      const musicFile = await MusicFile.load(path)

      expect(musicFile.tagType).toBeTruthy()
    })

    it('should load an OGG/Opus file', async () => {
      const path = join(base, 'ogg.opus')
      const musicFile = await MusicFile.load(path)

      expect(musicFile.tagType).toBeTruthy()
    })

    it('should load a WAV file', async () => {
      const path = join(base, 'wav.wav')
      const musicFile = await MusicFile.load(path)

      expect(musicFile.tagType).toBeTruthy()
    })

    it('should throw error for non-existent file', async () => {
      await expect(async () => {
        await MusicFile.load('non-existent-file.mp3')
      }).rejects.toThrow()
    })

    it('should throw error for invalid audio file', async () => {
      await expect(async () => {
        await MusicFile.load(__filename) // Try to load the test file itself
      }).rejects.toThrow()
    })
  })

  describe.skipIf(isWasi)('loadSync', () => {
    it('should load an MP3 file', () => {
      const path = join(base, 'mp3.mp3')
      const musicFile = MusicFile.loadSync(path)

      expect(musicFile.tagType).toBeTruthy()
    })

    it('should load a FLAC file', () => {
      const path = join(base, 'flac.flac')
      const musicFile = MusicFile.loadSync(path)

      expect(musicFile.tagType).toBeTruthy()
    })

    it('should load an OGG/Opus file', () => {
      const path = join(base, 'ogg.opus')
      const musicFile = MusicFile.loadSync(path)

      expect(musicFile.tagType).toBeTruthy()
    })

    it('should load a WAV file', () => {
      const path = join(base, 'wav.wav')
      const musicFile = MusicFile.loadSync(path)

      expect(musicFile.tagType).toBeTruthy()
    })

    it('should throw error for non-existent file', () => {
      expect(() => {
        MusicFile.loadSync('non-existent-file.mp3')
      }).toThrow()
    })

    it('should throw error for invalid audio file', () => {
      expect(() => {
        MusicFile.loadSync(__filename) // Try to load the test file itself
      }).toThrow()
    })
  })

  describe('loadSync', () => {
    it('should load from buffer', () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      const musicFile = MusicFile.loadSync(uint8Array)

      expect(musicFile.tagType).toBeTruthy()
    })

    it('should throw error for invalid buffer', () => {
      const invalidBuffer = new Uint8Array([1, 2, 3, 4, 5])

      expect(() => {
        MusicFile.loadSync(invalidBuffer)
      }).toThrow()
    })
  })

  describe.sequential('save', () => {
    it('should save buffer after loading from buffer', async () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      const musicFile = MusicFile.loadSync(uint8Array)
      musicFile.title = 'New Title'
      const newBuffer = (await musicFile.save(uint8Array)) as Uint8Array

      expect(newBuffer).toBeInstanceOf(Uint8Array)
      expect(newBuffer.length).toBeGreaterThan(0)
    })
  })

  describe.sequential('saveSync', () => {
    it('should save buffer after loading from buffer', () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      const musicFile = MusicFile.loadSync(uint8Array)
      musicFile.title = 'New Title'
      const newBuffer = musicFile.saveSync(uint8Array) as Uint8Array

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
        const musicFile = await MusicFile.load(path)
        musicFile.title = 'Saved Title'

        await musicFile.save()

        const verifyMusicFile = await MusicFile.load(path)
        expect(verifyMusicFile.title).toBe('Saved Title')
      } finally {
        rmSync(path, { force: true })
      }
    })

    it('should save to custom path', async () => {
      const path = join(base, 'mp3.mp3')
      const targetPath = join(tmpdir(), `music-tag-native-save-${Date.now()}.mp3`)
      const musicFile = await MusicFile.load(path)
      musicFile.title = 'Saved Custom Path Title'

      await musicFile.save(targetPath)

      try {
        const newMusicFile = await MusicFile.load(targetPath)
        expect(newMusicFile.title).toBe('Saved Custom Path Title')
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
        const musicFile = MusicFile.loadSync(path)
        musicFile.title = 'Saved Title'

        musicFile.saveSync()

        const verifyMusicFile = MusicFile.loadSync(path)
        expect(verifyMusicFile.title).toBe('Saved Title')
      } finally {
        rmSync(path, { force: true })
      }
    })

    it('should save to custom path', () => {
      const path = join(base, 'mp3.mp3')
      const targetPath = join(tmpdir(), `music-tag-native-save-${Date.now()}.mp3`)
      const musicFile = MusicFile.loadSync(path)
      musicFile.title = 'Saved Custom Path Title'

      musicFile.saveSync(targetPath)

      try {
        const newMusicFile = MusicFile.loadSync(targetPath)
        expect(newMusicFile.title).toBe('Saved Custom Path Title')
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

      const musicFile = MusicFile.loadSync(uint8Array)

      // Modify metadata
      musicFile.title = 'Modified Title'
      musicFile.artist = 'Modified Artist'
      musicFile.year = 2024
      musicFile.genre = 'Test Genre'

      // Verify changes
      expect(musicFile.title).toBe('Modified Title')
      expect(musicFile.artist).toBe('Modified Artist')
      expect(musicFile.year).toBe(2024)
      expect(musicFile.genre).toBe('Test Genre')

      // Save and reload
      const newBuffer = (await musicFile.save(uint8Array)) as Uint8Array

      // Create new tagger and load the modified buffer
      const newMusicFile = MusicFile.loadSync(newBuffer)

      // Verify persisted changes
      expect(newMusicFile.title).toBe('Modified Title')
      expect(newMusicFile.artist).toBe('Modified Artist')
      expect(newMusicFile.year).toBe(2024)
      expect(newMusicFile.genre).toBe('Test Genre')
    })

    it('sync should load, modify, and save metadata', () => {
      const path = join(base, 'mp3.mp3')
      const buffer = readFileSync(path)
      const uint8Array = new Uint8Array(buffer)

      const musicFile = MusicFile.loadSync(uint8Array)

      // Modify metadata
      musicFile.title = 'Modified Title'
      musicFile.artist = 'Modified Artist'
      musicFile.year = 2024
      musicFile.genre = 'Test Genre'

      // Verify changes
      expect(musicFile.title).toBe('Modified Title')
      expect(musicFile.artist).toBe('Modified Artist')
      expect(musicFile.year).toBe(2024)
      expect(musicFile.genre).toBe('Test Genre')

      // Save and reload
      const newBuffer = musicFile.saveSync(uint8Array) as Uint8Array

      // Create new tagger and load the modified buffer
      const newMusicFile = MusicFile.loadSync(newBuffer)

      // Verify persisted changes
      expect(newMusicFile.title).toBe('Modified Title')
      expect(newMusicFile.artist).toBe('Modified Artist')
      expect(newMusicFile.year).toBe(2024)
      expect(newMusicFile.genre).toBe('Test Genre')
    })

    it.skipIf(isWasi)('async should handle multiple file formats', async () => {
      const formats = ['mp3.mp3', 'flac.flac', 'ogg.opus', 'wav.wav']

      for (const file of formats) {
        const path = join(base, file)

        const musicFile = await MusicFile.load(path)
        expect(musicFile.tagType).toBeTruthy()
        expect(musicFile.duration).toBeGreaterThanOrEqual(0)
      }
    })

    it.skipIf(isWasi)('sync should handle multiple file formats', () => {
      const formats = ['mp3.mp3', 'flac.flac', 'ogg.opus', 'wav.wav']

      for (const file of formats) {
        const path = join(base, file)

        const musicFile = MusicFile.loadSync(path)
        expect(musicFile.tagType).toBeTruthy()
        expect(musicFile.duration).toBeGreaterThanOrEqual(0)
      }
    })
  })
})
