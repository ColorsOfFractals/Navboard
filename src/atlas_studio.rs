use dioxus::prelude::*;
use uuid::Uuid;
use crate::atlas::*;
#[component]
pub fn AtlasStudio(title:String,initial:Option<Atlas>,on_change:EventHandler<crate::model::PieceContent>)->Element{
 let mut map=use_signal(move ||initial.unwrap_or_default());
 let mut library=use_signal(load);
 let mut pending=use_signal(||None::<Equipment>);
 let mut selected=use_signal(||None::<Uuid>);
 let mut from=use_signal(||None::<Uuid>);
 let mut drag=use_signal(||None::<(Uuid,f64,f64,f64,f64)>);
 let mut name=use_signal(String::new);
 let mut color=use_signal(||"#8db8a8".to_string());
 let mut icon=use_signal(||"\u{25C7}".to_string());
 let mut kind=use_signal(||"Ethernet".to_string());
 let mut status=use_signal(||"Choose equipment, then click the map or drag it onto the map.".to_string());
 use_effect(move ||on_change.call(crate::model::PieceContent::Atlas(map.read().clone())));
 let snapshot=map.read().clone();let mut palette=defaults();if let Ok(v)=&*library.read(){palette.extend(v.clone());}
 rsx!{ div { class:"atlas-studio", onmouseup:move |_|drag.set(None),
 h2 { "Atlas / {title}" } p { "{status}" }
 div { class:"atlas-layout",
 div { class:"atlas-palette", h3 { "EQUIPMENT" }
 for e in palette { {let item=e.clone();let item2=e.clone();rsx!{button { style:"color:{e.color}", onmousedown:move |ev|{ev.prevent_default();pending.set(Some(item.clone()));}, onclick:move |_|pending.set(Some(item2.clone())), "{e.icon}  {e.name}" }}} }
 h3 { "CUSTOM TYPE" }
 input { placeholder:"Equipment type",value:"{name}",oninput:move |e|name.set(e.value()) }
 input { r#type:"color",value:"{color}",oninput:move |e|color.set(e.value()) }
 select { value:"{icon}",onchange:move |e|icon.set(e.value()), for e in defaults(){option {value:"{e.icon}","{e.icon} {e.name}"}} }
 button { disabled:name().trim().is_empty() || library.read().is_err(),onclick:move |_|{match save_type(Equipment{name:name().trim().into(),icon:icon(),color:color()}){Ok(v)=>{library.set(Ok(v));name.set(String::new());status.set("Reusable type saved".into());},Err(e)=>status.set(e)}},"Save equipment type" }
 if let Err(e)=&*library.read(){p {"Library locked: {e}"}}
 }
 div { class:"atlas-main",
 div { class:"atlas-tools", select {value:"{kind}",onchange:move |e|kind.set(e.value()),for k in ["Ethernet","Fiber","Wireless","VPN","Power"]{option {value:k,"{k}"}}} button {onclick:move |_|{pending.set(None);from.set(None);},"Cancel placement / link"} span {"{snapshot.nodes.len()} devices / {snapshot.links.len()} links"} }
 div { class:"atlas-viewport", div {class:"atlas-board",
 onmousemove:move |e|{if let Some((id,mx,my,x,y))=drag(){let p=e.client_coordinates();if let Some(n)=map.write().nodes.iter_mut().find(|n|n.id==id){n.x=(x+p.x-mx).clamp(0.,1060.);n.y=(y+p.y-my).clamp(0.,700.);}}},
 onmouseleave:move |_|drag.set(None),
 onmouseup:move |e|{drag.set(None);let item=pending.write().take();if let Some(item)=item{let p=e.element_coordinates();let id=map.write().add(item,p.x-65.,p.y-40.);selected.set(Some(id));}},
 svg {class:"atlas-links",width:"1200",height:"800",for l in &snapshot.links {if let (Some(a),Some(b))=(snapshot.nodes.iter().find(|n|n.id==l.a),snapshot.nodes.iter().find(|n|n.id==l.b)){ {let id=l.id;let x1=a.x+65.;let y1=a.y+40.;let x2=b.x+65.;let y2=b.y+40.;let c=match l.kind.as_str(){"Fiber"=>"#e5a65c","Wireless"=>"#8299ec","VPN"=>"#d9a6d8","Power"=>"#ef8181",_=>"#8db8a8"};rsx!{line {x1:"{x1}",y1:"{y1}",x2:"{x2}",y2:"{y2}",stroke:c,stroke_width:"3",class:"atlas-link",onmouseup:move |e|e.stop_propagation(),oncontextmenu:move |e|{e.prevent_default();map.write().links.retain(|l|l.id!=id);}}}}}}}
 for n in &snapshot.nodes { {let id=n.id;let x=n.x;let y=n.y;rsx!{div {key:"{id}",class:if selected()==Some(id){"atlas-node selected"}else{"atlas-node"},style:"left:{x}px;top:{y}px;color:{n.equipment.color}",onmouseup:move |e|{e.stop_propagation();drag.set(None);},onmousedown:move |e|{e.prevent_default();selected.set(Some(id));let p=e.client_coordinates();drag.set(Some((id,p.x,p.y,x,y)));},span {class:"atlas-icon","{n.equipment.icon}"} strong {"{n.label}"} button {class:"atlas-jack",onmousedown:move |e|e.stop_propagation(),onclick:move |_|{if let Some(a)=from(){map.write().connect(a,id,kind());from.set(None);}else{from.set(Some(id));status.set("Choose another device link jack".into());}},if from()==Some(id){"LINKING"}else{"LINK"}} }}} }
 }}
 if let Some(n)=snapshot.nodes.iter().find(|n|Some(n.id)==selected()){ {let id=n.id;rsx!{div {class:"atlas-inspector",h3 {"DEVICE DETAILS"} input {placeholder:"Device label",value:"{n.label}",oninput:move |e|{if let Some(n)=map.write().nodes.iter_mut().find(|n|n.id==id){n.label=e.value();}}} input {placeholder:"IP / hostname",value:"{n.address}",oninput:move |e|{if let Some(n)=map.write().nodes.iter_mut().find(|n|n.id==id){n.address=e.value();}}} textarea {placeholder:"Notes / location / verification",value:"{n.notes}",oninput:move |e|{if let Some(n)=map.write().nodes.iter_mut().find(|n|n.id==id){n.notes=e.value();}}} button {onclick:move |_|{map.write().remove(id);selected.set(None);from.set(None);},"Remove device"}}}} }
 p {"Drag devices to move. Link two jacks. Right-click a link to remove it."}
 }
 }
 }}
}
