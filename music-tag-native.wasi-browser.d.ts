import type { MetaPicture as MetaPictureInstance, TaggedFile as TaggedFileInstance } from './index'

export type { MetaPicture as MetaPictureInstance, TaggedFile as TaggedFileInstance } from './index'

export declare const MetaPicture: typeof MetaPictureInstance

export declare const TaggedFile: Omit<typeof TaggedFileInstance, 'load' | 'loadSync'> & {
  load: (buffer: Uint8Array) => Promise<TaggedFileInstance>
  loadSync: (buffer: Uint8Array) => TaggedFileInstance
}

declare const binding: {
  MetaPicture: typeof MetaPicture
  TaggedFile: typeof TaggedFile
}

export default binding
