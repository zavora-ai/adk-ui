use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedSurface {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub payload: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum SurfaceStoreError {
    #[error("surface not found: {owner}/{id}")]
    NotFound { owner: String, id: String },
    #[error("surface version conflict: expected {expected}, found {actual}")]
    VersionConflict { expected: u64, actual: u64 },
    #[error("invalid surface identifier: {0}")]
    InvalidId(String),
    #[error("surface store IO error: {0}")]
    Io(String),
    #[error("surface store JSON error: {0}")]
    Json(String),
}

#[async_trait]
pub trait SurfaceStore: Send + Sync {
    async fn save(
        &self,
        owner: &str,
        id: &str,
        name: &str,
        payload: Value,
        expected_version: Option<u64>,
    ) -> Result<SavedSurface, SurfaceStoreError>;

    async fn load(&self, owner: &str, id: &str) -> Result<SavedSurface, SurfaceStoreError>;
    async fn list(&self, owner: &str) -> Result<Vec<SavedSurface>, SurfaceStoreError>;
    async fn delete(
        &self,
        owner: &str,
        id: &str,
        expected_version: Option<u64>,
    ) -> Result<bool, SurfaceStoreError>;
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn validate_segment(value: &str) -> Result<(), SurfaceStoreError> {
    if value.trim().is_empty() || value.len() > 256 || value.contains('\0') {
        return Err(SurfaceStoreError::InvalidId(value.to_string()));
    }
    Ok(())
}

fn build_saved(
    previous: Option<&SavedSurface>,
    owner: &str,
    id: &str,
    name: &str,
    payload: Value,
) -> SavedSurface {
    let timestamp = now();
    SavedSurface {
        id: id.to_string(),
        name: name.to_string(),
        owner: owner.to_string(),
        version: previous.map_or(1, |surface| surface.version + 1),
        created_at: previous
            .map_or_else(|| timestamp.clone(), |surface| surface.created_at.clone()),
        updated_at: timestamp,
        payload,
    }
}

fn check_version(
    previous: Option<&SavedSurface>,
    expected: Option<u64>,
) -> Result<(), SurfaceStoreError> {
    if let Some(expected) = expected {
        let actual = previous.map_or(0, |surface| surface.version);
        if actual != expected {
            return Err(SurfaceStoreError::VersionConflict { expected, actual });
        }
    }
    Ok(())
}

#[derive(Default)]
pub struct InMemorySurfaceStore {
    surfaces: RwLock<HashMap<(String, String), SavedSurface>>,
}

impl InMemorySurfaceStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SurfaceStore for InMemorySurfaceStore {
    async fn save(
        &self,
        owner: &str,
        id: &str,
        name: &str,
        payload: Value,
        expected_version: Option<u64>,
    ) -> Result<SavedSurface, SurfaceStoreError> {
        validate_segment(owner)?;
        validate_segment(id)?;
        let key = (owner.to_string(), id.to_string());
        let mut surfaces = self
            .surfaces
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        check_version(surfaces.get(&key), expected_version)?;
        let saved = build_saved(surfaces.get(&key), owner, id, name, payload);
        surfaces.insert(key, saved.clone());
        Ok(saved)
    }

    async fn load(&self, owner: &str, id: &str) -> Result<SavedSurface, SurfaceStoreError> {
        let surfaces = self
            .surfaces
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        surfaces
            .get(&(owner.to_string(), id.to_string()))
            .cloned()
            .ok_or_else(|| SurfaceStoreError::NotFound {
                owner: owner.to_string(),
                id: id.to_string(),
            })
    }

    async fn list(&self, owner: &str) -> Result<Vec<SavedSurface>, SurfaceStoreError> {
        let surfaces = self
            .surfaces
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut values = surfaces
            .values()
            .filter(|surface| surface.owner == owner)
            .cloned()
            .collect::<Vec<_>>();
        values.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(values)
    }

    async fn delete(
        &self,
        owner: &str,
        id: &str,
        expected_version: Option<u64>,
    ) -> Result<bool, SurfaceStoreError> {
        let key = (owner.to_string(), id.to_string());
        let mut surfaces = self
            .surfaces
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        check_version(surfaces.get(&key), expected_version)?;
        Ok(surfaces.remove(&key).is_some())
    }
}

pub struct FsSurfaceStore {
    root: PathBuf,
    lock: Mutex<()>,
}

impl FsSurfaceStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, SurfaceStoreError> {
        let root = root.into();
        std::fs::create_dir_all(&root).map_err(|error| SurfaceStoreError::Io(error.to_string()))?;
        Ok(Self {
            root,
            lock: Mutex::new(()),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn encode(value: &str) -> String {
        value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn owner_dir(&self, owner: &str) -> PathBuf {
        self.root.join(Self::encode(owner))
    }
    fn surface_path(&self, owner: &str, id: &str) -> PathBuf {
        self.owner_dir(owner)
            .join(format!("{}.json", Self::encode(id)))
    }

    fn read_path(path: &Path, owner: &str, id: &str) -> Result<SavedSurface, SurfaceStoreError> {
        let raw = std::fs::read_to_string(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                SurfaceStoreError::NotFound {
                    owner: owner.to_string(),
                    id: id.to_string(),
                }
            } else {
                SurfaceStoreError::Io(error.to_string())
            }
        })?;
        serde_json::from_str(&raw).map_err(|error| SurfaceStoreError::Json(error.to_string()))
    }
}

#[async_trait]
impl SurfaceStore for FsSurfaceStore {
    async fn save(
        &self,
        owner: &str,
        id: &str,
        name: &str,
        payload: Value,
        expected_version: Option<u64>,
    ) -> Result<SavedSurface, SurfaceStoreError> {
        validate_segment(owner)?;
        validate_segment(id)?;
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let path = self.surface_path(owner, id);
        let previous = if path.exists() {
            Some(Self::read_path(&path, owner, id)?)
        } else {
            None
        };
        check_version(previous.as_ref(), expected_version)?;
        let saved = build_saved(previous.as_ref(), owner, id, name, payload);
        let owner_dir = self.owner_dir(owner);
        std::fs::create_dir_all(&owner_dir)
            .map_err(|error| SurfaceStoreError::Io(error.to_string()))?;
        let temp = owner_dir.join(format!(".{}.{}.tmp", Self::encode(id), std::process::id()));
        let bytes = serde_json::to_vec_pretty(&saved)
            .map_err(|error| SurfaceStoreError::Json(error.to_string()))?;
        std::fs::write(&temp, bytes).map_err(|error| SurfaceStoreError::Io(error.to_string()))?;
        std::fs::rename(&temp, &path).map_err(|error| SurfaceStoreError::Io(error.to_string()))?;
        Ok(saved)
    }

    async fn load(&self, owner: &str, id: &str) -> Result<SavedSurface, SurfaceStoreError> {
        validate_segment(owner)?;
        validate_segment(id)?;
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::read_path(&self.surface_path(owner, id), owner, id)
    }

    async fn list(&self, owner: &str) -> Result<Vec<SavedSurface>, SurfaceStoreError> {
        validate_segment(owner)?;
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let directory = self.owner_dir(owner);
        if !directory.exists() {
            return Ok(vec![]);
        }
        let mut surfaces: Vec<SavedSurface> = Vec::new();
        for entry in std::fs::read_dir(directory)
            .map_err(|error| SurfaceStoreError::Io(error.to_string()))?
        {
            let path = entry
                .map_err(|error| SurfaceStoreError::Io(error.to_string()))?
                .path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read_to_string(path)
                .map_err(|error| SurfaceStoreError::Io(error.to_string()))?;
            surfaces.push(
                serde_json::from_str(&raw)
                    .map_err(|error| SurfaceStoreError::Json(error.to_string()))?,
            );
        }
        surfaces.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(surfaces)
    }

    async fn delete(
        &self,
        owner: &str,
        id: &str,
        expected_version: Option<u64>,
    ) -> Result<bool, SurfaceStoreError> {
        validate_segment(owner)?;
        validate_segment(id)?;
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let path = self.surface_path(owner, id);
        if !path.exists() {
            check_version(None, expected_version)?;
            return Ok(false);
        }
        let current = Self::read_path(&path, owner, id)?;
        check_version(Some(&current), expected_version)?;
        std::fs::remove_file(path).map_err(|error| SurfaceStoreError::Io(error.to_string()))?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn exercise_store(store: &dyn SurfaceStore) {
        let first = store
            .save("agent", "main", "Main", json!({"value": 1}), Some(0))
            .await
            .unwrap();
        assert_eq!(first.version, 1);
        let second = store
            .save("agent", "main", "Main", json!({"value": 2}), Some(1))
            .await
            .unwrap();
        assert_eq!(second.version, 2);
        assert_eq!(second.created_at, first.created_at);
        assert!(matches!(
            store
                .save("agent", "main", "Main", json!({}), Some(1))
                .await,
            Err(SurfaceStoreError::VersionConflict { actual: 2, .. })
        ));
        assert_eq!(store.list("agent").await.unwrap().len(), 1);
        assert!(store.list("other").await.unwrap().is_empty());
        assert!(store.delete("agent", "main", Some(2)).await.unwrap());
    }

    #[tokio::test]
    async fn in_memory_store_versions_and_scopes_surfaces() {
        exercise_store(&InMemorySurfaceStore::new()).await;
    }

    #[tokio::test]
    async fn filesystem_store_versions_and_scopes_surfaces() {
        let directory = tempfile::tempdir().unwrap();
        let store = FsSurfaceStore::new(directory.path()).unwrap();
        exercise_store(&store).await;
    }
}
