use dioxus::prelude::*;
use crate::{echo::{Echo,SharedWorker},model::PieceContent};
#[component]
pub fn EchoStudio(initial:Option<Echo>,on_change:EventHandler<PieceContent>)->Element{
 let mut data=use_signal(move||initial.unwrap_or_default());
 let mut input=use_signal(||true);let mut output=use_signal(||true);let mut running=use_signal(||false);let mut paused=use_signal(||false);let mut status=use_signal(||String::from("Standby"));let mut view=use_signal(||String::from("Merged"));
 let worker=use_hook(SharedWorker::default);let cleanup=worker.clone();use_drop(move||{if let Ok(mut w)=cleanup.lock(){w.kill();}});
 let poll=worker.clone();use_future(move||{let poll=poll.clone();async move{loop{futures_timer::Delay::new(std::time::Duration::from_millis(150)).await;if !running(){continue;}let(events,done)=poll.lock().unwrap().drain();for e in events{if let Some(s)=e.segment{if !paused(){data.write().segments.push(s);}}if let Some(s)=e.status{status.set(s);}if let Some(s)=e.error{status.set(format!("Error: {s}"));}}if done{running.set(false);paused.set(false);on_change.call(PieceContent::Echo(data.read().clone()));}}}});
 let start=worker.clone();let pause=worker.clone();let stop=worker.clone();let snapshot=data.read().clone();let merged=snapshot.text(&view());
 rsx!{section{class:"echo-studio",
 p{class:"echo-eyebrow","DUAL CHANNEL / LOCAL RECOGNITION"} h1{"Echo Studio"} p{"Privacy Shield prototype active for new sessions. Test audio only. Review before sharing."}
 div{class:"echo-transport",
 button{disabled:running()||(!input()&&!output()),onclick:move |_|{match start.lock().unwrap().start(input(),output()){Ok(())=>{data.set(Echo::default());running.set(true);paused.set(false);status.set("Loading model".into());},Err(e)=>status.set(e)}} ,"Start new session"}
 button{disabled:!running(),onclick:move |_|{let next=!paused();pause.lock().unwrap().send(if next{"pause"}else{"resume"});paused.set(next);status.set(if next{"Paused".into()}else{"Resuming".into()});},if paused(){"Resume"}else{"Pause"}}
 button{disabled:!running(),onclick:move |_|{stop.lock().unwrap().kill();running.set(false);paused.set(false);status.set("Stopped; pending audio discarded".into());on_change.call(PieceContent::Echo(data.read().clone()));},"Stop"}
 span{"{status}"}}
 div{class:"echo-channels",
 div{class:"echo-channel echo-input",h2{"INPUT / Microphone"}button{disabled:running(),onclick:move |_|input.set(!input()),if input(){"ARMED"}else{"Arm"}}textarea{readonly:true,value:snapshot.text("INPUT"),rows:"12"}}
 div{class:"echo-channel echo-output",h2{"OUTPUT / System audio"}button{disabled:running(),onclick:move |_|output.set(!output()),if output(){"ARMED"}else{"Arm"}}textarea{readonly:true,value:snapshot.text("OUTPUT"),rows:"12"}}}
 div{class:"echo-transport",h2{"Transcript mixer"}for name in ["Merged","INPUT","OUTPUT"]{button{onclick:move |_|view.set(name.into()),"{name}"}}}
 textarea{class:"echo-mixer",readonly:true,value:"{merged}",rows:"10"}
 button{class:"nb-orbit-return",onclick:move |_|{let text=data.read().text(&view());match std::process::Command::new("clip.exe").stdin(std::process::Stdio::piped()).spawn(){Ok(mut c)=>{use std::io::Write;let result=c.stdin.take().map(|mut s|{let bytes:Vec<u8>=text.encode_utf16().flat_map(|v|v.to_le_bytes()).collect();s.write_all(&bytes)});let ok=matches!(result,Some(Ok(())))&&c.wait().map(|s|s.success()).unwrap_or(false);status.set(if ok{"Copied - review before sharing".into()}else{"Clipboard failed".into()});},Err(e)=>status.set(e.to_string())}},"Copy Transcript"}
 p{"Stop before exporting or assigning. Starting a new session replaces the displayed transcript."}
 }}
}
