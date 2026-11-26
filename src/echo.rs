use serde::{Serialize,Deserialize};
use std::{process::{Child,Command,Stdio},io::{BufRead,BufReader,Write},sync::{Arc,Mutex,mpsc},path::PathBuf};
#[derive(Clone,PartialEq,Serialize,Deserialize,Default)]
pub struct Echo { pub segments: Vec<Segment> }
#[derive(Clone,PartialEq,Serialize,Deserialize)]
pub struct Segment { pub channel:String,pub start_ms:u64,pub end_ms:u64,pub text:String }
impl Echo { pub fn text(&self, filter:&str)->String { let mut rows=self.segments.clone(); rows.sort_by_key(|s|s.start_ms); rows.iter().filter(|s|filter=="Merged"||s.channel==filter).map(|s|format!("[{}] {:02}:{:02}:{:02} {}",s.channel,s.start_ms/3600000,s.start_ms/60000%60,s.start_ms/1000%60,s.text)).collect::<Vec<_>>().join("\n") } }
#[derive(Deserialize)]
pub struct Event { pub segment:Option<Segment>,pub status:Option<String>,pub error:Option<String> }
#[derive(Default)]
pub struct Worker { child:Option<Child>, rx:Option<mpsc::Receiver<Event>> }
impl Worker {
 pub fn start(&mut self,input:bool,output:bool)->Result<(),String>{
 self.kill();
 let (root, python) = resolve_runtime()?;
 let mut cmd = Command::new(python);
 cmd.current_dir(&root);
 cmd.arg("-u").arg(root.join("echo_worker.py")).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
 #[cfg(windows)] {use std::os::windows::process::CommandExt;cmd.creation_flags(0x08000000);}
 if input{cmd.arg("input");} if output{cmd.arg("output");}
 let mut child=cmd.spawn().map_err(|e|e.to_string())?; let stdout=child.stdout.take().unwrap(); let(tx,rx)=mpsc::channel();
 std::thread::spawn(move||{for line in BufReader::new(stdout).lines().map_while(Result::ok){if let Ok(e)=serde_json::from_str::<Event>(&line){if tx.send(e).is_err(){break;}}} });
 self.child=Some(child);self.rx=Some(rx);Ok(()) }
 pub fn send(&mut self,cmd:&str){if let Some(s)=self.child.as_mut().and_then(|c|c.stdin.as_mut()){let _=writeln!(s,"{cmd}");let _=s.flush();}}
 pub fn drain(&mut self)->(Vec<Event>,bool){let events=self.rx.as_ref().map(|r|r.try_iter().collect()).unwrap_or_default();let done=self.child.as_mut().map(|c|matches!(c.try_wait(),Ok(Some(_)))).unwrap_or(true);(events,done)}
 pub fn kill(&mut self){if let Some(mut c)=self.child.take(){let _=c.kill();let _=c.wait();}self.rx=None;}
}
impl Drop for Worker {fn drop(&mut self){self.kill();}}
pub type SharedWorker=Arc<Mutex<Worker>>;
#[test] fn chronological(){let e=Echo{segments:vec![Segment{channel:"OUTPUT".into(),start_ms:2000,end_ms:3000,text:"second".into()},Segment{channel:"INPUT".into(),start_ms:1000,end_ms:2000,text:"first".into()}]};assert!(e.text("Merged").find("first")<e.text("Merged").find("second"));assert!(!e.text("INPUT").contains("second"));}


fn resolve_runtime() -> Result<(PathBuf, PathBuf), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Cannot locate Navboard: {e}"))?;

    let parent = exe.parent()
        .ok_or("Cannot locate Navboard directory")?;

    let root = parent.join("echo-lab");
    let python = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or("Cannot locate local application data")?
        .join("Navboard")
        .join("Echo")
        .join("venv")
        .join("Scripts")
        .join("python.exe");

    if python.is_file() && root.join("echo_worker.py").is_file() {
        return Ok((root, python));
    }

    #[cfg(debug_assertions)]
    {
        for directory in parent.ancestors() {
            if !directory.join("Cargo.toml").is_file() {
                continue;
            }

            let root = directory.join("echo-lab");
            let python = root.join(".venv/Scripts/python.exe");

            if python.is_file() && root.join("echo_worker.py").is_file() {
                return Ok((root, python));
            }
        }
    }

    Err("Echo setup required. Run Setup-Echo.ps1 in the installed echo-lab folder.".into())
}

