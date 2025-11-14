import { MusicTagger } from '.'

const tagger = new MusicTagger()

tagger.loadPath('./samples/mp3.mp3')
tagger.save()
