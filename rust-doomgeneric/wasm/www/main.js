// The page: shows the frames the worker sends, plays its sound and forwards the keyboard.
const canvas = document.getElementById('screen');
const context = canvas.getContext('2d');
const overlay = document.getElementById('overlay');
const message = document.getElementById('message');

let worker = null;
let audio = null;
let nextSoundTime = 0;
let pendingFrame = null;
let running = false;
const held = new Set();

function start() {
  if (worker) return;
  message.textContent = 'Loading…';
  // Sound can only start after a click or a key press, which is why the game waits for one.
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
        stop('You quit the game. Reload the page to play again.');
        break;
      case 'error':
        stop(`The game stopped: ${data.message}`);
        break;
    }
  };
  worker.postMessage({ type: 'start' });
}

function stop(text) {
  running = false;
  held.clear();
  message.textContent = text;
  overlay.hidden = false;
  audio?.close();
}

// Only the newest frame is drawn, once per screen refresh.
function showFrame({ pixels, width, height }) {
  const first = pendingFrame === null;
  pendingFrame = new ImageData(new Uint8ClampedArray(pixels), width, height);
  if (first) requestAnimationFrame(drawFrame);
}

function drawFrame() {
  context.putImageData(pendingFrame, 0, 0);
  pendingFrame = null;
}

// The worker sends the sound in short pieces as the game mixes it. Each one is scheduled right
// behind the previous one, with a little lead so that the next piece arrives in time.
const SOUND_LEAD = 0.06;
const SOUND_MAX_AHEAD = 0.3;
function playSound({ samples, rate }) {
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
    if (event.code === 'Enter' || event.code === 'Space') {
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

overlay.addEventListener('click', start);
canvas.addEventListener('dblclick', () => {
  if (document.fullscreenElement) document.exitFullscreen();
  else canvas.requestFullscreen();
});
