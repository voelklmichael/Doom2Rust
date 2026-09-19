"""Narrow functions that take `state: &mut GameState` to just the subsystems they use.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/narrow_state.py            # dry run: list candidates
    MAXF=3 python3 tools/narrow_state.py --apply

A function is a candidate when its body reads at most MAXF distinct
`state.<field>` paths (default 3) and never passes `state` itself on. Its
parameter becomes one `&mut`/`&` per field, callers pass `&mut state.field`,
and names used as values (callbacks) or defined twice are skipped. Narrowing a
leaf makes its callers narrower, so re-run until it reports 0 candidates, then
fix what the compiler reports (usually a call argument that reads a field the
call also borrows: hoist it into a local) and run `cargo clippy --fix` plus
`cargo fmt`. See docs/gamestate-decomposition.md.
"""
import re,glob,sys,os,collections,json
sys.path.insert(0,os.path.dirname(os.path.abspath(__file__)))
from rslex import *
SKIP_FIELDS={'platform','fs'}
MAXF=int(os.environ.get('MAXF','3'))
gs=open('engine/src/game_state.rs').read()
ftype=dict(re.findall(r'pub (\w+): (\w+State|\w+),',gs))
blacklist=set(json.load(open(os.path.dirname(os.path.abspath(__file__))+'/narrow_blacklist.json'))) if os.path.exists(os.path.dirname(os.path.abspath(__file__))+'/narrow_blacklist.json') else set()
files=[f for f in sorted(glob.glob('engine/src/*.rs')) if not f.endswith(('regression_tests.rs','game_state.rs'))]
src={f:open(f).read() for f in files}
TESTF='engine/src/regression_tests.rs'
src[TESTF]=open(TESTF).read()
allsrc='\n'.join(src.values())
def used_as_value(name):
    # appears as identifier not followed by '(' and not in fn definition
    for m in re.finditer(r'(?<![\w.])'+re.escape(name)+r'\b(?!\s*\()',allsrc):
        pre=allsrc[max(0,m.start()-4):m.start()]
        if pre.endswith('fn '): continue
        # in use statements: ignore
        ls=allsrc.rfind('\n',0,m.start())+1; line=allsrc[ls:allsrc.find('\n',m.start())]
        if re.match(r'\s*(pub(\(crate\))? )?use ',line) or re.match(r'\s+[\w:{}, ]+,?$',line) and False: continue
        if line.strip().startswith('//'): continue
        return True
    return False
cands={}
for f in files:
    s=src[f]; m=code_mask(s)
    for h in re.finditer(r'(?m)^(pub(?:\(crate\))? )?fn (\w+)(?:<[^>]*>)?\(\s*state: &(mut )?GameState',s):
        name=h.group(2)
        if name in blacklist: continue
        b=s.index('{',s.index(')',h.end()))
        try: e=match_brace(s,m,b)
        except: continue
        body=s[b:e]
        if re.search(r'(?<![\w.])state(?![\w.])',body): continue
        fields=set(re.findall(r'(?<![\w])state\.(\w+)',body))
        if not (1<=len(fields)<=MAXF): continue
        if any(X in SKIP_FIELDS or X not in ftype for X in fields): continue
        X=tuple(sorted(fields))
        # mutability: does the body need &mut? keep as declared
        cands[name]=(f,X,bool(h.group(3)))
# drop those used as values
cands={n:v for n,v in cands.items() if not used_as_value(n)}
# duplicate names in different files: skip ambiguous
cnt=collections.Counter(re.findall(r'(?m)^(?:pub(?:\(crate\))? )?fn (\w+)',allsrc))
cands={n:v for n,v in cands.items() if cnt[n]==1}
print('candidates',len(cands))
if '--apply' not in sys.argv:
    for n,(f,X,mu) in sorted(cands.items()): print(' ',f.split('/')[-1],n,','.join(X),'mut' if mu else '')
    sys.exit()
# apply: definitions
typemod={}
for n,(f,Xs,mu) in cands.items():
    s=src[f]; m=code_mask(s)
    h=re.search(r'(?m)^(pub(?:\(crate\))? )?fn '+n+r'(?:<[^>]*>)?\(\s*state: &(mut )?GameState',s)
    b=s.index('{',s.index(')',h.end())); e=match_brace(s,m,b)
    body=s[b:e+1]
    for X in Xs:
        body=re.sub(r'&mut state\.'+X+r'(?![\w.])',X,body)
        body=re.sub(r'&state\.'+X+r'(?![\w.])',X,body)
        body=re.sub(r'(?<![\w])state\.'+X+r'(?![\w])',X,body)
    kw='&mut ' if mu else '&'
    params=', '.join(f'{X}: {kw}{ftype[X]}' for X in Xs)
    head=s[h.start():h.end()].replace('state: &mut GameState',params).replace('state: &GameState',params)
    src[f]=s[:h.start()]+head+s[h.end():b]+body+s[e+1:]
    for X in Xs: typemod[(f,ftype[X])]=X
# callers
for n,(f0,Xs,mu) in cands.items():
    rx=re.compile(r'(?<![\w.])'+n+r'\(\s*state\b(?!\.)')
    args=', '.join((('&mut state.'+X) if mu else ('&state.'+X)) for X in Xs)
    for f in files+[TESTF]:
        s=src[f]
        s2=rx.sub(lambda m:m.group(0)[:-5]+args,s)
        if s2!=s: src[f]=s2
# imports for the types
tmod={}
for f in files: pass
for (f,T),X in typemod.items():
    s=src[f]
    if re.search(r'\b'+T+r'\b',s) and not re.search(r'use crate::\w+::[^;]*\b'+T+r'\b',s) and not re.search(r'(?m)^(pub )?struct '+T+r'\b',s):
        mod=X  # module named like the field
        s=re.sub(r'(?m)^use [^\n]*\n',lambda mm:f'use crate::{mod}::{T};\n'+mm.group(0),s,count=1)
        src[f]=s
for f,s in src.items(): open(f,'w').write(s)
print('applied',len(cands))
