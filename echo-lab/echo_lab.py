import tkinter as tk
from tkinter import ttk, messagebox
import threading, queue, time
from datetime import datetime, timedelta
import numpy as np
import soundcard as sc
from faster_whisper import WhisperModel

root = tk.Tk()
root.title('Navboard | Echo Ignition Lab')
root.geometry('850x600')
root.configure(bg='#101720')
style = ttk.Style(); style.theme_use('clam')
style.configure('.', background='#18232e', foreground='#e8edf5', fieldbackground='#18232e')
style.configure('TButton', padding=8)
status = tk.StringVar(value='Standby - model downloads on first start')
events = queue.Queue(); audio = queue.Queue(maxsize=8)
stop = threading.Event(); paused = threading.Event(); lock = threading.Lock()
state = {'generation':0, 'running':False, 'closing':False}
model = None

def capture(origin, wall, mic):
    try:
        with mic.recorder(samplerate=48000, blocksize=4800) as rec:
            chunks=[]; start=0; generation=-1
            while not stop.is_set():
                t=time.monotonic()-origin
                with lock: g=state['generation']; skip=paused.is_set()
                data=rec.record(numframes=4800)
                with lock: valid=(g==state['generation'] and not paused.is_set() and not stop.is_set())
                if skip or not valid or generation!=g:
                    chunks=[]; generation=g
                    if skip or not valid: continue
                if not chunks: start=t
                chunks.append(np.asarray(data,dtype=np.float32))
                if len(chunks)>=50:
                    samples=np.concatenate(chunks,axis=0); chunks=[]
                    mono=samples.mean(axis=1) if samples.ndim==2 else samples
                    # 48 kHz to 16 kHz, anti-aliasing before decimation.
                    from scipy.signal import resample_poly
                    mono=resample_poly(mono,1,3).astype(np.float32)
                    try: audio.put_nowait((g,start,wall,mono))
                    except queue.Full: events.put(('status','Recognition behind: audio block dropped'))
            # Deliberately discard partial block on stop/pause.
    except Exception as e:
        events.put(('error',str(e))); stop.set()

def recognize():
    while not stop.is_set() or not audio.empty():
        try: g,start,wall,pcm=audio.get(timeout=.2)
        except queue.Empty: continue
        with lock: valid=g==state['generation']
        if not valid: continue
        try:
            segments,_=model.transcribe(pcm,language='en',beam_size=1,vad_filter=True,condition_on_previous_text=False)
            for s in segments:
                if s.text.strip():
                    stamp=(wall+timedelta(seconds=start+s.start)).strftime('%H:%M:%S')
                    events.put(('line',(g,f'[OUTPUT] {stamp} {s.text.strip()}\n')))
        except Exception as e: events.put(('error',str(e))); stop.set()

def session():
    global model
    try:
        if model is None: model=WhisperModel('base.en',device='cpu',compute_type='int8')
        if stop.is_set(): return
        speaker=sc.default_speaker()
        mic=sc.get_microphone(id=speaker.id,include_loopback=True)
        origin=time.monotonic(); wall=datetime.now()
        events.put(('status','TRANSCRIBING - '+speaker.name))
        worker=threading.Thread(target=recognize,daemon=True); worker.start()
        capture(origin,wall,mic); stop.set(); worker.join()
    except Exception as e: events.put(('error',str(e)))
    finally: events.put(('done',None))

def start():
    if state['running']: return
    stop.clear(); paused.clear(); state['running']=True
    with lock: state['generation']+=1
    status.set('Loading local model / opening default output...')
    start_btn.config(state='disabled'); pause_btn.config(state='normal'); stop_btn.config(state='normal')
    threading.Thread(target=session,daemon=True).start()

def pause():
    if not state['running']: return
    with lock:
        state['generation']+=1
        if paused.is_set(): paused.clear(); label='RESUMED'
        else: paused.set(); label='PAUSED'
        while True:
            try: audio.get_nowait()
            except queue.Empty: break
    transcript.insert('end',f'[{label}] {datetime.now():%H:%M:%S}\n')
    pause_btn.config(text='Resume' if paused.is_set() else 'Pause'); status.set(label)

def finish():
    stop.set(); status.set('Stopping / finalizing complete blocks...'); pause_btn.config(state='disabled'); stop_btn.config(state='disabled')

def poll():
    while True:
        try: kind,value=events.get_nowait()
        except queue.Empty: break
        if kind=='line':
            g,line=value
            with lock: valid=g==state['generation']
            if valid: transcript.insert('end',line); transcript.see('end')
        elif kind=='status':
            if not paused.is_set(): status.set(value)
        elif kind=='error': status.set('ERROR: '+value); messagebox.showerror('Echo',value)
        elif kind=='done':
            state['running']=False; start_btn.config(state='normal'); pause_btn.config(state='disabled',text='Pause'); stop_btn.config(state='disabled'); status.set('Standby - session ended')
    if not state['closing']: root.after(100,poll)

def copy():
    root.clipboard_clear(); root.clipboard_append(transcript.get('1.0','end-1c')); status.set('Copied - review before sharing')

def close():
    state['closing']=True; stop.set(); root.destroy()

tk.Label(root,text='ECHO / IGNITION LAB',font=('Segoe UI',22,'bold'),fg='#ef8181',bg='#101720').pack(anchor='w',padx=20,pady=(20,5))
tk.Label(root,text='OUTPUT ONLY  |  Local recognition  |  Test audio only',fg='#79c9a1',bg='#101720').pack(anchor='w',padx=20)
bar=ttk.Frame(root); bar.pack(fill='x',padx=20,pady=15)
start_btn=ttk.Button(bar,text='Start',command=start); start_btn.pack(side='left')
pause_btn=ttk.Button(bar,text='Pause',command=pause,state='disabled'); pause_btn.pack(side='left')
stop_btn=ttk.Button(bar,text='Stop',command=finish,state='disabled'); stop_btn.pack(side='left')
ttk.Button(bar,text='Copy Transcript',command=copy).pack(side='right')
ttk.Button(bar,text='Clear',command=lambda: transcript.delete('1.0','end')).pack(side='right')
transcript=tk.Text(root,bg='#090f16',fg='#e8edf5',insertbackground='white',font=('Consolas',12),wrap='word',undo=False,padx=12,pady=12); transcript.pack(fill='both',expand=True,padx=20)
tk.Label(root,textvariable=status,fg='#ef8181',bg='#101720',wraplength=790).pack(anchor='w',padx=20,pady=12)
root.protocol('WM_DELETE_WINDOW',close); root.after(100,poll); root.mainloop()
