# `music-tag-native`

Music tag reader / writter in Node.js / Browser, powered by [`napi-rs`](https://github.com/napi-rs/napi-rs) and [`lofty`](https://github.com/Serial-ATA/lofty-rs)

## Install

```sh
npm i music-tag-native
```
```sh
yarn add music-tag-native
```
```sh
pnpm add music-tag-native
```
```sh
bun i music-tag-native
```

## Usage

Browser

```ts
import { MusicTagger } from 'music-tag-native'
import url from '../samples/flac.flac?url'

console.time('total')

const tagger = new MusicTagger()

const _ = await fetch(url).then(res => res.arrayBuffer())
const buffer = new Uint8Array(_)
tagger.loadBuffer(buffer)
// update the title
tagger.title = 'test'
// remove the album artist
tagger.albumArtist = null

tagger.save()
console.log(tagger.title)

getPictureBase64(file.pictures[0])
  .then(d => document.querySelector('img')!.src = d)

console.timeEnd('total')
```

Node.js

```ts
import { MusicTagger } from 'music-tag-native'

const tagger = new MusicTagger()
tagger.loadPath("/path/to/the/file")
tagger.title = 'test'

tagger.save()
console.log(tagger.title)
```

### Type Definition

See [index.d.ts](./index.d.ts)

## License

MIT
