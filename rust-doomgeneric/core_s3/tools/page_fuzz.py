"""Fuzzes the web controller page in headless Chromium against a local stand-in for the board.

Random keys (in the text box and outside it), soft-keyboard typing and compositions, clicks and
multi-finger holds on the buttons, blurs and mode switches; then checks that nothing is left held or
stuck, that every press the "board" received got its release, and that a click on Forward still works.
A stuck hold is exactly "the buttons stop working". Exits 1 on any failure.

usage: python3 page_fuzz.py [PAGE.html] [seeds] [single]
  (single: at most one finger per button, to tell multi-touch problems from the rest)
Needs chromium on PATH and the Python `websockets` package (>= 13)."""
import asyncio, json, subprocess, time, urllib.request, os, sys, shutil
import websockets
from websockets.asyncio.server import serve
from websockets.http11 import Response
from websockets.datastructures import Headers

PAGE = open(sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'assets', 'controller.html'), 'rb').read()
SEEDS = int(sys.argv[2]) if len(sys.argv) > 2 else 40
SINGLE = len(sys.argv) > 3 and sys.argv[3] == 'single'
import tempfile
SCR = tempfile.mkdtemp(prefix="core_s3_page_")
PORT = 9445
def process_request(connection, request):
    if request.path == '/ws': return None
    return Response(200, 'OK', Headers([('Content-Type', 'text/html; charset=utf-8'), ('Content-Length', str(len(PAGE)))]), PAGE)
async def handler(ws):
    async for m in ws: pass
profile = f"{SCR}/profile2"; shutil.rmtree(profile, ignore_errors=True)
chrome = subprocess.Popen(["chromium", "--headless=new", f"--remote-debugging-port={PORT}", f"--user-data-dir={profile}",
    "--no-sandbox", "--disable-gpu", "--no-first-run", "about:blank"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
def fetch(path, method="GET"):
    return json.load(urllib.request.urlopen(urllib.request.Request(f"http://127.0.0.1:{PORT}{path}", method=method), timeout=5))
for _ in range(50):
    try: fetch("/json/version"); break
    except Exception: time.sleep(0.2)
tab = fetch("/json/new?about:blank", "PUT")

FUZZ = r"""
(async (seed, steps, SINGLE) => {
  let s = seed >>> 0;
  const rnd = () => { s = (s * 1664525 + 1013904223) >>> 0; return s / 4294967296; };
  const pick = (a) => a[Math.floor(rnd() * a.length)];
  const sends = [];
  const orig = WebSocket.prototype.send;
  WebSocket.prototype.send = function (d) { sends.push(Array.from(new Uint8Array(d))); return orig.call(this, d); };
  Element.prototype.setPointerCapture = () => {};
  const box = document.getElementById('text');
  const codes = ['KeyW','KeyA','KeyS','KeyD','KeyQ','KeyE','KeyF','KeyM','Space','ArrowUp','ArrowDown','Enter','Escape','Tab','Backspace','ShiftLeft','Digit1','Digit2','KeyH','Comma'];
  const keyOf = (c) => ({Space:' ',ArrowUp:'ArrowUp',ArrowDown:'ArrowDown',Enter:'Enter',Escape:'Escape',Tab:'Tab',Backspace:'Backspace',ShiftLeft:'Shift'}[c] || (c.startsWith('Key') ? c.slice(3).toLowerCase() : c.startsWith('Digit') ? c.slice(5) : ','));
  const down = new Set();
  const buttons = [...document.querySelectorAll('button[data-a]')];
  const held = new Set();
  const kd = (target, code, repeat, soft) => target.dispatchEvent(new KeyboardEvent('keydown', {key: soft ? 'Unidentified' : keyOf(code), code: soft ? '' : code, keyCode: soft ? 229 : 0, repeat, bubbles: true, cancelable: true}));
  const ku = (target, code, soft) => target.dispatchEvent(new KeyboardEvent('keyup', {key: soft ? 'Unidentified' : keyOf(code), code: soft ? '' : code, bubbles: true, cancelable: true}));
  const type = (text, composing) => { box.value += text; box.dispatchEvent(new InputEvent('input', {data: text, inputType: 'insertText', isComposing: composing, bubbles: true})); };
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  let composing = false;
  for (let i = 0; i < steps; i++) {
    const r = rnd();
    const target = rnd() < 0.5 ? box : document.body;
    if (r < 0.20) { const c = pick(codes); if (!down.has(c)) { down.add(c); kd(target, c, false, false); } else kd(target, c, true, false); }
    else if (r < 0.40) { const c = pick([...down, ...codes]); down.delete(c); ku(pick([box, document.body]), c, false); }
    else if (r < 0.50) { type(pick(['w','a','d',' ','iddqd','wwwd','1','Q','é','\n'])); }
    else if (r < 0.55) { if (!composing) { composing = true; box.dispatchEvent(new CompositionEvent('compositionstart')); } type(pick(['w','a','s']), true); }
    else if (r < 0.60) { if (composing) { composing = false; box.dispatchEvent(new CompositionEvent('compositionend', {data: box.value})); } }
    else if (r < 0.70) { const b = pick(buttons); if (SINGLE && [...held].some(x => x[0] === b)) continue; const id = 1 + Math.floor(rnd()*3); b.dispatchEvent(new PointerEvent('pointerdown', {pointerId: id, bubbles: true, cancelable: true})); held.add([b, id]); }
    else if (r < 0.80) { if (held.size) { const [b, id] = pick([...held]); held.delete([...held].find(x => x[0] === b && x[1] === id)); b.dispatchEvent(new PointerEvent(pick(['pointerup','pointercancel']), {pointerId: id, bubbles: true})); } }
    else if (r < 0.83) { window.dispatchEvent(new Event('blur')); down.clear(); held.clear(); }
    else if (r < 0.86) { document.querySelector('input[name=mode][value=' + pick(['keys','text']) + ']').click(); }
    else if (r < 0.88) { box.focus(); }
    else if (r < 0.90) { document.getElementById('run').click(); }
    await wait(pick([0, 1, 5, 20, 60, 150]));
  }
  // everything comes up: release whatever the fuzzer still holds, the way a person would
  for (const [b, id] of held) b.dispatchEvent(new PointerEvent('pointerup', {pointerId: id, bubbles: true}));
  for (const c of down) ku(document.body, c, false);
  if (composing) box.dispatchEvent(new CompositionEvent('compositionend', {data: box.value}));
  if (document.getElementById('run').classList.contains('on')) document.getElementById('run').click();
  await wait(1500 + 300 * 0);   // longest hold timer + the text pump / macro
  await wait(3000);
  const state = {held: count.map((n,i)=>n?[i,n]:null).filter(Boolean), pending: pending.map((p,i)=>p!==null?i:null).filter(x=>x!==null), keysDown: [...keysDown.keys()], pumping, qlen: charQueue.length};
  // balance per action code: presses minus releases of non-char codes (0..31)
  const bal = {};
  for (const m of sends) for (const b of m) { if ((b & 0x7f) < 32) { const a = b & 0x7f; bal[a] = (bal[a] || 0) + (b & 0x80 ? -1 : 1); } }
  const unbalanced = Object.entries(bal).filter(([, v]) => v !== 0);
  // and still alive: a click on Forward must send press and release
  const before = sends.length;
  const fwd = document.querySelector('button[data-a="0"]');
  fwd.dispatchEvent(new PointerEvent('pointerdown', {pointerId: 9, bubbles: true, cancelable: true}));
  await wait(20);
  fwd.dispatchEvent(new PointerEvent('pointerup', {pointerId: 9, bubbles: true}));
  await wait(200);
  const alive = JSON.stringify(sends.slice(before));
  return JSON.stringify({sends: sends.length, state, unbalanced, alive});
})
"""

async def main():
    fails = 0
    async with serve(handler, '127.0.0.1', 8099, process_request=process_request):
        async with websockets.connect(tab["webSocketDebuggerUrl"], max_size=None) as ws:
            n = 0
            async def call(method, **params):
                nonlocal n
                n += 1; mid = n
                await ws.send(json.dumps({"id": mid, "method": method, "params": params}))
                while True:
                    msg = json.loads(await ws.recv())
                    if msg.get("id") == mid: return msg
            await call("Page.enable"); await call("Runtime.enable")
            for seed in range(1, SEEDS + 1):
                await call("Page.navigate", url="http://127.0.0.1:8099/")
                for _ in range(100):
                    r = await call("Runtime.evaluate", expression="document.getElementById('state') && document.getElementById('state').textContent", returnByValue=True)
                    if r['result']['result'].get('value') == 'connected': break
                    await asyncio.sleep(0.05)
                r = await call("Runtime.evaluate", expression=f"({FUZZ.strip()})({seed}, 250, {'true' if SINGLE else 'false'})", awaitPromise=True, returnByValue=True)
                res = r['result']
                if 'exceptionDetails' in res:
                    print(seed, 'EXCEPTION', res['exceptionDetails']); fails += 1; continue
                out = json.loads(res['result']['value'])
                bad = out['state']['held'] or out['state']['pending'] or out['state']['keysDown'] or out['state']['pumping'] or out['state']['qlen'] or out['unbalanced'] or '[0]' not in out['alive'] or '[128]' not in out['alive']
                if bad:
                    fails += 1; print(seed, 'STUCK', out)
    print('seeds', SEEDS, 'failures', fails)
    return fails
try:
    failures = asyncio.run(main())
finally:
    chrome.terminate()
sys.exit(1 if failures else 0)
