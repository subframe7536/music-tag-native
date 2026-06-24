import type { MetaPicture as MetaPictureInstance, MusicFile as MusicFileInstance } from './index'

export type { MetaPicture as MetaPictureInstance, MusicFile as MusicFileInstance } from './index'

export declare const MetaPicture: typeof MetaPictureInstance

export declare const MusicFile: Omit<typeof MusicFileInstance, 'load' | 'loadSync'> & {
  load: (buffer: Uint8Array) => Promise<MusicFileInstance>
  loadSync: (buffer: Uint8Array) => MusicFileInstance
}

declare const binding: {
  MetaPicture: typeof MetaPicture
  MusicFile: typeof MusicFile
}

export default binding
