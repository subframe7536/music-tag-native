import { subfFmt } from '@subf/config/oxfmt'

export default subfFmt({
  ignorePatterns: [
    'index.js',
    'index.d.ts',
    'wasi-worker.mjs',
    'wasi-worker-browser.mjs',
    'music-tag-native.wasi-browser.js',
    'music-tag-native.wasi.cjs',
  ],
})
