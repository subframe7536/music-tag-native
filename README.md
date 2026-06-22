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
import { TaggedFile } from 'music-tag-native'

// Load from file path
const tagged_file = await TaggedFile.loadFromPath('/path/to/audio/file.mp3')
// synchronous:
const tagged_file_sync = TaggedFile.loadFromPathSync('/path/to/audio/file.mp3')

// Read metadata
console.log(tagged_file.title)
console.log(tagged_file.artist)
console.log(tagged_file.album)

// Modify metadata
tagged_file.title = 'New Title'
tagged_file.artist = 'New Artist'
tagged_file.year = 2024

// Remove a tag (set to null)
tagged_file.albumArtist = null

// Save changes back to file
await tagged_file.save()
// synchronous:
tagged_file.saveSync()

// Or save to a different file path
await tagged_file.save('/path/to/output.mp3')
```

### Browser

```ts
import { TaggedFile } from 'music-tag-native'

// Load from buffer
const response = await fetch('/path/to/audio/file.mp3')
const arrayBuffer = await response.arrayBuffer()
const buffer = new Uint8Array(arrayBuffer)

// `loadFromBuffer` is synchronous only
const tagged_file = TaggedFile.loadFromBuffer(buffer)

// Read and modify metadata
console.log(tagged_file.title)
tagged_file.title = 'New Title'

// Get modified buffer, you need to provide the original data, a new copy with updated tags will be returned
const modifiedBuffer = await tagged_file.save(buffer)
// synchronous:
const modifiedBufferSync = tagged_file.saveSync(buffer)

// Display album art
const pictures = tagged_file.pictures
if (pictures && pictures.length > 0) {
  const picture = pictures[0]
  const blob = new Blob([picture.data], { type: picture.mimeType })
  const url = URL.createObjectURL(blob)
  document.querySelector('img').src = url
}
```

## API Reference

### TaggedFile

#### Loading Files

- `TaggedFile.loadFromPath(path: string): Promise<TaggedFile>` - Load audio file from path (Node.js only)
- `TaggedFile.loadFromPathSync(path: string): TaggedFile` - Load audio file from path (Node.js only)
- `TaggedFile.loadFromBuffer(buffer: Uint8Array): TaggedFile` - Load audio file from buffer

#### Saving Changes

- `save(bufferOrPath?: Uint8Array | string | null): Promise<Uint8Array | void>` - Save changes asynchronously. Files loaded from a path are saved to the original path by default, or to `bufferOrPath` when a path is provided. Files loaded from a buffer require the original buffer and return an updated copy.
- `saveSync(bufferOrPath?: Uint8Array | string | null): Uint8Array | undefined` - Synchronous version of `save`.
- `path(): string | null` - Return the source path for path-loaded files, or `null` for buffer-loaded files.

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
- `discsTotal: number | null`
- `conductor: string | null`
- `lyricist: string | null`
- `publisher: string | null`
- `lyrics: string | null`
- `copyright: string | null`
- `trackReplayGain: number | null`
- `trackReplayPeak: number | null`
- `albumReplayGain: number | null`
- `albumReplayPeak: number | null`
- `pictures: MetaPicture[] | null`

#### Audio Properties (Read-Only)

- `quality: 'HQ' | 'SQ' | 'HiRes'` - Audio quality classification
- `bitDepth: number | null` - Bit depth
- `bitRate: number | null` - Audio bitrate in kbps
- `sampleRate: number | null` - Sample rate in Hz
- `channels: number | null` - Number of channels
- `duration: number` - Duration in milliseconds
- `tagType: 'AIFF' | 'APE' | 'ID3V1' | 'ID3V2' | 'ILST' | 'RIFF' | 'VORBIS' | null` - Metadata tag type

#### Album Art

- `pictures: MetaPicture[] | null` - Embedded pictures. Set to `null` to remove all pictures.

#### ReplayGain

- `trackReplayGain: number | null`
- `trackReplayPeak: number | null`
- `albumReplayGain: number | null`
- `albumReplayPeak: number | null`

### MetaPicture

Properties for album art and embedded images:

- `coverType: PictureType` - Type of picture
- `mimeType?: string` - MIME type (e.g., 'image/jpeg', 'image/png')
- `description?: string` - Optional description
- `data: Uint8Array` - Image data

#### PictureType Values

`'Cover Art (Other)'`, `'Cover Art (Png Icon)'`, `'Cover Art (Icon)'`, `'Cover Art (Front)'`, `'Cover Art (Back)'`, `'Cover Art (Leaflet)'`, `'Cover Art (Media)'`, `'Cover Art (Lead Artist)'`, `'Cover Art (Artist)'`, `'Cover Art (Conductor)'`, `'Cover Art (Band)'`, `'Cover Art (Composer)'`, `'Cover Art (Lyricist)'`, `'Cover Art (Recording Location)'`, `'Cover Art (During Recording)'`, `'Cover Art (During Performance)'`, `'Cover Art (Video Capture)'`, `'Cover Art (Fish)'`, `'Cover Art (Illustration)'`, `'Cover Art (Band Logotype)'`, `'Cover Art (Publisher Logotype)'`, `'Unknown'`

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
