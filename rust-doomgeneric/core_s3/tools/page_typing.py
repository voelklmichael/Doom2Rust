"""Measures how long the web controller page takes to turn typing into WebSocket bytes, in headless
Chromium against a local stand-in for the board: a hardware key in the text box, a soft-keyboard
character, a word composed on a phone keyboard, a paste, Text mode, and typing followed by button
clicks. Prints a timeline per scenario (milliseconds from the first event) and the page's hold state.

usage: python3 page_typing.py [PAGE.html] [scenario ...]
  scenarios: desk single comp paste text textsingle mixA mixB mixC mixD (default: mixA-mixD)
Needs chromium on PATH and the Python `websockets` package (>= 13)."""
import asyncio, json, subprocess, time, urllib.request, os, sys, shutil
import websockets
from websockets.asyncio.server import serve
from websockets.http11 import Response
from websockets.datastructures import Headers

PAGE = open(sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'assets', 'controller.html'), 'rb').read()
import tempfile
SCR = tempfile.mkdtemp(prefix="core_s3_page_")
PORT = 9444

def process_request(connection, request):
    if request.path == '/ws': return None
    return Response(200, 'OK', Headers([('Content-Type', 'text/html; charset=utf-8'), ('Content-Length', str(len(PAGE)))]), PAGE)

async def handler(ws):
    async for m in ws: pass

profile = f"{SCR}/profile"
shutil.rmtree(profile, ignore_errors=True)
chrome = subprocess.Popen(["chromium", "--headless=new", f"--remote-debugging-port={PORT}", f"--user-data-dir={profile}",
    "--no-sandbox", "--disable-gpu", "--no-first-run", "--window-size=900,1000", "about:blank"],
    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
def fetch(path, method="GET"):
    req = urllib.request.Request(f"http://127.0.0.1:{PORT}{path}", method=method)
    return json.load(urllib.request.urlopen(req, timeout=5))
for _ in range(50):
    try: fetch("/json/version"); break
    except Exception: time.sleep(0.2)
tab = fetch("/json/new?about:blank", "PUT")

class Cdp:
    def __init__(self, ws): self.ws, self.n = ws, 0
    async def call(self, method, **params):
        self.n += 1; mid = self.n
        await self.ws.send(json.dumps({"id": mid, "method": method, "params": params}))
        while True:
            msg = json.loads(await self.ws.recv())
            if msg.get("id") == mid:
                if "error" in msg: raise RuntimeError(f"{method}: {msg['error']}")
                return msg["result"]
    async def js(self, expr):
        r = await self.call("Runtime.evaluate", expression=expr, returnByValue=True, awaitPromise=True)
        if "exceptionDetails" in r: raise RuntimeError(r["exceptionDetails"])
        return r["result"].get("value")

HOOK = """
window.__log = [];
const t0 = performance.now();
const stamp = () => performance.now();
const s = WebSocket.prototype.send;
WebSocket.prototype.send = function(d){ window.__log.push(['send', stamp(), Array.from(new Uint8Array(d))]); return s.call(this, d); };
for (const type of ['keydown','keyup','input','compositionstart','compositionupdate','compositionend'])
  addEventListener(type, (e) => window.__log.push([type, stamp(), e.key || e.data || '', e.isComposing ? 'C' : '']), true);
"""

async def scenario(c, name, mode, actions):
    """actions: list of (delay_ms_before, cdp_method, params)."""
    await c.call("Page.navigate", url="http://127.0.0.1:8099/")
    for _ in range(100):
        if await c.js("document.getElementById('state') && document.getElementById('state').textContent") == "connected": break
        await asyncio.sleep(0.05)
    await c.js(f"document.querySelector('input[name=mode][value={mode}]').click()")
    await c.js("document.getElementById('text').focus()")
    await c.js("window.__log = []")
    for delay, method, params in actions:
        await asyncio.sleep(delay / 1000)
        await c.call(method, **params)
    await asyncio.sleep(1.2)
    st = await c.js("JSON.stringify({held: count.map((n,i)=>n?[i,n]:null).filter(Boolean), pending: pending.map((p,i)=>p!==null?i:null).filter(x=>x!==null), keysDown: [...keysDown.keys()], pumping, qlen: charQueue.length})")
    log = await c.js("window.__log")
    t_first = log[0][1] if log else 0
    print(f"--- {name} ({mode} mode)")
    for e in log:
        if e[0] == 'send':
            print(f"  {e[1]-t_first:7.1f} ms  SEND {e[2]}")
        else:
            print(f"  {e[1]-t_first:7.1f} ms  {e[0]:17s} {e[2]!r} {e[3]}")
    print("  state after:", st)

def key(t, k, code, vk):
    p = dict(type=t, key=k, code=code, windowsVirtualKeyCode=vk)
    if t == 'keyDown' and len(k) == 1: p['text'] = k
    return ("Input.dispatchKeyEvent", p)

def click(x, y, hold=50):
    return [(0, "Input.dispatchMouseEvent", dict(type='mouseMoved', x=x, y=y)),
            (0, "Input.dispatchMouseEvent", dict(type='mousePressed', x=x, y=y, button='left', clickCount=1)),
            (hold, "Input.dispatchMouseEvent", dict(type='mouseReleased', x=x, y=y, button='left', clickCount=1))]

async def main():
    async with serve(handler, '127.0.0.1', 8099, process_request=process_request):
        async with websockets.connect(tab["webSocketDebuggerUrl"], max_size=None) as ws:
            c = Cdp(ws)
            await c.call("Page.enable"); await c.call("Runtime.enable")
            await c.call("Page.addScriptToEvaluateOnNewDocument", source=HOOK)
            await c.call("Emulation.setDeviceMetricsOverride", width=390, height=900, deviceScaleFactor=2, mobile=True)
            which = sys.argv[2:] or ['desk', 'single', 'comp', 'paste', 'text', 'textsingle', 'mixA', 'mixB', 'mixC', 'mixD']
            async def center(sel):
                return await c.js(f"(()=>{{const r=document.querySelector({json.dumps(sel)}).getBoundingClientRect();return [r.x+r.width/2,r.y+r.height/2]}})()")
            async def btn(a):
                sel = 'button[data-a="%s"]' % a
                await c.js("document.querySelector(%s).scrollIntoView({block:'center'})" % json.dumps(sel))
                return await center(sel)
            async def with_buttons(name, mode, acts):
                # like scenario(), but the click coordinates are found after the page has loaded
                await c.call("Page.navigate", url="http://127.0.0.1:8099/")
                for _ in range(100):
                    if await c.js("document.getElementById('state') && document.getElementById('state').textContent") == "connected": break
                    await asyncio.sleep(0.05)
                await c.js(f"document.querySelector('input[name=mode][value={mode}]').click()")
                await c.js("window.__log = []")
                for delay, method, params in acts:
                    if method == 'CLICK':
                        x, y = await btn(params)
                        for d2, m2, p2 in click(x, y):
                            await asyncio.sleep(d2/1000); await c.call(m2, **p2)
                    elif method == 'FOCUSBOX':
                        await c.js("document.getElementById('text').focus()")
                    else:
                        await asyncio.sleep(delay/1000); await c.call(method, **params)
                await asyncio.sleep(0.6)
                st = await c.js("JSON.stringify({held: count.map((n,i)=>n?[i,n]:null).filter(Boolean), pending: pending.map((p,i)=>p!==null?i:null).filter(x=>x!==null), keysDown: [...keysDown.keys()], pumping, qlen: charQueue.length})")
                log = await c.js("window.__log")
                t0 = log[0][1] if log else 0
                print(f"--- {name} ({mode})")
                for e in log:
                    if e[0] == 'send': print(f"  {e[1]-t0:7.1f} ms  SEND {e[2]}")
                    else: print(f"  {e[1]-t0:7.1f} ms  {e[0]:17s} {e[2]!r} {e[3]}")
                print("  state after:", st)
            if 'mixA' in which:   # type in the box with a hardware keyboard, then click Forward, then press ArrowUp on the page
                await with_buttons("A: box keys w, space; then click Forward; then ArrowUp key", "keys", [
                    (0, 'FOCUSBOX', None),
                    (0, *key('keyDown','w','KeyW',87)), (50, *key('keyUp','w','KeyW',87)),
                    (60, *key('keyDown',' ','Space',32)), (50, *key('keyUp',' ','Space',32)),
                    (100, 'CLICK', 0),
                    (100, *key('keyDown','ArrowUp','ArrowUp',38)), (150, *key('keyUp','ArrowUp','ArrowUp',38))])
            if 'mixB' in which:   # soft keyboard char in the box, then buttons
                await with_buttons("B: insertText w, then click Forward twice", "keys", [
                    (0, 'FOCUSBOX', None), (0, "Input.insertText", dict(text='w')),
                    (300, 'CLICK', 0), (100, 'CLICK', 0)])
            if 'mixC' in which:
                await with_buttons("C: text mode 'hello', then click Forward, Fire", "text", [
                    (0, 'FOCUSBOX', None), (0, "Input.insertText", dict(text='hello')),
                    (300, 'CLICK', 0), (100, 'CLICK', 6)])
            if 'mixD' in which:   # hold w in box while clicking Forward
                await with_buttons("D: w held in box, click Forward while held, then release w", "keys", [
                    (0, 'FOCUSBOX', None), (0, *key('keyDown','w','KeyW',87)),
                    (100, 'CLICK', 0), (100, *key('keyUp','w','KeyW',87)),
                    (200, 'CLICK', 0)])
            if 'desk' in which:   # hardware keyboard in the box, Keys mode: w down, up 40 ms later
                await scenario(c, "desktop keys w tap 40 ms", "keys", [
                    (0, *key('keyDown', 'w', 'KeyW', 87)), (40, *key('keyUp', 'w', 'KeyW', 87))])
            if 'single' in which:  # soft keyboard, no composition: one committed character
                await scenario(c, "soft keyboard, one char w", "keys", [(0, "Input.insertText", dict(text='w'))])
            if 'comp' in which:   # soft keyboard with word composition: w, ww, www, wwwd typed 120 ms apart, then space commits
                acts = [(0, "Input.imeSetComposition", dict(text='w', selectionStart=1, selectionEnd=1))]
                for t in ['ww', 'www', 'wwwd']:
                    acts.append((120, "Input.imeSetComposition", dict(text=t, selectionStart=len(t), selectionEnd=len(t))))
                acts.append((300, "Input.insertText", dict(text='wwwd ')))
                await scenario(c, "soft keyboard composition wwwd + space", "keys", acts)
            if 'paste' in which:
                await scenario(c, "paste wwwd", "keys", [(0, "Input.insertText", dict(text='wwwd'))])
            if 'text' in which:
                await scenario(c, "type iddqd at once", "text", [(0, "Input.insertText", dict(text='iddqd'))])
            if 'textsingle' in which:   # five chars typed 80 ms apart, like fast thumbs
                await scenario(c, "type i d d q d, 80 ms apart", "text",
                               [(0 if i == 0 else 80, "Input.insertText", dict(text=ch)) for i, ch in enumerate('iddqd')])
try:
    asyncio.run(main())
finally:
    chrome.terminate()
