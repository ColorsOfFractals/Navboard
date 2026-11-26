import sys, json, threading, queue, time
import numpy as np
import soundcard as sc
import sounddevice as sd
from math import gcd
from contextlib import contextmanager
from scipy.signal import resample_poly
from faster_whisper import WhisperModel
import re

# Conservative prototype: suppress entire matching segment, not just the token.
WORDS = r"zero|oh|one|two|three|four|five|six|seven|eight|nine"
NATO = r"alpha|alfa|bravo|charlie|delta|echo|echoromio|romeo|foxtrot|golf|hotel|india|juliet|juliett|kilo|lima|mike|november|oscar|papa|quebec|sierra|tango|uniform|victor|whiskey|xray|yankee|zulu"
RULES = [
    re.compile(r"\b(?:password|passcode|passphrase|pin|otp|verification code|security code|recovery code|credit card|card number|api key|secret key)\b", re.I),
    re.compile(r"\b(?:capital|uppercase|lowercase|lower case|upper case)\b", re.I),
    re.compile(r"(?<!\w)[A-Za-z](?:[\s,.-]+[A-Za-z]){1,}(?!\w)"),
    re.compile(r"\b(?:"+NATO+r")(?:[\s,.-]+(?:"+NATO+r"))+(?!\w)", re.I),
    re.compile(r"\b(?:"+WORDS+r"|\d)(?:[\s,.-]+(?:"+WORDS+r"|\d))+(?!\w)", re.I),
    re.compile(r"\d{4,}"),
]
# Hard vocabulary: deliberately conservative for spoken credential spelling.
HARD = re.compile(r"\b(?:exclamation[\s-]+(?:mark|point)|dollar[\s-]+sign|ampersand|asterisk|capital[\s-]+letter|case[\s-]+sensitive|as[\s-]+in[\s-]+(?:"+NATO+r")|upper[\s-]*case|lower[\s-]*case)\b", re.I)
RULES.append(HARD)
def risky(text):
    return any(r.search(text) for r in RULES)

class Shield:
    def __init__(self):
        self.pending = {}
        self.carry = {}
    def clear(self):
        self.pending.clear(); self.carry.clear()
    def push(self, ch, block):
        prev = self.pending.pop(ch, [])
        if not block:
            return self.release(ch, prev, [])
        self.pending[ch] = block
        return self.release(ch, prev, block)
    def release(self, ch, prev, following):
        if not prev: return []
        # Include next block for boundary detection. Conservative whole-block suppression.
        before = ' '.join(s['text'] for s in prev)
        after = ' '.join(s['text'] for s in following)
        cross = risky(before+' '+after) and not risky(before) and not risky(after)
        # Carry one block after any detected credential cue or boundary match.
        cue = bool(RULES[0].search(before) or HARD.search(before))
        redact = risky(before) or cross or self.carry.pop(ch, False)
        self.carry[ch] = cue or cross
        if redact:
            return [dict(prev[0], end_ms=prev[-1]['end_ms'], text='[REDACTED - PRIVACY SHIELD]')]
        return prev

def self_test():
    def b(t): return [dict(channel='INPUT',start_ms=0,end_ms=5000,text=t)]
    for t in ['D E R P','Delta, Echoromio, Papa.','Capital B, lowercase K','My temporary password is purple banana seven','One, two, three, four.','5, 6, the DNS server is not responding.']:
        s=Shield(); assert s.push('INPUT',b(t))==[]; assert 'REDACTED' in s.push('INPUT',[])[0]['text']
    s=Shield(); s.push('INPUT',b('DNS, DHCP, SMB, TCP')); assert s.push('INPUT',[])[0]['text']=='DNS, DHCP, SMB, TCP'
    s=Shield(); s.push('INPUT',b('D')); assert 'REDACTED' in s.push('INPUT',b('E'))[0]['text']; assert 'REDACTED' in s.push('INPUT',[])[0]['text']
    s=Shield(); s.push('INPUT',b('My password is')); s.push('OUTPUT',b('ordinary speech')); assert s.push('OUTPUT',[])[0]['text']=='ordinary speech'; assert 'REDACTED' in s.push('INPUT',b('purple banana'))[0]['text']; assert 'REDACTED' in s.push('INPUT',[])[0]['text']
    for t in ['case K7, exclamation mark.', 'exclamation point', 'dollar sign', 'ampersand', 'asterisk', 'case sensitive', 'as in Bravo', 'lower-case K']:
        s=Shield(); s.push('INPUT',b(t)); assert 'REDACTED' in s.push('INPUT',[])[0]['text']
    for t in ['The seven port switch is online.', 'DNS server is not responding.']:
        s=Shield(); s.push('INPUT',b(t)); assert s.push('INPUT',[])[0]['text']==t
    s=Shield(); s.push('INPUT',b('exclamation')); assert 'REDACTED' in s.push('INPUT',b('mark'))[0]['text']; assert 'REDACTED' in s.push('INPUT',[])[0]['text']
    s=Shield(); s.push('INPUT',b('exclamation mark')); assert 'REDACTED' in s.push('INPUT',b('fictional continuation'))[0]['text']; assert 'REDACTED' in s.push('INPUT',[])[0]['text']
    print('Privacy Shield: 21 test scenarios passed')

shield=Shield()
stop=threading.Event(); paused=threading.Event(); lock=threading.Lock(); output=threading.Lock()
generation=0; audio=queue.Queue(maxsize=12); origin=time.monotonic()
def emit(**obj):
    with output: print(json.dumps(obj,ensure_ascii=True),flush=True)
def commands():
    global generation
    for line in sys.stdin:
        cmd=line.strip()
        with lock:
            if cmd=='stop': stop.set(); return
            if cmd in ('pause','resume'):
                generation+=1
                shield.clear()
                if cmd=='pause': paused.set()
                else: paused.clear()
                while True:
                    try: audio.get_nowait()
                    except queue.Empty: break
    stop.set()
@contextmanager
def microphone_stream():
    device=sd.query_devices(kind='input')
    rate=int(device['default_samplerate'])
    frames=rate//10
    with sd.InputStream(samplerate=rate,channels=1,dtype='float32',blocksize=frames) as stream:
        yield stream,rate,frames

@contextmanager
def loopback_stream(mic):
    with mic.recorder(samplerate=48000,blocksize=4800) as stream:
        yield stream,48000,4800

def capture(channel, mic):
    try:
        with (microphone_stream() if channel=='INPUT' else loopback_stream(mic)) as (rec,rate,frames):
            divisor=gcd(rate,16000)
            chunks=[]; previous=-1; start=0
            while not stop.is_set():
                with lock: g=generation; skip=paused.is_set()
                t=time.monotonic()-origin
                if channel=='INPUT':
                    data,overflow=rec.read(frames)
                    if overflow:
                        chunks=[]
                        emit(status='Input overflow: audio block dropped')
                        continue
                else: data=rec.record(numframes=frames)
                with lock: valid=g==generation and not paused.is_set() and not stop.is_set()
                if previous!=g or skip or not valid: chunks=[]; previous=g
                if skip or not valid: continue
                if not chunks: start=t
                chunks.append(np.asarray(data,dtype=np.float32))
                if len(chunks)==50:
                    samples=np.concatenate(chunks); chunks=[]
                    mono=samples.mean(axis=1) if samples.ndim==2 else samples
                    try: audio.put_nowait((g,channel,start,resample_poly(mono,16000//divisor,rate//divisor).astype(np.float32)))
                    except queue.Full: emit(status='Recognition behind: audio block dropped')
    except Exception as e: emit(error=channel+': '+str(e)); stop.set()
def main():
    global origin
    threading.Thread(target=commands,daemon=True).start()
    emit(status='Loading local model')
    model=WhisperModel('base.en',device='cpu',compute_type='int8')
    devices=[]
    if 'input' in sys.argv: devices.append(('INPUT',None))
    if 'output' in sys.argv:
        speaker=sc.default_speaker(); devices.append(('OUTPUT',sc.get_microphone(id=speaker.id,include_loopback=True)))
    if stop.is_set(): return
    origin=time.monotonic(); emit(status='Capturing')
    workers=[threading.Thread(target=capture,args=d,daemon=True) for d in devices]
    for w in workers: w.start()
    while not stop.is_set():
        try: g,ch,start,pcm=audio.get(timeout=.2)
        except queue.Empty: continue
        with lock:
            if g!=generation or paused.is_set(): continue
        segments,_=model.transcribe(pcm,language='en',beam_size=1,vad_filter=True,condition_on_previous_text=False)
        block=[{'channel':ch,'start_ms':int((start+s.start)*1000),'end_ms':int((start+s.end)*1000),'text':s.text.strip()} for s in segments if s.text.strip()]
        with lock:
            if g==generation and not paused.is_set() and not stop.is_set():
                for item in shield.push(ch,block): emit(segment=item)
if __name__=='__main__':
    if '--shield-test' in sys.argv: self_test(); sys.exit(0)
    try: main()
    except Exception: emit(error='Echo worker failed; pending text discarded')
    finally: stop.set(); emit(status='Stopped')