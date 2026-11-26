use serde::{Deserialize, Serialize};
use crate::model::{Panel, PanelKind, PieceContent};
use std::{io::Write, path::{Path, PathBuf}};

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub version: u32,
    pub piece: Panel,
}
impl Manifest {
    pub fn new(piece: Panel) -> Self {
        Self { format: "navboard".into(), version: 1, piece }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "navboard" || self.version != 1 { return Err("Unsupported manifest format or version".into()); }
        if self.piece.id.is_nil() || self.piece.title.trim().is_empty() { return Err("Invalid piece identity or title".into()); }
        match (&self.piece.kind, &self.piece.content) {
            (PanelKind::Atlas, Some(PieceContent::Atlas(map))) => map.validate(),
            (PanelKind::Text, None) | (PanelKind::Canvas, Some(PieceContent::Canvas(_))) |
            (PanelKind::Clock, Some(PieceContent::Clock(_))) | (PanelKind::Specs, Some(PieceContent::Specs(_))) | (PanelKind::Lens, Some(PieceContent::Lens(_))) | (PanelKind::Echo, Some(PieceContent::Echo(_))) | (PanelKind::Stitch, Some(PieceContent::Stitch(_))) | (PanelKind::Forge, Some(PieceContent::Forge(_))) => Ok(()),
            _ => Err("Instrument and payload do not match".into()),
        }
    }
    #[allow(dead_code)]
    pub fn read(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        if file.metadata().map_err(|e| e.to_string())?.len() > 64 * 1024 * 1024 { return Err("Manifest exceeds 64 MiB".into()); }
        let doc: Self = serde_json::from_reader(file).map_err(|e| e.to_string())?;
        doc.validate()?;
        Ok(doc)
    }
    pub fn write_new(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if bytes.len() > 64 * 1024 * 1024 { return Err("Manifest exceeds 64 MiB".into()); }
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| e.to_string())?;
        if let Err(e) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
            drop(file); let _ = std::fs::remove_file(path); return Err(e.to_string());
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub fn export_dialog(panel: Panel) -> Result<Option<PathBuf>, String> {
    use std::os::windows::process::CommandExt;
    let doc = Manifest::new(panel);
    doc.validate()?;
    let name: String = doc.piece.title.chars().map(|c| if c.is_control() || "<>:\"/\\|?*".contains(c) { '_' } else { c }).take(100).collect();
    let script = r#"$ErrorActionPreference='Stop'; Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.SaveFileDialog; $d.Filter='Navboard Manifest (*.nvbd)|*.nvbd'; $d.DefaultExt='nvbd'; $d.AddExtension=$true; $d.OverwritePrompt=$true; $d.FileName=$env:NB_EXPORT_NAME; if($d.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){ [Console]::OutputEncoding=New-Object System.Text.UTF8Encoding($false); [Console]::Write($d.FileName) }; $d.Dispose()"#;
    let result = std::process::Command::new("powershell.exe").args(["-NoProfile", "-STA", "-NonInteractive", "-Command", script]).env("NB_EXPORT_NAME", format!("{}.nvbd", name.trim().trim_end_matches('.'))).creation_flags(0x08000000).output().map_err(|e| e.to_string())?;
    if !result.status.success() { return Err(String::from_utf8_lossy(&result.stderr).into_owned()); }
    if result.stdout.is_empty() { return Ok(None); }
    let mut path = PathBuf::from(String::from_utf8(result.stdout).map_err(|e| e.to_string())?);
    if !path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("nvbd")).unwrap_or(false) { path.set_extension("nvbd"); }
    doc.write_new(&path)?;
    Ok(Some(path))
}
#[cfg(not(target_os = "windows"))]
pub fn export_dialog(_: Panel) -> Result<Option<PathBuf>, String> { Err("Windows dialog only".into()) }

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Manifest {
        let mut stitch = crate::stitch::Stitch::new("Procedure");
        stitch.add("First"); stitch.add("Second");
        Manifest::new(Panel { id: uuid::Uuid::new_v4(), title: "Procedure".into(), notes: String::new(), kind: PanelKind::Stitch, content: Some(PieceContent::Stitch(stitch)) })
    }
    #[test]
    fn roundtrip_and_no_overwrite() {
        let doc = sample();
        let path = std::env::temp_dir().join(format!("{}.nvbd", uuid::Uuid::new_v4()));
        doc.write_new(&path).unwrap();
        let restored = Manifest::read(&path).unwrap();
        assert!(doc.piece == restored.piece);
        assert!(doc.write_new(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn reject_version_and_mismatch() {
        let mut doc = sample(); doc.version = 2; assert!(doc.validate().is_err());
        doc.version = 1; doc.piece.kind = PanelKind::Forge; assert!(doc.validate().is_err());
    }
}
#[cfg(target_os = "windows")]
pub fn import_dialog() -> Result<Option<Manifest>, String> {
    use std::os::windows::process::CommandExt;
    let script = r#"$ErrorActionPreference='Stop'; Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.OpenFileDialog; $d.Filter='Navboard Manifest (*.nvbd)|*.nvbd'; $d.CheckFileExists=$true; if($d.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){ [Console]::OutputEncoding=New-Object System.Text.UTF8Encoding($false); [Console]::Write($d.FileName) }; $d.Dispose()"#;
    let result = std::process::Command::new("powershell.exe").args(["-NoProfile","-STA","-NonInteractive","-Command",script]).creation_flags(0x08000000).output().map_err(|e|e.to_string())?;
    if !result.status.success() { return Err(String::from_utf8_lossy(&result.stderr).into_owned()); }
    if result.stdout.is_empty() { return Ok(None); }
    let path = PathBuf::from(String::from_utf8(result.stdout).map_err(|e|e.to_string())?);
    Manifest::read(&path).map(Some)
}
#[cfg(not(target_os = "windows"))]
pub fn import_dialog() -> Result<Option<Manifest>, String> { Err("Windows dialog only".into()) }