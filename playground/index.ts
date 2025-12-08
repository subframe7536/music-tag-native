// @ts-expect-error fxxk
import { MusicTagger } from '../music-tag-native.wasi-browser.js'
// oxlint-disable-next-line no-unused-vars
import type { MusicTagger as Tagger } from '../index'
declare class MusicTagger extends Tagger {}

import flacSampleUrl from '../samples/flac.flac?url'
import mp3SampleUrl from '../samples/mp3.mp3?url'
import oggSampleUrl from '../samples/ogg.opus?url'
import wavSampleUrl from '../samples/wav.wav?url'

const samples = [
  { name: 'FLAC', format: 'flac', url: flacSampleUrl },
  { name: 'MP3', format: 'mp3', url: mp3SampleUrl },
  { name: 'OGG Opus', format: 'opus', url: oggSampleUrl },
  { name: 'WAV', format: 'wav', url: wavSampleUrl },
]

interface AppState {
  tagger: MusicTagger | null
  currentBuffer: () => Uint8Array<ArrayBuffer> | null
  currentSample: string | null
  editMode: boolean
  hasChanges: boolean
  originalTags: Record<string, any>
}

const state: AppState = {
  tagger: null,
  currentBuffer() {
    return this.tagger?.buffer as any
  },
  currentSample: null,
  editMode: false,
  hasChanges: false,
  originalTags: {},
}

const elements = {
  sampleList: document.querySelector('[data-sample-list]')!,
  status: document.querySelector('[data-status]')!,
  properties: document.querySelector('[data-properties]')!,
  tags: document.querySelector('[data-tags]')!,
  replay: document.querySelector('[data-replay]')!,
  pictures: document.querySelector('[data-pictures]')!,
  editModeBtn: document.querySelector('[data-edit-mode]')!,
  saveBtn: document.querySelector('[data-save-changes]')!,
  resetBtn: document.querySelector('[data-reset-changes]')!,
  downloadSection: document.querySelector('[data-download-section]')!,
  downloadBtn: document.querySelector('[data-download]')!,
}

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
    trackReplayGain: tagger.trackReplayGain,
    trackReplayPeak: tagger.trackReplayPeak,
    albumReplayGain: tagger.albumReplayGain,
    albumReplayPeak: tagger.albumReplayPeak,
  }
}

function renderTable(tbody: Element, data: Record<string, any>, editable = false) {
  if (Object.keys(data).length === 0) {
    tbody.innerHTML = '<tr class="table-empty"><td colspan="2">No data available</td></tr>'
    return
  }

  tbody.innerHTML = Object.entries(data)
    .map(([key, value]) => {
      const displayValue = value ?? '<em>null</em>'
      const valueCell = editable && state.editMode
        ? `<input class="editable-input" data-tag="${key}" value="${value ?? ''}" placeholder="null" />`
        : `<code>${displayValue}</code>`
      
      return `<tr ${editable ? `data-tag-row="${key}"` : ''}>
        <td>${key}</td>
        <td>${valueCell}</td>
      </tr>`
    })
    .join('')

  if (editable && state.editMode) {
    tbody.querySelectorAll('.editable-input').forEach((input) => {
      input.addEventListener('input', handleTagEdit)
    })
  }
}

function handleTagEdit(e: Event) {
  const input = e.target as HTMLInputElement
  const tagName = input.dataset.tag!
  const originalValue = state.originalTags[tagName]
  const newValue = input.value.trim() || null
  
  const row = input.closest('tr')!
  if (String(originalValue) !== String(newValue)) {
    row.classList.add('tag-modified')
    state.hasChanges = true
  } else {
    row.classList.remove('tag-modified')
  }
  
  updateButtons()
}

function updateButtons() {
  const saveBtn = elements.saveBtn as HTMLButtonElement
  const resetBtn = elements.resetBtn as HTMLButtonElement
  
  saveBtn.disabled = !state.hasChanges
  resetBtn.disabled = !state.hasChanges
}

function renderPictures(tagger: MusicTagger) {
  const pictures = tagger.pictures
  
  if (!pictures || pictures.length === 0) {
    elements.pictures.innerHTML = '<p class="empty-state">No embedded pictures</p>'
    return
  }

  elements.pictures.innerHTML = pictures
    .map((pic) => {
      const blob = new Blob([pic.data as any], { type: pic.mimeType })
      const url = URL.createObjectURL(blob)
      return `
        <figure class="picture-card">
          <img src="${url}" alt="${pic.description || 'Album art'}" />
          <figcaption>
            <strong>${pic.coverType}</strong><br />
            ${pic.mimeType}<br />
            ${pic.description || '<em>No description</em>'}
          </figcaption>
        </figure>
      `
    })
    .join('')
}

async function loadSample(sample: typeof samples[0]) {
  try {
    elements.status.textContent = `Loading ${sample.name}...`
    elements.status.classList.remove('error')

    const response = await fetch(sample.url)
    const arrayBuffer = await response.arrayBuffer()
    const buffer = new Uint8Array(arrayBuffer)

    if (state.tagger) {
      state.tagger.dispose()
    }

    const tagger = new MusicTagger()
    tagger.loadBuffer(buffer)

    state.tagger = tagger
    state.currentSample = sample.format
    state.originalTags = readTags(tagger)
    state.hasChanges = false
    state.editMode = false

    elements.status.textContent = `✓ Loaded ${sample.name} successfully`
    
    renderTable(elements.properties, readProperties(tagger))
    renderTable(elements.tags, readTags(tagger), true)
    renderPictures(tagger)

    elements.editModeBtn.classList.remove('active')
    ;(elements.editModeBtn as HTMLButtonElement).disabled = false
    ;(elements.downloadSection as HTMLButtonElement).style.display = 'none'
    updateButtons()

  } catch (error) {
    elements.status.textContent = `✗ Error: ${error}`
    elements.status.classList.add('error')
    console.error(error)
  }
}

function toggleEditMode() {
  state.editMode = !state.editMode
  elements.editModeBtn.classList.toggle('active', state.editMode)
  
  if (state.tagger) {
    renderTable(elements.tags, readTags(state.tagger), true)
  }
}

function saveChanges() {
  if (!state.tagger || !state.hasChanges) return

  const inputs = elements.tags.querySelectorAll('.editable-input') as NodeListOf<HTMLInputElement>
  
  inputs.forEach((input) => {
    const tagName = input.dataset.tag!
    const value = input.value.trim() || null
    
    // @ts-expect-error dynamic property access
    state.tagger[tagName] = value === null ? null : (
      tagName === 'year' || tagName.includes('Number') || tagName.includes('Total')
        ? Number(value) || null
        : value
    )
  })

  state.tagger.save()
  
  if (state.tagger.buffer) {
    state.originalTags = readTags(state.tagger)
    state.hasChanges = false
    
    elements.status.textContent = '✓ Changes saved to buffer!'
    ;(elements.downloadSection as HTMLElement).style.display = 'block'
    
    renderTable(elements.tags, readTags(state.tagger), true)
    updateButtons()
    
    console.log('✓ Tags updated successfully')
    console.table(readTags(state.tagger))
  }
}

function resetChanges() {
  if (!state.tagger) return
  
  state.hasChanges = false
  renderTable(elements.tags, state.originalTags, true)
  updateButtons()
  
  elements.status.textContent = 'Changes reset'
}

function downloadModifiedFile() {
  if (!state.currentSample) {
    return
  }

  const buf = state.currentBuffer()
  if (!buf) {
    return
  }
  const blob = new Blob([buf], { type: 'audio/*' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `modified-${Date.now()}.${state.currentSample}`
  a.click()
  URL.revokeObjectURL(url)
  
  elements.status.textContent = '✓ File downloaded!'
}

// Render sample buttons
elements.sampleList.innerHTML = samples
  .map(
    (sample) => `
    <button class="sample-button" data-sample="${sample.format}">
      <strong>${sample.name}</strong>
      <span>${sample.format.toUpperCase()}</span>
    </button>
  `
  )
  .join('')

// Event listeners
elements.sampleList.addEventListener('click', (e) => {
  const button = (e.target as Element).closest('.sample-button') as HTMLButtonElement
  if (!button) return

  const format = button.dataset.sample!
  const sample = samples.find((s) => s.format === format)!
  
  document.querySelectorAll('.sample-button').forEach((btn) => btn.classList.remove('is-active'))
  button.classList.add('is-active')
  
  loadSample(sample)
})

elements.editModeBtn.addEventListener('click', toggleEditMode)
elements.saveBtn.addEventListener('click', saveChanges)
elements.resetBtn.addEventListener('click', resetChanges)
elements.downloadBtn.addEventListener('click', downloadModifiedFile)

// Load first sample by default
loadSample(samples[0])
