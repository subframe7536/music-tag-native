import { rmSync } from 'fs'
import { MusicTagger } from '.'

const tagger = new MusicTagger()

tagger.loadPath('./samples/mp3.mp3')
console.log(tagger.title)
tagger.rating = 1
const targetPath = './test.mp3'
tagger.save(targetPath)
tagger.dispose()
tagger.loadPath(targetPath)
console.log(tagger.rating)
rmSync(targetPath)