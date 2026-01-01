use pb::c2::ReportFileRequest;
use pb::eldritch::{File, FileMetadata};
use std::io::Read;
use std::path::Path;

/// Strategy 1: V2 Current Implementation
/// Reads entire file into memory with std::fs::read(), sends as single chunk
pub mod v2_current {
    use super::*;

    pub fn report_file(file_path: &Path) -> Result<Vec<ReportFileRequest>, String> {
        // Current implementation: read entire file at once
        let content = std::fs::read(file_path).map_err(|e| e.to_string())?;

        let req = ReportFileRequest {
            task_id: 1,
            chunk: Some(File {
                metadata: Some(FileMetadata {
                    path: file_path.to_string_lossy().to_string(),
                    ..Default::default()
                }),
                chunk: content,
            }),
        };

        Ok(vec![req])
    }
}

/// Strategy 2: V1 Legacy Implementation
/// Streams file in 1KB chunks (old approach from eldritch v1)
pub mod v1_legacy {
    use super::*;

    const CHUNK_SIZE: usize = 1024; // 1 KB

    pub fn report_file(file_path: &Path) -> Result<Vec<ReportFileRequest>, String> {
        let mut file = std::fs::File::open(file_path).map_err(|e| e.to_string())?;
        let mut requests = Vec::new();
        let mut buffer = [0u8; CHUNK_SIZE];
        let mut first_chunk = true;

        loop {
            let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }

            let metadata = if first_chunk {
                first_chunk = false;
                Some(FileMetadata {
                    path: file_path.to_string_lossy().to_string(),
                    ..Default::default()
                })
            } else {
                None
            };

            requests.push(ReportFileRequest {
                task_id: 1,
                chunk: Some(File {
                    metadata,
                    chunk: buffer[..n].to_vec(),
                }),
            });
        }

        Ok(requests)
    }
}

/// Strategy 3: Streaming 2MB
/// Modern streaming approach with 2MB chunks
pub mod streaming_2mb {
    use super::*;

    const CHUNK_SIZE: usize = 2 * 1024 * 1024; // 2 MB

    pub fn report_file(file_path: &Path) -> Result<Vec<ReportFileRequest>, String> {
        let mut file = std::fs::File::open(file_path).map_err(|e| e.to_string())?;
        let mut requests = Vec::new();
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut first_chunk = true;

        loop {
            let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }

            let metadata = if first_chunk {
                first_chunk = false;
                Some(FileMetadata {
                    path: file_path.to_string_lossy().to_string(),
                    ..Default::default()
                })
            } else {
                None
            };

            requests.push(ReportFileRequest {
                task_id: 1,
                chunk: Some(File {
                    metadata,
                    chunk: buffer[..n].to_vec(),
                }),
            });
        }

        Ok(requests)
    }
}

/// Strategy 4: Adaptive Chunking
/// Chooses chunk size based on file size:
/// - Small files (<256KB): 1KB chunks
/// - Medium files (<10MB): 256KB chunks
/// - Large files: 2MB chunks
pub mod adaptive {
    use super::*;

    pub fn report_file(file_path: &Path) -> Result<Vec<ReportFileRequest>, String> {
        // Determine chunk size based on file size
        let file_size = std::fs::metadata(file_path)
            .map_err(|e| e.to_string())?
            .len() as usize;

        let chunk_size = if file_size < 256 * 1024 {
            1024 // 1 KB for small files
        } else if file_size < 10 * 1024 * 1024 {
            256 * 1024 // 256 KB for medium files
        } else {
            2 * 1024 * 1024 // 2 MB for large files
        };

        let mut file = std::fs::File::open(file_path).map_err(|e| e.to_string())?;
        let mut requests = Vec::new();
        let mut buffer = vec![0u8; chunk_size];
        let mut first_chunk = true;

        loop {
            let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }

            let metadata = if first_chunk {
                first_chunk = false;
                Some(FileMetadata {
                    path: file_path.to_string_lossy().to_string(),
                    ..Default::default()
                })
            } else {
                None
            };

            requests.push(ReportFileRequest {
                task_id: 1,
                chunk: Some(File {
                    metadata,
                    chunk: buffer[..n].to_vec(),
                }),
            });
        }

        Ok(requests)
    }
}
