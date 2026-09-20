// Runs the game. Everything the game shows or plays is sent to the page (main.js) as it happens,
// and the page's keys arrive as messages between two ticks.
import init, { Doom } from './pkg/doomgeneric_wasm.js';

// One tic of game time, 1/35 s.
const TIC_MS = 1000 / 35;

let doom = null;
let stopped = false;
let consoleLine = '';

// What the game calls (see `Host` in src/platform.rs). The typed arrays it passes are views into
// the module's memory that are only valid during the call, so they are copied.
const host = {
  present(rgba, width, height) {
    const pixels = rgba.slice().buffer;
    postMessage({ type: 'frame', pixels, width, height }, [pixels]);
  },
  audio(samples, rate) {
    const copy = samples.slice();
    postMessage({ type: 'audio', samples: copy, rate }, [copy.buffer]);
  },
  now() {
    return performance.now();
  },
  log(message, error) {
    // The game prints in pieces; the console wants whole lines.
    consoleLine += message;
    const lines = consoleLine.split('\n');
    consoleLine = lines.pop();
    for (const line of lines) (error ? console.error : console.log)(line);
  },
  quit() {
    stopped = true;
    postMessage({ type: 'quit' });
  },
};

function fail(error) {
  stopped = true;
  console.error(error);
  postMessage({ type: 'error', message: String(error) });
}

function loop() {
  if (stopped) return;
  const start = performance.now();
  try {
    // Waits for the next tic if it is not due yet, then runs and draws it.
    doom.tick();
  } catch (error) {
    // `quit` above has already said the game is over.
    if (!stopped) fail(error);
    return;
  }
  setTimeout(loop, Math.max(0, TIC_MS - (performance.now() - start)));
}

async function start() {
  try {
    await init();
    doom = new Doom(host);
  } catch (error) {
    fail(error);
    return;
  }
  postMessage({ type: 'ready' });
  loop();
}

onmessage = ({ data }) => {
  if (data.type === 'start') {
    start();
  } else if (data.type === 'key' && doom && !stopped) {
    doom.key_event(data.pressed, data.code);
  }
};
