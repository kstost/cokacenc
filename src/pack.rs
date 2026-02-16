use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use md5::{Digest, Md5};
use serde::{Serialize, Deserialize};

use crate::crypto::{
    derive_key, generate_iv, generate_salt, load_key_file, write_header, ChunkEncryptor,
};
use crate::error::CokacencError;
use crate::naming;

const READ_BUF_SIZE: usize = 64 * 1024; // 64KB

// ─── Chunk metadata (embedded inside each encrypted chunk) ─────────────

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ChunkMetadata {
    #[serde(rename = "v")]
    pub version: u32,
    #[serde(rename = "group")]
    pub group_id: String,
    #[serde(rename = "name")]
    pub filename: String,
    #[serde(rename = "size")]
    pub file_size: u64,
    #[serde(rename = "md5")]
    pub file_md5: String,
    #[serde(rename = "mtime")]
    pub modified: i64,
    #[serde(rename = "perm")]
    pub permissions: u32,
    #[serde(rename = "chunks")]
    pub total_chunks: usize,
    #[serde(rename = "idx")]
    pub chunk_index: usize,
    #[serde(rename = "offset")]
    pub chunk_offset: u64,
    #[serde(rename = "len")]
    pub chunk_data_size: u64,
}

// ─── File info gathered in first pass ──────────────────────────────────

struct FileInfo {
    size: u64,
    md5: String,
    modified: i64,
    permissions: u32,
}

fn gather_file_info(path: &Path, use_md5: bool) -> Result<FileInfo, CokacencError> {
    let metadata = fs::metadata(path)?;
    let size = metadata.len();

    let modified = metadata.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    #[cfg(unix)]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    };
    #[cfg(not(unix))]
    let permissions = 0u32;

    let md5 = if use_md5 {
        // Compute MD5 (first pass)
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Md5::new();
        let mut buf = [0u8; READ_BUF_SIZE];
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
        }
        format!("{:032x}", hasher.finalize())
    } else {
        String::new()
    };

    Ok(FileInfo { size, md5, modified, permissions })
}

// ─── Pack (encrypt) ────────────────────────────────────────────────────

/// Pack (encrypt + split) all eligible files in a directory.
/// Uses 2-pass: first pass computes MD5+metadata, second pass encrypts.
/// Each chunk embeds full metadata.
pub fn pack_directory(
    dir: &Path,
    key_path: &Path,
    split_size_mb: u64,
    delete: bool,
    use_md5: bool,
) -> Result<(), CokacencError> {
    let password = load_key_file(key_path)?;
    let split_size = if split_size_mb == 0 {
        u64::MAX
    } else {
        split_size_mb * 1024 * 1024
    };

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            if !path.is_file() {
                return false;
            }
            let name = e.file_name().to_string_lossy().to_string();
            // Skip .cokacenc files, hidden files
            !name.ends_with(naming::EXT) && !name.starts_with('.')
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("No files to pack in {}", dir.display());
        return Ok(());
    }

    for entry in &entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        println!("Packing: {}", name);
        pack_file(&path, &name, dir, &password, split_size, delete, use_md5)?;
        println!("  Done: {}", name);
    }

    Ok(())
}

/// Pack a single file using 2-pass approach.
/// Pass 1: gather file info (MD5, size, mtime, permissions).
/// Pass 2: encrypt with metadata embedded in each chunk.
fn pack_file(
    file_path: &Path,
    original_name: &str,
    out_dir: &Path,
    password: &[u8],
    split_size: u64,
    delete: bool,
    use_md5: bool,
) -> Result<(), CokacencError> {
    // ── Pass 1: gather info ──
    let info = gather_file_info(file_path, use_md5)?;

    let group_id = loop {
        let id = naming::generate_group_id();
        if !naming::group_id_exists(out_dir, &id) {
            break id;
        }
    };
    let kp = naming::key_prefix(password);
    let total_chunks = if info.size == 0 {
        1
    } else {
        ((info.size + split_size - 1) / split_size) as usize
    };

    // ── Pass 2: encrypt ──
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut read_buf = [0u8; READ_BUF_SIZE];
    let mut created_chunks: Vec<std::path::PathBuf> = Vec::new();

    let result = (|| -> Result<(), CokacencError> {
        for chunk_idx in 0..total_chunks {
            let chunk_offset = chunk_idx as u64 * split_size;
            let chunk_data_size = if info.size == 0 {
                0
            } else {
                split_size.min(info.size - chunk_offset)
            };

            let metadata = ChunkMetadata {
                version: 2,
                group_id: group_id.clone(),
                filename: original_name.to_string(),
                file_size: info.size,
                file_md5: info.md5.clone(),
                modified: info.modified,
                permissions: info.permissions,
                total_chunks,
                chunk_index: chunk_idx,
                chunk_offset,
                chunk_data_size,
            };

            let chunk_path = naming::chunk_filename(out_dir, &kp, &group_id, chunk_idx)?;
            let chunk_file = File::create(&chunk_path)?;
            created_chunks.push(chunk_path);
            let mut writer = BufWriter::new(chunk_file);

            let salt = generate_salt();
            let iv = generate_iv();
            let key = derive_key(password, &salt);
            write_header(&mut writer, &salt, &iv, original_name)?;

            let mut enc = ChunkEncryptor::new(&key, &iv);

            // Write metadata length + metadata into encrypted stream
            let meta_bytes = serde_json::to_vec(&metadata)
                .map_err(|e| CokacencError::Other(format!("JSON serialize: {}", e)))?;
            let meta_len_bytes = (meta_bytes.len() as u32).to_le_bytes();

            let encrypted = enc.update(&meta_len_bytes);
            writer.write_all(encrypted)?;
            let encrypted = enc.update(&meta_bytes);
            writer.write_all(encrypted)?;

            // Write file data portion
            let mut remaining = chunk_data_size;
            while remaining > 0 {
                let to_read = (READ_BUF_SIZE as u64).min(remaining) as usize;
                let n = reader.read(&mut read_buf[..to_read])?;
                if n == 0 { break; }
                let encrypted = enc.update(&read_buf[..n]);
                writer.write_all(encrypted)?;
                remaining -= n as u64;
            }

            let final_block = enc.finalize();
            writer.write_all(&final_block)?;
            writer.flush()?;
        }

        Ok(())
    })();

    // On error, clean up any partial chunk files
    if result.is_err() {
        for path in &created_chunks {
            let _ = fs::remove_file(path);
        }
        return result;
    }

    if info.md5.is_empty() {
        println!(
            "  -> group {}, {} chunk(s), MD5: off ({})",
            group_id,
            total_chunks,
            format_size(info.size),
        );
    } else {
        println!(
            "  -> group {}, {} chunk(s), MD5: {} ({})",
            group_id,
            total_chunks,
            &info.md5[..8],
            format_size(info.size),
        );
    }

    // Delete original file if requested
    if delete {
        fs::remove_file(file_path)?;
    }

    result
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
