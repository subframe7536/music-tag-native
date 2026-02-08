# music-tag-native

A high-performance music metadata reader/writer for Node.js and browsers. Read and modify audio file tags (ID3, Vorbis, MP4, etc.) across multiple formats with native performance.

Powered by Rust's [`lofty`](https://github.com/Serial-ATA/lofty-rs) crate and [`napi-rs`](https://github.com/napi-rs/napi-rs) for native bindings, with WebAssembly support for browsers.

## Features

- **Read/Write Metadata**: Title, artist, album, year, genre, track numbers, and more
- **Album Art Support**: Read and write embedded pictures with multiple formats
- **Audio Properties**: Bitrate, sample rate, bit depth, channels, duration
- **Audio Quality Classification**: Automatic HQ/SQ/HiRes detection
- **ReplayGain Support**: Read and write ReplayGain tags
- **Cross-Platform**: Native binaries for macOS, Linux, Windows, Android + WASM for browsers
- **Multiple Formats**: MP3, FLAC, M4A, WAV, OGG, and more

## Installation

```sh
npm install music-tag-native
```

```sh
yarn add music-tag-native
```

```sh
pnpm add music-tag-native
```

```sh
bun add music-tag-native
```

## Usage

### Node.js

```ts
import { MusicTagger } from 'music-tag-native'

const tagger = new MusicTagger()

// Load from file path
tagger.loadPath('/path/to/audio/file.mp3')

// Read metadata
console.log(tagger.title)
console.log(tagger.artist)
console.log(tagger.album)

// Modify metadata
tagger.title = 'New Title'
tagger.artist = 'New Artist'
tagger.year = 2024

// Remove a tag (set to null)
tagger.albumArtist = null

// Save changes back to file
tagger.save()

// Or save to a different file path
tagger.save('/path/to/output.mp3')

// Clean up resources
tagger.dispose()
```

### Browser

```ts
import { MusicTagger } from 'music-tag-native'

const tagger = new MusicTagger()

// Load from buffer
const response = await fetch('/path/to/audio/file.mp3')
const arrayBuffer = await response.arrayBuffer()
const buffer = new Uint8Array(arrayBuffer)

tagger.loadBuffer(buffer)

// Read and modify metadata
console.log(tagger.title)
tagger.title = 'New Title'

// Get modified buffer
const modifiedBuffer = tagger.save()

// Display album art
const pictures = tagger.pictures
if (pictures.length > 0) {
  const picture = pictures[0]
  const blob = new Blob([picture.data], { type: picture.mimeType })
  const url = URL.createObjectURL(blob)
  document.querySelector('img').src = url
}

tagger.dispose()
```

## API Reference

### MusicTagger

#### Loading Files

- `loadPath(path: string): void` - Load audio file from path (Node.js only)
- `loadBuffer(buffer: Uint8Array): void` - Load audio file from buffer

#### Saving Changes

- `save(path?: string | null): Uint8Array | undefined` - Save changes. In Node.js, saves to original path by default, or to `path` when provided. In browser/wasm, `path` is not supported.

#### Resource Management

- `dispose(): void` - Clean up resources. Always call when done with the tagger instance

#### Metadata Properties (Read/Write)

All properties can be read and written. Set to `null` to remove a tag.

- `title: string | null`
- `artist: string | null`
- `album: string | null`
- `albumArtist: string | null`
- `genre: string | null`
- `composer: string | null`
- `comment: string | null`
- `year: number | null`
- `rating: number | null`
- `trackNumber: number | null`
- `trackTotal: number | null`
- `discNumber: number | null`
- `discTotal: number | null`

#### Audio Properties (Read-Only)

- `duration: number` - Duration in seconds
- `overallBitrate: number` - Overall bitrate in kbps
- `audioBitrate: number` - Audio bitrate in kbps
- `sampleRate: number` - Sample rate in Hz
- `bitDepth: number` - Bit depth
- `channels: number` - Number of channels
- `audioQuality: 'HQ' | 'SQ' | 'HiRes' | null` - Audio quality classification

#### Album Art

- `pictures: MetaPicture[]` - Array of embedded pictures
- `setPictures(pictures: MetaPicture[]): void` - Replace all pictures

#### ReplayGain

- `replayGainTrackGain: number | null`
- `replayGainTrackPeak: number | null`
- `replayGainAlbumGain: number | null`
- `replayGainAlbumPeak: number | null`

### MetaPicture

Properties for album art and embedded images:

- `pictureType: PictureType` - Type of picture (e.g., 'CoverFront', 'CoverBack')
- `mimeType: string` - MIME type (e.g., 'image/jpeg', 'image/png')
- `description: string | null` - Optional description
- `data: Uint8Array` - Image data

#### PictureType Values

`'Other'`, `'Icon'`, `'OtherIcon'`, `'CoverFront'`, `'CoverBack'`, `'Leaflet'`, `'Media'`, `'LeadArtist'`, `'Artist'`, `'Conductor'`, `'Band'`, `'Composer'`, `'Lyricist'`, `'RecordingLocation'`, `'DuringRecording'`, `'DuringPerformance'`, `'ScreenCapture'`, `'BrightFish'`, `'Illustration'`, `'BandLogo'`, `'PublisherLogo'`, `'Undefined'`

## Platform Support

Native binaries are automatically installed for:

- macOS (x64, ARM64)
- Linux (x64, ARM64 - GNU and musl)
- Windows (x64, ia32, ARM64)
- Android (ARM64)

WebAssembly fallback is available for unsupported platforms and browsers.

## Development

```sh
# Install dependencies
pnpm install

# Build native addon for current platform
pnpm build

# Build WASM target
pnpm build:wasm

# Run tests
pnpm test

# Run playground
pnpm play
```

See [package.json](./package.json) for all available scripts.

## Type Definitions

Full TypeScript definitions are available in [index.d.ts](./index.d.ts).

## License

MIT
