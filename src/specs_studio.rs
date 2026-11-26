use dioxus::prelude::*;
use crate::{specs::{Specs,TEMPLATES},model::PieceContent};
#[component]
pub fn SpecsStudio(title: String, initial: Option<Specs>, on_change: EventHandler<PieceContent>) -> Element {
    let mut form=use_signal(move || initial.unwrap_or_else(Specs::new));
    let mut pending=use_signal(|| None::<String>);
    use_effect(move || on_change.call(PieceContent::Specs(form.read().clone())));
    let snapshot=form.read().clone();
    let field_count=snapshot.fields.len();
    let count=snapshot.fields.iter().filter(|f| !f.value.trim().is_empty()).count();
    rsx! { section { style:"padding:24px;color:#d5e9df;",
        p { style:"color:#d5c57b;letter-spacing:3px;font-size:12px;", "STRUCTURED COMPOSITION" }
        h1 { style:"color:#d5c57b;font-size:32px;font-weight:bold;", "Specs Studio" }
        p { "Define properties. Capture requirements." }
        for category in ["Infrastructure","IMACD Operations","General Projects"] {
            h3 { style:"color:#d5c57b;margin:20px 0 8px;", "{category}" }
            div { style:"display:grid;grid-template-columns:repeat(auto-fit,minmax(130px,1fr));gap:8px;",
                for (_,name,_) in TEMPLATES.iter().filter(|(c,_,_)| *c==category) {
                    { let name=*name; let active=snapshot.template==name; rsx! { button { key:"{name}", class:"nb-orbit-return", style:if active {"border:1px solid #d5c57b;color:#d5c57b;"} else {""}, onclick:move |_| { if form.peek().template!=name { if form.peek().fields.is_empty() {form.write().select(name);} else {pending.set(Some(name.into()));} } }, "{name}" } } }
                }
            }
        }
        if !snapshot.template.is_empty() {
            h2 { style:"color:#d5c57b;font-size:24px;margin-top:24px;", "{snapshot.template} Specifications" }
            p { style:"color:#9aaec4;font-size:13px;", "Enter known values only. Leave unknown or unverified details blank." }
            div { style:"display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:12px;margin:16px 0;",
                for field in snapshot.fields.iter() {
                    { let id=field.id; rsx! { div { key:"{id}", style:"border:1px solid #304356;border-radius:10px;padding:12px;background:#111923;",
                        div { style:"display:flex;gap:8px;align-items:center;",
                            input { style:"flex:1;min-width:0;color:#d5c57b;background:transparent;", value:"{field.label}", oninput:move |e| { if let Some(f)=form.write().fields.iter_mut().find(|f|f.id==id){f.label=e.value();} } }
                            button { class:"nb-orbit-return", title:"Remove field", onclick:move |_| form.write().fields.retain(|f|f.id!=id), "Remove" }
                        }
                        textarea { style:"width:100%;min-height:64px;background:#18232e;padding:8px;margin-top:8px;resize:vertical;", placeholder:"Enter value", value:"{field.value}", oninput:move |e| {if let Some(f)=form.write().fields.iter_mut().find(|f|f.id==id){f.value=e.value();}} }
                    } } }
                }
            }
            button { class:"nb-orbit-return", onclick:move |_| form.write().add(), "+ Add custom property" }
            p { style:"color:#d5c57b;margin-top:16px;", "{count} / {field_count} properties populated" }
        }
        if pending().is_some() { div { class:"nb-instrument-modal", div { class:"nb-instrument-dialog", h2 { "Replace current form?" } p { "Changing templates replaces the current fields and values. Export or assign the current form first if you want to keep it." } button { class:"nb-orbit-return", onclick:move |_| pending.set(None), "Cancel" } button { class:"nb-orbit-return", onclick:move |_| {let next=pending.write().take(); if let Some(name)=next {form.write().select(&name);}}, "Replace" } } } }
    } }
}