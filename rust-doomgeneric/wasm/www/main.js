// The page: takes the WAD, shows the frames the worker sends, plays its sound and forwards the
// keyboard.
import * as storage from './storage.js';

const stage = document.getElementById('stage');
const canvas = document.getElementById('screen');
const context = canvas.getContext('2d');
const overlay = document.getElementById('overlay');
const message = document.getElementById('message');
const hint = document.getElementById('hint');
const chooser = document.getElementById('chooser');
const fileInput = document.getElementById('file');

// The WAD to play, `{ name, data }` (data an ArrayBuffer), or null while there is none. It is kept
// in the browser (see storage.js), so that it only has to be dropped once.
let wad = null;
let worker = null;
let audio = null;
let nextSoundTime = 0;
let pendingFrame = null;
let running = false;
const held = new Set();

const DROP_HINT =
  'The shareware doom1.wad works, and so does the WAD of a game you own. It stays in your browser.';

function say(text, subtext, canChoose) {
  message.textContent = text;
  hint.textContent = subtext;
  chooser.hidden = !canChoose;
  overlay.hidden = false;
}

// Waiting for the player: to drop a WAD, or to start the game.
function idle(text) {
  if (wad) say(text ?? 'Click to play', `${wad.name} · drop another .wad here to replace it`, true);
  else say(text ?? 'Drop a DOOM .wad here', DROP_HINT, true);
}

function start() {
  if (worker || !wad) return;
  say('Loading…', '', false);
  // The browser only lets sound start after a click or a key press (see `resumeAudio`).
  audio = new AudioContext();
  worker = new Worker('worker.js', { type: 'module' });
  worker.onmessage = ({ data }) => {
    switch (data.type) {
      case 'frame':
        showFrame(data);
        break;
      case 'audio':
        playSound(data);
        break;
      case 'ready':
        running = true;
        overlay.hidden = true;
        break;
      case 'quit':
        stop('You quit the game.');
        break;
      case 'error':
        if (data.badWad) forgetWad();
        stop(`The game stopped: ${data.message}`);
        break;
    }
  };
  // The worker copies what it needs; it is sent a copy so that the WAD stays here.
  const copy = wad.data.slice(0);
  worker.postMessage({ type: 'start', wad: copy }, [copy]);
  if (document.hidden) pause();
}

// Ends the game that is running, if any.
function teardown() {
  running = false;
  held.clear();
  worker?.terminate();
  worker = null;
  pendingFrame = null;
  audio?.close();
  audio = null;
  nextSoundTime = 0;
}

function stop(text) {
  teardown();
  idle(text);
}

function forgetWad() {
  wad = null;
  storage.remove('wad', 'last');
}

// A dropped or chosen file. Only the first bytes are looked at here; the game says whether it can
// run the WAD when it starts.
async function takeFile(file) {
  const data = await file.arrayBuffer();
  const magic = new TextDecoder().decode(new Uint8Array(data, 0, Math.min(4, data.byteLength)));
  if (magic !== 'IWAD' && magic !== 'PWAD') {
    if (worker) stop();
    idle(`${file.name} is not a WAD file.`);
    return;
  }
  wad = { name: file.name, data };
  storage.put('wad', 'last', wad);
  // The game starts right away, replacing one that is running with another WAD.
  teardown();
  start();
}

// ---- pausing --------------------------------------------------------------------------------

// A page that is not shown is not played: the game stops, with its clock, and so does the sound.
function pause() {
  if (!worker) return;
  for (const code of [...held]) sendKey(false, code);
  worker.postMessage({ type: 'pause' });
  audio.suspend();
}

function resume() {
  if (!worker) return;
  worker.postMessage({ type: 'resume' });
  audio.resume();
}

document.addEventListener('visibilitychange', () => (document.hidden ? pause() : resume()));

// ---- picture and sound ----------------------------------------------------------------------

// Only the newest frame is drawn, once per screen refresh.
function showFrame({ pixels, width, height }) {
  const first = pendingFrame === null;
  pendingFrame = new ImageData(new Uint8ClampedArray(pixels), width, height);
  if (first) requestAnimationFrame(drawFrame);
}

function drawFrame() {
  if (pendingFrame) context.putImageData(pendingFrame, 0, 0);
  pendingFrame = null;
}

// The worker sends the sound in short pieces as the game mixes it. Each one is scheduled right
// behind the previous one, with a little lead so that the next piece arrives in time.
const SOUND_LEAD = 0.06;
const SOUND_MAX_AHEAD = 0.3;
function playSound({ samples, rate }) {
  // Nothing is played while the browser holds the sound back (or the page is hidden): what the
  // game mixed in the meantime would come out late.
  if (audio?.state !== 'running') return;
  const frames = samples.length / 2;
  const now = audio.currentTime;
  if (nextSoundTime < now) nextSoundTime = now + SOUND_LEAD;
  // The game's clock runs a little differently from the sound card's; do not let the delay grow.
  if (nextSoundTime > now + SOUND_MAX_AHEAD) return;
  const buffer = audio.createBuffer(2, frames, rate);
  const left = buffer.getChannelData(0);
  const right = buffer.getChannelData(1);
  for (let i = 0; i < frames; i++) {
    left[i] = samples[2 * i] / 32768;
    right[i] = samples[2 * i + 1] / 32768;
  }
  const source = audio.createBufferSource();
  source.buffer = buffer;
  source.connect(audio.destination);
  source.start(nextSoundTime);
  nextSoundTime += frames / rate;
}

// ---- keyboard -------------------------------------------------------------------------------

// A game that starts without a click (a dropped WAD is not one, and neither is choosing a file
// after a long wait) gets its sound held back by the browser until the player presses a key or
// clicks. Then it is let through.
function resumeAudio() {
  if (audio?.state === 'suspended' && !document.hidden) audio.resume();
}
window.addEventListener('keydown', resumeAudio, { capture: true });
window.addEventListener('pointerdown', resumeAudio, { capture: true });

function sendKey(pressed, code) {
  if (pressed) held.add(code);
  else held.delete(code);
  worker.postMessage({ type: 'key', pressed, code });
}

// Keys the browser should keep for itself: reload, full screen, developer tools, and shortcuts
// with Ctrl (which is also "fire") and a letter.
function leaveToBrowser(event) {
  return (
    event.metaKey ||
    ['F5', 'F11', 'F12'].includes(event.code) ||
    (event.ctrlKey && event.code.startsWith('Key'))
  );
}

window.addEventListener('keydown', (event) => {
  if (!worker) {
    if (wad && (event.code === 'Enter' || event.code === 'Space')) {
      event.preventDefault();
      start();
    }
    return;
  }
  if (!running) return;
  if (!leaveToBrowser(event)) event.preventDefault();
  if (!event.repeat) sendKey(true, event.code);
});

window.addEventListener('keyup', (event) => {
  if (!running) return;
  if (!leaveToBrowser(event)) event.preventDefault();
  sendKey(false, event.code);
});

// A key released while another window has the focus would stay pressed.
window.addEventListener('blur', () => {
  if (!running) return;
  for (const code of [...held]) sendKey(false, code);
});

// ---- dropping a WAD -------------------------------------------------------------------------

const hasFiles = (event) => event.dataTransfer?.types.includes('Files');

// A file dropped beside the game window must not make the browser open it instead.
window.addEventListener('dragover', (event) => hasFiles(event) && event.preventDefault());
window.addEventListener('drop', (event) => hasFiles(event) && event.preventDefault());

// dragenter and dragleave come for every element the pointer crosses, so count them.
let dragDepth = 0;
stage.addEventListener('dragenter', (event) => {
  if (!hasFiles(event)) return;
  dragDepth++;
  stage.classList.add('dragging');
});
stage.addEventListener('dragleave', () => {
  dragDepth = Math.max(0, dragDepth - 1);
  if (dragDepth === 0) stage.classList.remove('dragging');
});
stage.addEventListener('drop', (event) => {
  dragDepth = 0;
  stage.classList.remove('dragging');
  const file = event.dataTransfer?.files[0];
  if (file) takeFile(file);
});

fileInput.addEventListener('change', () => {
  const file = fileInput.files[0];
  fileInput.value = '';
  if (file) takeFile(file);
});

// Clicking anywhere on the overlay starts the game, except on the file chooser.
overlay.addEventListener('click', (event) => {
  if (!event.target.closest('#chooser')) start();
});

canvas.addEventListener('dblclick', () => {
  if (document.fullscreenElement) document.exitFullscreen();
  else canvas.requestFullscreen();
});

storage.get('wad', 'last').then((last) => {
  // Not while a WAD was dropped in the meantime.
  if (last && !wad) wad = last;
  if (!worker) idle();
});
