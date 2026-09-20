// Runs the game. Everything the game shows or plays is sent to the page (main.js) as it happens,
// and the page's keys arrive as messages between two ticks.
import init, { Setup } from './pkg/doomgeneric_wasm.js';
import * as storage from './storage.js';

// One tic of game time, 1/35 s.
const TIC_MS = 1000 / 35;

let doom = null;
let stopped = false;
let consoleLine = '';
// While the page is hidden the game stands still, and so does its clock: the time spent paused is
// taken off, or the game would run the missed tics all at once when it goes on.
let paused = false;
let pausedAt = 0;
let pausedTotal = 0;
let timer = null;
// Where the files of the running game are kept: see `Setup.storage_key`.
let filesPrefix = '';

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
    return performance.now() - pausedTotal;
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
  store(path, data) {
    storage.put('files', filesPrefix + path, data.slice());
  },
  remove(path) {
    storage.remove('files', filesPrefix + path);
  },
};

function fail(error, badWad = false) {
  stopped = true;
  console.error(error);
  postMessage({ type: 'error', message: error?.message ?? String(error), badWad });
}

function loop() {
  timer = null;
  if (stopped || paused) return;
  const start = performance.now();
  try {
    // Waits for the next tic if it is not due yet, then runs and draws it.
    doom.tick();
  } catch (error) {
    // `quit` above has already said the game is over.
    if (!stopped) fail(error);
    return;
  }
  timer = setTimeout(loop, Math.max(0, TIC_MS - (performance.now() - start)));
}

async function start(wad) {
  let setup;
  try {
    await init();
    try {
      setup = new Setup(new Uint8Array(wad));
    } catch (error) {
      fail(error, true);
      return;
    }
    filesPrefix = `${setup.storage_key()}/`;
    for (const [key, data] of await storage.entries('files', filesPrefix)) {
      setup.add_file(key.slice(filesPrefix.length), data);
    }
    doom = setup.start(host);
  } catch (error) {
    fail(error);
    return;
  }
  postMessage({ type: 'ready' });
  loop();
}

onmessage = ({ data }) => {
  switch (data.type) {
    case 'start':
      start(data.wad);
      break;
    case 'key':
      if (doom && !stopped) doom.key_event(data.pressed, data.code);
      break;
    case 'pause':
      if (!paused) {
        paused = true;
        pausedAt = performance.now();
        clearTimeout(timer);
        timer = null;
      }
      break;
    case 'resume':
      if (paused) {
        paused = false;
        pausedTotal += performance.now() - pausedAt;
        // Before the game has started there is no loop to restart; `start` begins it.
        if (doom && !stopped && timer === null) loop();
      }
      break;
  }
};
