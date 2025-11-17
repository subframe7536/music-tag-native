// @ts-expect-error fxxk
import { MusicTagger } from '../music-tag-native.wasi-browser.js'
import type { MusicTagger as Tagger } from '../index'
declare class MusicTagger extends Tagger {}

import flacSampleUrl from '../samples/flac.flac?url'

function readProperties(tagger: MusicTagger) {
  return {
    quality: tagger.quality,
    bitDepth: tagger.bitDepth,
    bitRate: tagger.bitRate,
    sampleRate: tagger.sampleRate,
    channels: tagger.channels,
    duration: tagger.duration,
    tagType: tagger.tagType,
  }
}

function readTags(tagger: MusicTagger) {
  return {
    title: tagger.title,
    artist: tagger.artist,
    album: tagger.album,
    albumArtist: tagger.albumArtist,
    genre: tagger.genre,
    year: tagger.year,
    trackNumber: tagger.trackNumber,
    trackTotal: tagger.trackTotal,
    discNumber: tagger.discNumber,
    discsTotal: tagger.discsTotal,
    composer: tagger.composer,
    conductor: tagger.conductor,
    lyricist: tagger.lyricist,
    publisher: tagger.publisher,
    comment: tagger.comment,
    lyrics: tagger.lyrics,
    copyright: tagger.copyright,
  }
}

function readReplay(tagger: MusicTagger) {
  return {
    trackReplayGain: tagger.trackReplayGain,
    trackReplayPeak: tagger.trackReplayPeak,
    albumReplayGain: tagger.albumReplayGain,
    albumReplayPeak: tagger.albumReplayPeak,
  }
}

const tagger = new MusicTagger()

const _ = await fetch(flacSampleUrl).then((res) => res.arrayBuffer())
const buffer = new Uint8Array(_)
console.log(buffer)
tagger.loadBuffer(buffer)

console.table(readTags(tagger))
console.table(readProperties(tagger))
console.table(readReplay(tagger))
