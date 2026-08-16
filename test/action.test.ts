import {
  copyFileSync,
  linkSync,
  lstatSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
} from 'node:fs'
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

  describe('save', { concurrent: false }, () => {
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

  describe('saveSync', { concurrent: false }, () => {
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

    it('should atomically save when the source path is passed explicitly', async () => {
      const path = join(tmpdir(), `music-tag-native-save-explicit-source-${Date.now()}.mp3`)
      copyFileSync(join(base, 'mp3.mp3'), path)

      try {
        const musicFile = await MusicFile.load(path)
        musicFile.title = 'Explicit source path title'
        await musicFile.save(path)

        expect((await MusicFile.load(path)).title).toBe('Explicit source path title')
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

    it.skipIf(process.platform === 'win32')(
      'preserves symlinks while replacing their target',
      async () => {
        const directory = mkdtempSync(join(tmpdir(), 'music-tag-native-symlink-'))
        const realPath = join(directory, 'real.mp3')
        const linkPath = join(directory, 'alias.mp3')
        copyFileSync(join(base, 'mp3.mp3'), realPath)
        symlinkSync(realPath, linkPath)

        try {
          const musicFile = await MusicFile.load(linkPath)
          musicFile.title = 'Symlink title'
          await musicFile.save(linkPath)

          expect(lstatSync(linkPath).isSymbolicLink()).toBe(true)
          expect((await MusicFile.load(realPath)).title).toBe('Symlink title')
        } finally {
          rmSync(directory, { force: true, recursive: true })
        }
      },
    )

    it('replaces a hard-link directory entry independently', async () => {
      const directory = mkdtempSync(join(tmpdir(), 'music-tag-native-hardlink-'))
      const realPath = join(directory, 'real.mp3')
      const hardLinkPath = join(directory, 'alias.mp3')
      copyFileSync(join(base, 'mp3.mp3'), realPath)
      linkSync(realPath, hardLinkPath)

      try {
        const musicFile = await MusicFile.load(realPath)
        musicFile.title = 'Hard-link title'
        await musicFile.save(hardLinkPath)

        expect((await MusicFile.load(realPath)).title).not.toBe('Hard-link title')
        expect((await MusicFile.load(hardLinkPath)).title).toBe('Hard-link title')
      } finally {
        rmSync(directory, { force: true, recursive: true })
      }
    })

    it('keeps the source unchanged when the destination parent is missing', async () => {
      const directory = mkdtempSync(join(tmpdir(), 'music-tag-native-save-failure-'))
      const sourcePath = join(directory, 'source.mp3')
      const targetPath = join(directory, 'missing', 'target.mp3')
      copyFileSync(join(base, 'mp3.mp3'), sourcePath)
      const sourceBefore = readFileSync(sourcePath)

      try {
        const musicFile = await MusicFile.load(sourcePath)
        musicFile.title = 'Should not be written'

        await expect(musicFile.save(targetPath)).rejects.toThrow()
        expect(readFileSync(sourcePath)).toEqual(sourceBefore)
      } finally {
        rmSync(directory, { force: true, recursive: true })
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

    it('should atomically save when the source path is passed explicitly', () => {
      const path = join(tmpdir(), `music-tag-native-save-explicit-source-sync-${Date.now()}.mp3`)
      copyFileSync(join(base, 'mp3.mp3'), path)

      try {
        const musicFile = MusicFile.loadSync(path)
        musicFile.title = 'Explicit source path title'
        musicFile.saveSync(path)

        expect(MusicFile.loadSync(path).title).toBe('Explicit source path title')
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

    it('keeps the source unchanged when the destination parent is missing', () => {
      const directory = mkdtempSync(join(tmpdir(), 'music-tag-native-save-failure-sync-'))
      const sourcePath = join(directory, 'source.mp3')
      const targetPath = join(directory, 'missing', 'target.mp3')
      copyFileSync(join(base, 'mp3.mp3'), sourcePath)
      const sourceBefore = readFileSync(sourcePath)

      try {
        const musicFile = MusicFile.loadSync(sourcePath)
        musicFile.title = 'Should not be written'

        expect(() => musicFile.saveSync(targetPath)).toThrow()
        expect(readFileSync(sourcePath)).toEqual(sourceBefore)
      } finally {
        rmSync(directory, { force: true, recursive: true })
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
