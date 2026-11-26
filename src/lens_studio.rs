use dioxus::prelude::*;
use crate::{lens::{Lens,pick},model::PieceContent};
#[component]
pub fn LensStudio(title:String,initial:Option<Lens>,on_change:EventHandler<PieceContent>)->Element{
 let mut reel=use_signal(move || initial.unwrap_or_else(Lens::new));
 let mut selected=use_signal(||0usize); let mut rotation=use_signal(||0i64); let mut status=use_signal(String::new);
 use_effect(move || on_change.call(PieceContent::Lens(reel.read().clone())));
 let snapshot=reel.read().clone(); let count=snapshot.frames.len(); let index=selected().min(count.saturating_sub(1));
 let angle=rotation()*360/7;
 let mut projection=use_signal(||false);
 let counter=format!("Frame {} / {}",if count==0 {0}else{index+1},count);
 let export_title=title.clone();
 rsx!{section{style:"padding:24px;color:#d5e9df;",
 p{style:"color:#d9a6d8;letter-spacing:3px;font-size:12px;","VISUAL CAPTURE"}
 h1{style:"color:#d9a6d8;font-size:32px;font-weight:bold;","Lens Studio"}
 p{"Capture perspectives. Preserve evidence."}
 button{class:"nb-orbit-return",style:"margin:20px 0;",onclick:move |_|{match pick(){Ok(paths)=>{let mut added=0;let mut errors=Vec::new();for path in paths{match reel.write().add(std::path::Path::new(&path)){Ok(())=>added+=1,Err(e)=>errors.push(e)}}if added>0||!errors.is_empty(){status.set(format!("Added {} image(s). {}",added,errors.join("; ")));}},Err(e)=>status.set(e)}},"+ Import Images"}
 button{class:"nb-orbit-return",disabled:count==0,onclick:move |_|{let data=reel.read().clone();match crate::lens::export_gallery(&export_title,&data){Ok(msg)=>status.set(msg),Err(e)=>status.set(e)}},"Export Gallery (.zip)"}
 p{style:"color:#d9a6d8;","{status}"}
 div{style:"display:flex;flex-wrap:wrap;align-items:center;justify-content:center;gap:24px;margin:20px 0;",
 div{style:"position:relative;width:260px;height:260px;flex-shrink:0;",
 div{style:"position:absolute;inset:0;border-radius:50%;background:radial-gradient(circle,#222a3b,#101720);border:2px solid #465365;box-shadow:inset 0 0 0 7px #18232e,0 0 28px #d9a6d815;transform:rotate({angle}deg);transition:transform 260ms ease;",
 for i in 0..7usize { {let a=i as f64*360.0/7.0; let slot_style=format!("position:absolute;left:calc(50% - 24px);top:calc(50% - 17px);width:48px;height:34px;transform:rotate({a}deg) translateY(-88px);background:#202d3c;border:1px solid #465365;border-radius:6px;overflow:hidden;"); let offset=(i+(rotation().rem_euclid(7) as usize))%7; let target=index+offset; let target=if offset<count {target%count.max(1)}else{count}; rsx!{button{style:"{slot_style}",title:"Select frame",onclick:move |_|{if target<count{selected.set(target);rotation.set(rotation()-offset as i64);}},if let Some(f)=snapshot.frames.get(target){img{style:"width:100%;height:100%;object-fit:cover;",src:"data:{f.mime};base64,{f.data}"}}}} } }
 }
 div{style:"position:absolute;left:35%;top:35%;width:30%;height:30%;border-radius:50%;background:#0d141c;border:1px solid #465365;display:grid;place-items:center;color:#d9a6d8;font-weight:bold;pointer-events:none;","LENS"}
 div{style:"position:absolute;left:calc(50% - 28px);top:20px;width:56px;height:43px;border:2px solid #d9a6d8;border-radius:7px;pointer-events:none;"}
 }
 div{style:"background:linear-gradient(145deg,#19232f,#111923);border:1px solid #354353;border-radius:14px;padding:20px;min-width:220px;text-align:center;",
 p{style:"color:#d9a6d8;letter-spacing:3px;","NAVBOARD / LENS"}
 div{class:"lens-optic"}
 h2{style:"font-size:24px;color:#e8edf5;","PROJECTOR"}
 button{class:"nb-orbit-return",disabled:count==0,onclick:move |_|{projection.set(true);},"Engage Projector / Fullscreen"}
 p{"{counter}"}
 button{class:"nb-orbit-return",disabled:count==0,onclick:move |_|{selected.set((index+count-1)%count);rotation.set(rotation()+1);},"Previous"}
 button{style:"margin:12px 0 0 8px;background:#30243e;border:1px solid #d9a6d8;color:#d9a6d8;padding:12px;border-radius:8px;font-weight:bold;",disabled:count==0,onclick:move |_|{selected.set((index+1)%count);rotation.set(rotation()-1);},"Next Frame"}
 }
 }
 if let Some(f)=snapshot.frames.get(index){ {let id=f.id;rsx!{div{style:"border:1px solid #72576e;border-radius:12px;padding:14px;background:#111923;",
 img{style:"display:block;width:100%;max-height:500px;object-fit:contain;background:#080d13;border-radius:8px;",src:"data:{f.mime};base64,{f.data}"}
 input{style:"width:100%;margin-top:12px;background:#18232e;padding:10px;color:#d9a6d8;",value:"{f.title}",oninput:move |e|{if let Some(f)=reel.write().frames.iter_mut().find(|f|f.id==id){f.title=e.value();}}}
 textarea{style:"width:100%;min-height:80px;background:#18232e;padding:10px;margin:8px 0;",placeholder:"Frame observations...",value:"{f.notes}",oninput:move |e|{if let Some(f)=reel.write().frames.iter_mut().find(|f|f.id==id){f.notes=e.value();}}}
 button{class:"nb-orbit-return",disabled:index==0,onclick:move |_|{if index>0{reel.write().frames.swap(index,index-1);selected.set(index-1);}},"Move earlier"}
 button{class:"nb-orbit-return",disabled:index+1>=count,onclick:move |_|{if index+1<count{reel.write().frames.swap(index,index+1);selected.set(index+1);}},"Move later"}
 button{class:"nb-orbit-return",onclick:move |_|{reel.write().frames.retain(|f|f.id!=id);selected.set(index.min(count.saturating_sub(2)));},"Remove frame"}
 }}} }else{p{style:"text-align:center;color:#9aaec4;","Load PNG or JPEG images to begin your reel."}}
 if projection() && count>0 {
 div{class:"lens-projection",tabindex:"0",onmounted:move |e|{async move {let _=e.set_focus(true).await;}},onkeydown:move |e|{match e.key(){Key::Escape=>projection.set(false),Key::ArrowRight=>selected.set((index+1)%count),Key::ArrowLeft=>selected.set((index+count-1)%count),_=>{}}},
 div{class:"lens-projection-ignition",div{class:"lens-magazine-animation",for i in 0..7 { {let a=i*360/7;let st=format!("transform:rotate({a}deg) translateY(-60px);");rsx!{div{class:"lens-animation-slot",style:"{st}",if let Some(f)=snapshot.frames.get(i){img{src:"data:{f.mime};base64,{f.data}"}}}}} }},div{class:"lens-projector-body",div{class:"lens-optic"}},div{class:"lens-beam"}}
 div{class:"lens-slide",if let Some(f)=snapshot.frames.get(index){img{src:"data:{f.mime};base64,{f.data}",alt:"{f.title}"} p{"{f.title}"}}
 div{class:"lens-slide-controls",button{onclick:move |_|selected.set((index+count-1)%count),"Previous"} span{"{counter}"} button{onclick:move |_|selected.set((index+1)%count),"Next"} button{onclick:move |_|projection.set(false),"Eject / Esc"}}
 }
 }
 }
 p{style:"color:#9aaec4;font-size:12px;margin-top:12px;","4 MiB per image / 16 MiB encoded per reel / 100 frames maximum"}
 }}
}