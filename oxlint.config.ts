import { subfLint } from '@subf/config/oxlint'

export default subfLint({
  ignorePatterns: [
    'index.js',
    'index.d.ts',
    'wasi-worker.mjs',
    'wasi-worker-browser.mjs',
    'music-tag-native.wasi-browser.js',
    'music-tag-native.wasi.cjs',
  ],
})
