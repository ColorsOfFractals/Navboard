use serde::{Serialize,Deserialize};
use uuid::Uuid;
use std::{fs,io::Write,path::PathBuf,collections::HashSet};
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct Equipment { pub name:String, pub icon:String, pub color:String }
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct Node { pub id:Uuid,pub equipment:Equipment,pub label:String,pub address:String,pub notes:String,pub x:f64,pub y:f64 }
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct Link { pub id:Uuid,pub a:Uuid,pub b:Uuid,pub kind:String }
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,Default)]
pub struct Atlas { pub nodes:Vec<Node>,pub links:Vec<Link> }
impl Atlas {
 pub fn add(&mut self,e:Equipment,x:f64,y:f64)->Uuid { let id=Uuid::new_v4(); self.nodes.push(Node{id,label:e.name.clone(),equipment:e,address:String::new(),notes:String::new(),x:x.clamp(0.,1060.),y:y.clamp(0.,700.)}); id }
 pub fn connect(&mut self,a:Uuid,b:Uuid,kind:String) { if a!=b && self.nodes.iter().any(|n|n.id==a) && self.nodes.iter().any(|n|n.id==b) && !self.links.iter().any(|l|l.kind==kind && ((l.a==a&&l.b==b)||(l.a==b&&l.b==a))) { self.links.push(Link{id:Uuid::new_v4(),a,b,kind}); } }
 pub fn remove(&mut self,id:Uuid){self.nodes.retain(|n|n.id!=id);self.links.retain(|l|l.a!=id&&l.b!=id);}
 pub fn validate(&self)->Result<(),String>{let mut ids=HashSet::new(); for n in &self.nodes {if n.id.is_nil()||!ids.insert(n.id)||!n.x.is_finite()||!n.y.is_finite(){return Err("Invalid Atlas node".into());}} let mut links=HashSet::new(); for l in &self.links {if l.id.is_nil()||!links.insert(l.id)||l.a==l.b||!ids.contains(&l.a)||!ids.contains(&l.b){return Err("Invalid Atlas link".into());}} Ok(())}
}
pub fn defaults()->Vec<Equipment>{[("Internet","\u{2601}"),("Firewall","\u{25A6}"),("Router","\u{21C4}"),("Switch","\u{25A4}"),("Access point","\u{25CE}"),("Server","\u{25A5}"),("Workstation","\u{25A3}"),("Laptop","\u{25B1}"),("Printer","\u{2399}"),("NAS","\u{25A4}"),("UPS","\u{26A1}"),("Custom","\u{25C7}")].iter().map(|(n,i)|Equipment{name:(*n).into(),icon:(*i).into(),color:"#8db8a8".into()}).collect()}
fn path()->Result<PathBuf,String>{Ok(PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA missing")?).join("Navboard").join("atlas-equipment.json"))}
pub fn load()->Result<Vec<Equipment>,String>{match fs::read(path()?){Ok(b)=>serde_json::from_slice(&b).map_err(|e|e.to_string()),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(vec![]),Err(e)=>Err(e.to_string())}}
pub fn save_type(e:Equipment)->Result<Vec<Equipment>,String>{let mut all=load()?;if all.iter().chain(defaults().iter()).any(|x|x.name.eq_ignore_ascii_case(&e.name)){return Err("Type already exists".into());} all.push(e);let p=path()?;let dir=p.parent().ok_or("Missing parent")?;fs::create_dir_all(dir).map_err(|e|e.to_string())?;let mut tmp=tempfile::NamedTempFile::new_in(dir).map_err(|e|e.to_string())?;let b=serde_json::to_vec_pretty(&all).map_err(|e|e.to_string())?;tmp.write_all(&b).and_then(|_|tmp.as_file().sync_all()).map_err(|e|e.to_string())?;tmp.persist(p).map_err(|e|e.to_string())?;Ok(all)}
#[cfg(test)] mod tests {use super::*;#[test] fn topology(){let mut m=Atlas::default();let e=defaults()[0].clone();let a=m.add(e.clone(),0.,0.);let b=m.add(e,100.,100.);m.connect(a,b,"Ethernet".into());m.connect(b,a,"Ethernet".into());assert_eq!(m.links.len(),1);assert!(m.validate().is_ok());let copy:Atlas=serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();assert_eq!(m,copy);m.remove(a);assert!(m.links.is_empty());}}
