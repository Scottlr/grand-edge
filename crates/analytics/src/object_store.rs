use std::fs;
use std::path::{Path, PathBuf};

use crate::AnalyticsError;

pub trait ObjectStore {
    fn put(&self, relative_path: &Path, bytes: &[u8]) -> Result<(), AnalyticsError>;
    fn root(&self) -> &Path;
}

#[derive(Debug, Clone)]
pub struct LocalFileObjectStore {
    root: PathBuf,
}

impl LocalFileObjectStore {
    pub fn new(root: PathBuf) -> Result<Self, AnalyticsError> {
        fs::create_dir_all(&root).map_err(|_| AnalyticsError::CreateDirectory(root.clone()))?;
        Ok(Self { root })
    }
}

impl ObjectStore for LocalFileObjectStore {
    fn put(&self, relative_path: &Path, bytes: &[u8]) -> Result<(), AnalyticsError> {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| AnalyticsError::CreateDirectory(parent.to_path_buf()))?;
        }
        fs::write(path, bytes)?;
        Ok(())
    }

    fn root(&self) -> &Path {
        &self.root
    }
}
