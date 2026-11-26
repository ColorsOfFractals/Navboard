use serde::{Serialize,Deserialize};
use uuid::Uuid;
use std::{path::Path,io::Read};
const MAX_IMAGE: usize=4*1024*1024;
const MAX_REEL: usize=16*1024*1024;
#[derive(Clone,PartialEq,Serialize,Deserialize)]
pub struct Frame { pub id:Uuid, pub title:String, pub notes:String, pub mime:String, pub data:String }
#[derive(Clone,PartialEq,Serialize,Deserialize)]
pub struct Lens { pub id:Uuid, pub frames:Vec<Frame> }
fn encode(bytes:&[u8])->String {
 let alphabet=b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
 let mut out=String::with_capacity((bytes.len()+2)/3*4);
 for c in bytes.chunks(3) { let n=((c[0] as usize)<<16)|((c.get(1).copied().unwrap_or(0) as usize)<<8)|(c.get(2).copied().unwrap_or(0) as usize); out.push(alphabet[(n>>18)&63] as char);out.push(alphabet[(n>>12)&63] as char);out.push(if c.len()>1 {alphabet[(n>>6)&63] as char}else{'='});out.push(if c.len()>2 {alphabet[n&63] as char}else{'='}); } out
}
impl Lens {
 pub fn new()->Self {Self{id:Uuid::new_v4(),frames:vec![]}}
 pub fn add(&mut self,path:&Path)->Result<(),String>{
  if self.frames.len()>=100{return Err("100 frame limit".into());}
  let mut bytes=Vec::new(); std::fs::File::open(path).map_err(|e|e.to_string())?.take((MAX_IMAGE+1) as u64).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
  if bytes.len()>MAX_IMAGE{return Err("Image exceeds 4 MiB".into());}
  let mime=if bytes.starts_with(b"\x89PNG\r\n\x1a\n"){"image/png"}else if bytes.starts_with(&[255,216,255]){"image/jpeg"}else{return Err("Not a PNG or JPEG signature".into());};
  let data=encode(&bytes); if self.frames.iter().map(|f|f.data.len()).sum::<usize>()+data.len()>MAX_REEL{return Err("Reel exceeds 16 MiB encoded data".into());}
  self.frames.push(Frame{id:Uuid::new_v4(),title:path.file_stem().unwrap_or_default().to_string_lossy().into_owned(),notes:String::new(),mime:mime.into(),data}); Ok(())
 }
}
#[cfg(target_os="windows")]
pub fn pick()->Result<Vec<String>,String>{
 use std::os::windows::process::CommandExt;
 let script=r#"$ErrorActionPreference='Stop'; Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.OpenFileDialog; $d.Filter='Images (*.png;*.jpg;*.jpeg)|*.png;*.jpg;*.jpeg'; $d.Multiselect=$true; $d.CheckFileExists=$true; if($d.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){[Console]::OutputEncoding=New-Object System.Text.UTF8Encoding($false); [Console]::Write((ConvertTo-Json -InputObject @($d.FileNames) -Compress))}; $d.Dispose()"#;
 let r=std::process::Command::new("powershell.exe").args(["-NoProfile","-STA","-NonInteractive","-Command",script]).creation_flags(0x08000000).output().map_err(|e|e.to_string())?;
 if !r.status.success(){return Err(String::from_utf8_lossy(&r.stderr).into_owned());} if r.stdout.is_empty(){return Ok(vec![]);} serde_json::from_slice(&r.stdout).map_err(|e|e.to_string())
}
#[cfg(not(target_os="windows"))] pub fn pick()->Result<Vec<String>,String>{Err("Windows dialog only".into())}
#[cfg(test)] mod tests{use super::*;#[test] fn encoding(){assert_eq!(encode(b"f"),"Zg==");assert_eq!(encode(b"fo"),"Zm8=");assert_eq!(encode(b"foo"),"Zm9v");}#[test] fn roundtrip(){let mut l=Lens::new();l.frames.push(Frame{id:Uuid::new_v4(),title:"Photo".into(),notes:"Evidence".into(),mime:"image/png".into(),data:"AAAA".into()});assert!(l==serde_json::from_str::<Lens>(&serde_json::to_string(&l).unwrap()).unwrap());}}

#[cfg(target_os="windows")]
pub fn export_gallery(title:&str,reel:&Lens)->Result<String,String>{
 use std::{io::Write,process::{Command,Stdio},os::windows::process::CommandExt};
 let script=r#"$ErrorActionPreference='Stop'; Add-Type -AssemblyName System.Windows.Forms; Add-Type -AssemblyName System.IO.Compression; Add-Type -AssemblyName System.IO.Compression.FileSystem;
 [Console]::InputEncoding=New-Object System.Text.UTF8Encoding($false); [Console]::OutputEncoding=New-Object System.Text.UTF8Encoding($false);
 $r=ConvertFrom-Json -InputObject ([Console]::In.ReadToEnd());
 function Safe([string]$s){$s=($s -replace '[<>:"/\\|?*\x00-\x1f]','_').Trim(' ','.'); if(!$s){$s='Gallery'}; if($s.Length -gt 80){$s=$s.Substring(0,80)}; return $s}
 $name=Safe $r.title; $d=New-Object System.Windows.Forms.SaveFileDialog; $d.Filter='ZIP Gallery (*.zip)|*.zip'; $d.FileName=$name+'.zip'; $d.AddExtension=$true; $d.DefaultExt='zip'; if($d.ShowDialog() -ne 'OK'){$d.Dispose();[Console]::Write('Export cancelled');exit}; $dest=$d.FileName;$d.Dispose(); if(Test-Path -LiteralPath $dest){throw 'Destination already exists; choose a new filename.'};
 $tmp=$dest+'.'+[guid]::NewGuid().ToString()+'.tmp'; $zip=$null; $stream=$null;
 try {$stream=[IO.File]::Open($tmp,[IO.FileMode]::CreateNew);$zip=New-Object IO.Compression.ZipArchive($stream,[IO.Compression.ZipArchiveMode]::Create);$notes=New-Object System.Text.StringBuilder; $i=0;foreach($f in $r.frames){$i++;$ext=if($f.mime -eq 'image/png'){'.png'}elseif($f.mime -eq 'image/jpeg'){'.jpg'}else{throw 'Unsupported image type'};$file=('{0:D3} - {1}{2}' -f $i,(Safe $f.title),$ext);$entry=$zip.CreateEntry($name+'/'+$file);$es=$entry.Open();try{$bytes=[Convert]::FromBase64String($f.data);$es.Write($bytes,0,$bytes.Length)}finally{$es.Dispose()};[void]$notes.AppendLine($file);[void]$notes.AppendLine([string]$f.notes);[void]$notes.AppendLine()};$entry=$zip.CreateEntry($name+'/Gallery Notes.txt');$es=$entry.Open();try{$bytes=[Text.Encoding]::UTF8.GetBytes($notes.ToString());$es.Write($bytes,0,$bytes.Length)}finally{$es.Dispose()};$zip.Dispose();$zip=$null;$stream.Dispose();$stream=$null;[IO.File]::Move($tmp,$dest);[Console]::Write('Gallery exported: '+$dest)}finally{if($zip){$zip.Dispose()};if($stream){$stream.Dispose()};if(Test-Path -LiteralPath $tmp){Remove-Item -LiteralPath $tmp -Force}}
 "#;
 let payload=serde_json::to_vec(&serde_json::json!({"title":title,"frames":&reel.frames})).map_err(|e|e.to_string())?;
 let mut child=Command::new("powershell.exe").args(["-NoProfile","-STA","-NonInteractive","-Command",script]).creation_flags(0x08000000).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e|e.to_string())?;
 child.stdin.take().ok_or("Missing export input")?.write_all(&payload).map_err(|e|e.to_string())?;
 let result=child.wait_with_output().map_err(|e|e.to_string())?;
 if result.status.success(){Ok(String::from_utf8_lossy(&result.stdout).into_owned())}else{Err(String::from_utf8_lossy(&result.stderr).into_owned())}
}
#[cfg(not(target_os="windows"))]
pub fn export_gallery(_: &str,_:&Lens)->Result<String,String>{Err("Windows export only".into())}
