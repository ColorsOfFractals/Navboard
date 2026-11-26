use serde::{Serialize, Deserialize};
use uuid::Uuid;
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Field { pub id: Uuid, pub label: String, pub value: String }
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Specs { pub id: Uuid, pub template: String, pub fields: Vec<Field> }
pub const TEMPLATES: &[(&str,&str,&[&str])] = &[
    ("Infrastructure", "Server", &["Hostname", "Manufacturer / model", "Serial / asset tag", "Operating system", "CPU", "RAM", "Storage / RAID", "IP address", "Server role", "Backup solution", "Last verified"]),
    ("Infrastructure", "Workstation", &["Hostname", "Assigned user", "Asset tag", "Model", "Operating system", "CPU", "RAM", "Storage", "Warranty"]),
    ("Infrastructure", "Network", &["Device name", "Model", "Firmware", "Management IP", "Subnet", "Gateway", "VLANs", "Protocols", "Configuration backup"]),
    ("Infrastructure", "Storage", &["Device name", "Model", "Capacity", "RAID", "Volumes", "Shares", "Backup", "Health"]),
    ("Infrastructure", "Printer", &["Model", "Serial", "IP address", "Driver", "Queue", "Location", "Test result"]),
    ("Infrastructure", "Software", &["Application", "Version", "Platform", "Dependencies", "Requirements", "License", "Deployment method"]),
    ("IMACD Operations", "Install", &["Client", "Asset", "Location", "Prerequisites", "Deployment scope", "Acceptance criteria"]),
    ("IMACD Operations", "Move", &["Asset", "Origin", "Destination", "Connectivity", "Downtime", "Acceptance criteria"]),
    ("IMACD Operations", "Add", &["Existing system", "New component", "Compatibility", "Prerequisites", "Acceptance criteria"]),
    ("IMACD Operations", "Change", &["System", "Current state", "Desired state", "Impact", "Rollback plan", "Acceptance criteria"]),
    ("IMACD Operations", "Dispose", &["Asset", "Authorization", "Data retention", "Sanitization method", "Disposal method", "Evidence"]),
    ("General Projects", "Requirements", &["Objective", "Scope", "Requirements", "Constraints", "Acceptance criteria"]),
    ("General Projects", "Resources", &["Resource", "Owner", "Availability", "Quantity", "Dependencies"]),
    ("General Projects", "Custom Form", &["Property"]),
 ];
impl Specs {
    pub fn new() -> Self { Self { id: Uuid::new_v4(), template: String::new(), fields: vec![] } }
    pub fn select(&mut self, name: &str) { if let Some((_,_,fields))=TEMPLATES.iter().find(|(_,n,_)| *n==name) { self.template=name.into(); self.fields=fields.iter().map(|label| Field { id:Uuid::new_v4(), label:(*label).into(), value:String::new() }).collect(); } }
    pub fn add(&mut self) { self.fields.push(Field { id:Uuid::new_v4(), label:"Custom property".into(), value:String::new() }); }
}
#[cfg(test)] mod tests { use super::*; #[test] fn templates_roundtrip() { assert_eq!(TEMPLATES.len(),14); for (_,name,_) in TEMPLATES { let mut s=Specs::new(); s.select(name); assert!(!s.fields.is_empty()); assert!(s.fields.iter().all(|f|f.value.is_empty())); s.fields[0].value="example".into(); assert!(s==serde_json::from_str::<Specs>(&serde_json::to_string(&s).unwrap()).unwrap()); } } }