#![allow(dead_code)]

use crate::model::Project;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

const SCHEMA_VERSION: u32 = 1;
const PRIMARY_FILE: &str = "worlds.json";
const BACKUP_FILE: &str = "worlds.json.bak";

#[derive(Serialize, Deserialize)]
struct VaultDocument {
    schema_version: u32,
    projects: Vec<Project>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadOrigin {
    Empty,
    Primary,
    Backup,
}

pub struct LoadedWorlds {
    pub projects: Vec<Project>,
    pub origin: LoadOrigin,
}

#[derive(Debug)]
pub enum VaultError {
    Io(io::Error),
    Json(serde_json::Error),
    UnsupportedSchema(u32),
    InvalidData(String),
    RecoveryFailed(String),
    MissingLocalAppData,
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Storage I/O error: {error}"),
            Self::Json(error) => write!(f, "Invalid JSON: {error}"),
            Self::UnsupportedSchema(version) => {
                write!(f, "Unsupported schema version: {version}")
            }
            Self::InvalidData(message) => write!(f, "Invalid data: {message}"),
            Self::RecoveryFailed(message) => {
                write!(f, "Recovery failed: {message}")
            }
            Self::MissingLocalAppData => {
                write!(f, "LOCALAPPDATA is unavailable")
            }
        }
    }
}

impl std::error::Error for VaultError {}

impl From<io::Error> for VaultError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for VaultError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub struct Vault {
    root: PathBuf,
}

impl Vault {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn local() -> Result<Self, VaultError> {
        let local = std::env::var_os("LOCALAPPDATA")
            .ok_or(VaultError::MissingLocalAppData)?;

        Ok(Self::new(PathBuf::from(local).join("Navboard")))
    }

    pub fn primary_path(&self) -> PathBuf {
        self.root.join(PRIMARY_FILE)
    }

    pub fn backup_path(&self) -> PathBuf {
        self.root.join(BACKUP_FILE)
    }

    pub fn load(&self) -> Result<LoadedWorlds, VaultError> {
        let primary = self.primary_path();
        let backup = self.backup_path();

        match read_optional(&primary) {
            Ok(Some(document)) => {
                return Ok(LoadedWorlds {
                    projects: document.projects,
                    origin: LoadOrigin::Primary,
                });
            }
            Err(error @ VaultError::UnsupportedSchema(_)) => {
                return Err(error);
            }
            Err(error @ VaultError::Io(_)) => {
                return Err(error);
            }
            primary_result => {
                match read_optional(&backup) {
                    Ok(Some(document)) => Ok(LoadedWorlds {
                        projects: document.projects,
                        origin: LoadOrigin::Backup,
                    }),
                    Ok(None) => {
                        match primary_result {
                            Ok(None) => Ok(LoadedWorlds {
                                projects: Vec::new(),
                                origin: LoadOrigin::Empty,
                            }),
                            Err(error) => Err(error),
                            Ok(Some(_)) => unreachable!(),
                        }
                    }
                    Err(error @ VaultError::UnsupportedSchema(_)) => {
                        Err(error)
                    }
                    Err(error) => Err(VaultError::RecoveryFailed(
                        format!("No usable primary or backup: {error}")
                    )),
                }
            }
        }
    }

    pub fn save(&self, projects: &[Project]) -> Result<(), VaultError> {
        validate_projects(projects)?;

        let document = VaultDocument {
            schema_version: SCHEMA_VERSION,
            projects: projects.to_vec(),
        };

        let bytes = serde_json::to_vec_pretty(&document)?;

        // Refuse writes when existing storage cannot be safely loaded.
        // This also protects unsupported future schema versions.
        self.load()?;

        fs::create_dir_all(&self.root)?;

        let primary = self.primary_path();
        let backup = self.backup_path();

        // Preserve only a validated primary as the recovery snapshot.
        match fs::read(&primary) {
            Ok(previous_bytes) => {
                match decode(&previous_bytes) {
                    Ok(_) => atomic_write(&backup, &previous_bytes)?,
                    Err(error @ VaultError::UnsupportedSchema(_)) => {
                        return Err(error);
                    }
                    Err(_) => {
                        // Recovery backup is already valid.
                        // Never replace it with corrupted primary data.
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        atomic_write(&primary, &bytes)?;

        Ok(())
    }
}

fn validate_projects(projects: &[Project]) -> Result<(), VaultError> {
    let mut identities = HashSet::new();

    for project in projects {
        if project.id.is_nil() || !identities.insert(project.id) {
            return Err(VaultError::InvalidData(
                format!("Invalid or duplicate project ID: {}", project.id)
            ));
        }

        for panel in &project.panels {
            if panel.id.is_nil() || !identities.insert(panel.id) {
                return Err(VaultError::InvalidData(
                    format!("Invalid or duplicate panel ID: {}", panel.id)
                ));
            }
        }
    }

    Ok(())
}

fn decode(bytes: &[u8]) -> Result<VaultDocument, VaultError> {
    // Inspect the schema before attempting to decode project objects.
    #[derive(Deserialize)]
    struct SchemaHeader {
        schema_version: u32,
    }

    let header: SchemaHeader = serde_json::from_slice(bytes)?;

    if header.schema_version != SCHEMA_VERSION {
        return Err(VaultError::UnsupportedSchema(
            header.schema_version
        ));
    }

    let document: VaultDocument = serde_json::from_slice(bytes)?;

    validate_projects(&document.projects)?;

    Ok(document)
}

fn read_optional(path: &Path) -> Result<Option<VaultDocument>, VaultError> {
    match fs::read(path) {
        Ok(bytes) => decode(&bytes).map(Some),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), VaultError> {
    let parent = path.parent().ok_or_else(|| {
        VaultError::InvalidData("Storage path has no parent".into())
    })?;

    // Same-directory temporary file keeps replacement on one filesystem.
    let mut temporary = NamedTempFile::new_in(parent)?;

    temporary.write_all(bytes)?;
    temporary.flush()?;
    temporary.as_file().sync_all()?;

    temporary.persist(path)
        .map_err(|error| VaultError::Io(error.error))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Panel, PanelKind};
    use uuid::Uuid;

    fn sample_worlds() -> Vec<Project> {
        let kinds = [
            PanelKind::Text,
            PanelKind::Canvas,
            PanelKind::Stitch,
            PanelKind::Forge,
        ];

        vec![Project {
            id: Uuid::new_v4(),
            name: "Test World".into(),
            panels: kinds.into_iter().map(|kind| Panel {
                id: Uuid::new_v4(),
                title: format!("{} Panel", kind.label()),
                notes: "Persistent content".into(),
                kind,
                content: None,
            }).collect(),
        }]
    }

    #[test]
    fn empty_vault_loads_cleanly() {
        let directory = tempfile::tempdir().unwrap();
        let vault = Vault::new(directory.path().to_path_buf());

        let loaded = vault.load().unwrap();

        assert!(loaded.projects.is_empty());
        assert_eq!(loaded.origin, LoadOrigin::Empty);
    }

    #[test]
    fn roundtrip_preserves_all_objects_and_ids() {
        let directory = tempfile::tempdir().unwrap();
        let vault = Vault::new(directory.path().to_path_buf());
        let original = sample_worlds();

        vault.save(&original).unwrap();

        let loaded = vault.load().unwrap();

        assert_eq!(loaded.origin, LoadOrigin::Primary);
        assert!(loaded.projects == original);
        assert_eq!(loaded.projects[0].panels.len(), 4);
    }

    #[test]
    fn second_save_preserves_previous_snapshot() {
        let directory = tempfile::tempdir().unwrap();
        let vault = Vault::new(directory.path().to_path_buf());
        let original = sample_worlds();

        vault.save(&original).unwrap();

        let mut updated = original.clone();
        updated[0].name = "Updated World".into();

        vault.save(&updated).unwrap();

        assert!(vault.load().unwrap().projects == updated);

        let backup = fs::read(vault.backup_path()).unwrap();
        assert!(decode(&backup).unwrap().projects == original);
    }

    #[test]
    fn corrupted_primary_recovers_from_backup() {
        let directory = tempfile::tempdir().unwrap();
        let vault = Vault::new(directory.path().to_path_buf());
        let original = sample_worlds();

        vault.save(&original).unwrap();
        vault.save(&original).unwrap();

        fs::write(vault.primary_path(), b"corrupted").unwrap();

        let recovered = vault.load().unwrap();

        assert_eq!(recovered.origin, LoadOrigin::Backup);
        assert!(recovered.projects == original);
    }

    #[test]
    fn recovery_save_preserves_valid_backup() {
        let directory = tempfile::tempdir().unwrap();
        let vault = Vault::new(directory.path().to_path_buf());
        let original = sample_worlds();

        vault.save(&original).unwrap();
        vault.save(&original).unwrap();

        let backup_before = fs::read(vault.backup_path()).unwrap();

        fs::write(vault.primary_path(), b"corrupted").unwrap();

        let recovered = vault.load().unwrap();
        vault.save(&recovered.projects).unwrap();

        assert_eq!(
            vault.load().unwrap().origin,
            LoadOrigin::Primary
        );

        assert_eq!(
            fs::read(vault.backup_path()).unwrap(),
            backup_before
        );
    }

    #[test]
    fn corrupted_storage_is_not_treated_as_empty() {
        let directory = tempfile::tempdir().unwrap();
        let vault = Vault::new(directory.path().to_path_buf());

        fs::write(vault.primary_path(), b"corrupted").unwrap();

        assert!(vault.load().is_err());
        assert!(vault.save(&[]).is_err());

        assert_eq!(
            fs::read(vault.primary_path()).unwrap(),
            b"corrupted"
        );
    }

    #[test]
    fn unsupported_schema_blocks_load_and_save() {
        let directory = tempfile::tempdir().unwrap();
        let vault = Vault::new(directory.path().to_path_buf());

        let future = br#"{"schema_version":999,"projects":[]}"#;

        fs::write(vault.primary_path(), future).unwrap();

        assert!(matches!(
            vault.load(),
            Err(VaultError::UnsupportedSchema(999))
        ));

        assert!(vault.save(&[]).is_err());

        assert_eq!(
            fs::read(vault.primary_path()).unwrap(),
            future
        );
    }

    #[test]
    fn duplicate_identities_are_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let vault = Vault::new(directory.path().to_path_buf());
        let mut worlds = sample_worlds();

        let duplicate = worlds[0].panels[0].id;
        worlds[0].panels[1].id = duplicate;

        assert!(matches!(
            vault.save(&worlds),
            Err(VaultError::InvalidData(_))
        ));

        assert!(!vault.primary_path().exists());
    }
}