use std::{
    fs::{self, Metadata},
    io,
    path::PathBuf, time::UNIX_EPOCH,
};

pub struct DocumentEntry {
    pub id: usize,
    pub path: PathBuf,
    pub metadata: DocMetaData,
    pub token_count: usize,
}

#[derive(Debug, Clone)]
pub struct DocMetaData {
    pub create_time: u64,
    pub modified_time: u64,
    pub permissions: u32,
    pub is_dir: bool,
}

impl DocumentEntry {
    pub fn new(id: usize, path: PathBuf, token_count: usize) -> io::Result<Self> {
        Ok(DocumentEntry {
            id,
            path: path.clone(),
            metadata: DocMetaData::new(fs::metadata(path)?),
            token_count,
        })
    }
}

impl DocMetaData {
    pub fn new(meta_data: Metadata) -> Self {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;

            let created = meta_data.created().ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let modified = meta_data.modified().ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            
            DocMetaData {
                create_time: created,
                modified_time: modified,
                permissions: meta_data.mode(),
                is_dir: meta_data.is_dir(),
            }
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;

            const EPOCH_DIFF_SECS: i64 = 11_644_473_600;

            let created = (meta_data.creation_time() / 10_000_000) - EPOCH_DIFF_SECS;
            let modified = (meta_data.modified_time() / 10_000_000) - EPOCH_DIFF_SECS;

            DocMetaData {
                create_time: created,
                modified_time: modified,
                permissions: 0,
            }
        }
    }
}

