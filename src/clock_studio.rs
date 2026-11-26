use dioxus::prelude::*;
use crate::{clock::Clock, model::PieceContent};
#[component]
pub fn ClockStudio(title: String, initial: Option<Clock>, on_change: EventHandler<PieceContent>) -> Element {
    let mut clock=use_signal(move || initial.unwrap_or_else(Clock::new));
    let mut status=use_signal(String::new);
    let mut pending=use_signal(|| None::<Clock>);
    use_effect(move || on_change.call(PieceContent::Clock(clock.read().clone())));
    let snapshot=clock.read().clone();
    let total=snapshot.total();
    let mut elapsed=0u64;
    rsx! {
        section { style: "padding:24px;color:#d5e9df;",
            p { style:"color:#8299ec;letter-spacing:3px;font-size:12px;", "TEMPORAL COMPOSITION" }
            h1 { style:"color:#8299ec;font-size:32px;font-weight:bold;", "Clock Studio" }
            p { "Arrange time. Understand duration." }
            div { style:"display:flex;gap:12px;margin:24px 0;flex-wrap:wrap;",
                button { class:"nb-orbit-return", onclick:move |_| {
                    match crate::manifest::import_dialog() {
                        Ok(Some(doc)) => match Clock::from_panel(doc.piece) {
                            Ok(next) => { if clock.peek().slots.is_empty() { clock.set(next); status.set("Manifest imported".into()); } else { pending.set(Some(next)); } },
                            Err(e) => status.set(e),
                        },
                        Ok(None) => status.set("Import canceled".into()),
                        Err(e) => status.set(format!("Import failed: {e}")),
                    }
                }, "Import Manifest" }
                button { class:"nb-orbit-return", onclick:move |_| clock.write().add(), "+ Add time slot" }
            }
            if let Some(source)=snapshot.source_title.as_ref() { p { style:"color:#79c9a1;", "SOURCE / {source}" } }
            p { style:"color:#e5a65c;", "{status}" }
            for (i,slot) in snapshot.slots.iter().enumerate() {
                { let id=slot.id; elapsed+=u64::from(slot.minutes); let end=elapsed; let hours=slot.minutes/60; let mins=slot.minutes%60; let percent=if total==0 {0.0} else {100.0*f64::from(slot.minutes)/total as f64}; rsx! {
                    div { key:"{id}", style:"border:1px solid #304356;border-radius:12px;padding:16px;margin:12px 0;background:#111923;",
                        div { style:"display:flex;gap:8px;align-items:center;",
                            span { style:"color:#8299ec;", "{i+1}" }
                            input { style:"flex:1;min-width:0;background:#18232e;padding:8px;color:#d5e9df;", value:"{slot.text}", oninput:move |e| { if let Some(s)=clock.write().slots.iter_mut().find(|s|s.id==id){s.text=e.value();} } }
                            button { class:"nb-orbit-return", disabled:i==0, onclick:move |_| { let mut c=clock.write(); if let Some(j)=c.slots.iter().position(|s|s.id==id){if j>0 {c.slots.swap(j,j-1);}} }, "Up" }
                            button { class:"nb-orbit-return", disabled:i+1==snapshot.slots.len(), onclick:move |_| { let mut c=clock.write(); if let Some(j)=c.slots.iter().position(|s|s.id==id){if j+1<c.slots.len(){c.slots.swap(j,j+1);}} }, "Down" }
                            button { class:"nb-orbit-return", onclick:move |_| clock.write().slots.retain(|s|s.id!=id), "Remove" }
                        }
                        div { style:"display:flex;gap:8px;align-items:center;justify-content:flex-end;margin:12px 0;",
                            span { "Duration" }
                            input { r#type:"number", min:"0", max:"999", style:"width:75px;background:#18232e;padding:8px;", value:"{hours}", oninput:move |e| { if let Ok(n)=e.value().parse::<u32>() { if let Some(s)=clock.write().slots.iter_mut().find(|s|s.id==id){s.minutes=n.min(999)*60+s.minutes%60;} } } }
                            span { "hr" }
                            input { r#type:"number", min:"0", max:"59", style:"width:75px;background:#18232e;padding:8px;", value:"{mins}", oninput:move |e| { if let Ok(n)=e.value().parse::<u32>() { if let Some(s)=clock.write().slots.iter_mut().find(|s|s.id==id){s.minutes=s.minutes/60*60+n.min(59);} } } }
                            span { "min" }
                        }
                        div { style:"height:7px;background:#202b3c;border-radius:8px;", div { style:"height:100%;width:{percent}%;background:#8299ec;border-radius:8px;" } }
                        p { style:"font-size:12px;color:#9aaec4;margin-top:8px;", "Elapsed: {end} min" }
                    }
                } }
            }
            p { style:"color:#8299ec;letter-spacing:2px;margin-top:24px;", "COMPOSED TIMELINE" }
            div { style:"display:flex;gap:2px;height:24px;background:#202b3c;border-radius:6px;overflow:hidden;",
                for (i,s) in snapshot.slots.iter().enumerate() { if s.minutes>0 { { let color=["#8299ec","#67cbd8","#a89ce8","#79c9a1"][i%4]; rsx! { div { key:"{s.id}", title:"{s.text}: {s.minutes} min", style:"flex:{s.minutes};background:{color};" } } } } }
            }
            p { "Total estimated duration" }
            h2 { style:"color:#8299ec;font-size:30px;", "{total/60}h {total%60}m" }
            if pending().is_some() {
                div { class:"nb-instrument-modal", div { class:"nb-instrument-dialog",
                    h2 { "Replace current timeline?" } p { "The current time slots will be replaced. The source file will not change." }
                    button { class:"nb-orbit-return", onclick:move |_| pending.set(None), "Cancel" }
                    button { class:"nb-orbit-return", onclick:move |_| { let next=pending.write().take(); if let Some(next)=next {clock.set(next);status.set("Manifest imported".into());} }, "Replace" }
                } }
            }
        }
    }
}