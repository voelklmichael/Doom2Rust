import re
def code_mask(s):
    """Return list of bools: True where char is real code (not string/char/comment)."""
    n=len(s); m=[True]*n; i=0
    while i<n:
        c=s[i]
        if c=='/' and i+1<n and s[i+1]=='/':
            j=s.find('\n',i)
            j=n if j<0 else j
            for k in range(i,j): m[k]=False
            i=j
        elif c=='/' and i+1<n and s[i+1]=='*':
            j=s.find('*/',i+2); j=n if j<0 else j+2
            for k in range(i,j): m[k]=False
            i=j
        elif c=='r' and re.match(r'r#*"',s[i:i+8]) and (i==0 or not (s[i-1].isalnum() or s[i-1]=='_')):
            h=re.match(r'r(#*)"',s[i:]).group(1)
            end=s.find('"'+h,i+2+len(h))
            end=n if end<0 else end+1+len(h)
            for k in range(i,end): m[k]=False
            i=end
        elif c=='"':
            j=i+1
            while j<n and s[j]!='"':
                j+=2 if s[j]=='\\' else 1
            j=min(n,j+1)
            for k in range(i,j): m[k]=False
            i=j
        elif c=="'":
            # char literal?  'x' or '\n' or '\u{..}' ; else lifetime
            mm=re.match(r"'(\\.[^']*|[^\\'])'",s[i:i+12])
            if mm:
                for k in range(i,i+mm.end()): m[k]=False
                i+=mm.end()
            else: i+=1
        else: i+=1
    return m
def match_brace(s,m,i):
    assert s[i]=='{'
    d=0
    for j in range(i,len(s)):
        if not m[j]: continue
        if s[j]=='{': d+=1
        elif s[j]=='}':
            d-=1
            if d==0: return j
    raise Exception('unbalanced')

def region_mask(s):
    """0 = code, 1 = comment, 2 = string/char literal."""
    n=len(s); m=[0]*n; i=0
    while i<n:
        c=s[i]
        if c=='/' and i+1<n and s[i+1]=='/':
            j=s.find('\n',i); j=n if j<0 else j
            for k in range(i,j): m[k]=1
            i=j
        elif c=='/' and i+1<n and s[i+1]=='*':
            j=s.find('*/',i+2); j=n if j<0 else j+2
            for k in range(i,j): m[k]=1
            i=j
        elif c=='r' and re.match(r'r#*"',s[i:i+8]) and (i==0 or not (s[i-1].isalnum() or s[i-1]=='_')):
            h=re.match(r'r(#*)"',s[i:]).group(1)
            end=s.find('"'+h,i+2+len(h)); end=n if end<0 else end+1+len(h)
            for k in range(i,end): m[k]=2
            i=end
        elif c=='"':
            j=i+1
            while j<n and s[j]!='"': j+=2 if s[j]=='\\' else 1
            j=min(n,j+1)
            for k in range(i,j): m[k]=2
            i=j
        elif c=="'":
            mm=re.match(r"'(\\.[^']*|[^\\'])'",s[i:i+12])
            if mm:
                for k in range(i,i+mm.end()): m[k]=2
                i+=mm.end()
            else: i+=1
        else: i+=1
    return m
