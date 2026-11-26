use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::model::{Panel, PieceContent};
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Slot { pub id: Uuid, pub source_step: Option<Uuid>, pub text: String, pub minutes: u32 }
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Clock { pub id: Uuid, pub source_piece: Option<Uuid>, pub source_title: Option<String>, pub slots: Vec<Slot> }
impl Clock {
    pub fn new() -> Self { Self { id: Uuid::new_v4(), source_piece: None, source_title: None, slots: vec![] } }
    pub fn from_panel(panel: Panel) -> Result<Self,String> {
        match panel.content {
            Some(PieceContent::Stitch(stitch)) => Ok(Self { id: Uuid::new_v4(), source_piece: Some(panel.id), source_title: Some(panel.title), slots: stitch.steps.into_iter().map(|s| Slot { id: Uuid::new_v4(), source_step: Some(s.id), text: s.text, minutes: 0 }).collect() }),
            Some(PieceContent::Clock(clock)) => Ok(clock),
            _ => Err("Choose a Stitch or Clock manifest".into()),
        }
    }
    pub fn total(&self) -> u64 { self.slots.iter().map(|s| u64::from(s.minutes)).sum() }
    pub fn add(&mut self) { self.slots.push(Slot { id: Uuid::new_v4(), source_step: None, text: "New time slot".into(), minutes: 0 }); }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn import_preserves_order_and_identity() {
        let mut s=crate::stitch::Stitch::new("Test"); s.add("One"); s.add("Two"); let id=s.steps[0].id;
        let mut c=Clock::from_panel(Panel { id: Uuid::new_v4(), title:"Test".into(), notes:String::new(), kind:crate::model::PanelKind::Stitch, content:Some(PieceContent::Stitch(s)) }).unwrap();
        assert_eq!(c.slots[0].source_step,Some(id)); assert_eq!(c.slots[1].text,"Two"); assert_eq!(c.total(),0); c.slots[0].minutes=75; assert_eq!(c.total(),75);
        let json=serde_json::to_string(&c).unwrap(); assert!(c==serde_json::from_str::<Clock>(&json).unwrap());
    }
}