use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use md5::{Digest, Md5};

use crate::crypto::{decrypt_chunk_streaming, derive_key, load_key_file, read_header};
use crate::error::CokacencError;
use crate::naming;
use crate::pack::ChunkMetadata;

// ─── MetadataSplitWriter (extracts metadata from decrypted stream) ─────

enum SplitState {
    ReadingLen,
    ReadingMeta,
    Data,
}

/// Writer that splits the decrypted plaintext into metadata + file data.
/// Plaintext format: [4B meta_len LE u32][metadata JSON][file data...]
/// The metadata is buffered; file data is forwarded to the inner writer.
struct MetadataSplitWriter<'a, W: Write> {
    state: SplitState,
    len_buf: [u8; 4],
    len_filled: usize,
    meta_buf: Vec<u8>,
    meta_len: usize,
    inner: &'a mut W,
}

impl<'a, W: Write> MetadataSplitWriter<'a, W> {
    fn new(inner: &'a mut W) -> Self {
        Self {
            state: SplitState::ReadingLen,
            len_buf: [0u8; 4],
            len_filled: 0,
            meta_buf: Vec::new(),
            meta_len: 0,
            inner,
        }
    }

    fn take_metadata_bytes(&mut self) -> Result<Vec<u8>, CokacencError> {
        match self.state {
            SplitState::Data => Ok(std::mem::take(&mut self.meta_buf)),
            _ => Err(CokacencError::MetadataParse(
                "Incomplete metadata in chunk".to_string(),
            )),
        }
    }
}

impl<W: Write> Write for MetadataSplitWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let total = buf.len();
        let mut pos = 0;

        while pos < total {
            match self.state {
                SplitState::ReadingLen => {
                    let need = 4 - self.len_filled;
                    let take = need.min(total - pos);
                    self.len_buf[self.len_filled..self.len_filled + take]
                        .copy_from_slice(&buf[pos..pos + take]);
                    self.len_filled += take;
                    pos += take;
                    if self.len_filled == 4 {
                        self.meta_len = u32::from_le_bytes(self.len_buf) as usize;
                        self.meta_buf = Vec::with_capacity(self.meta_len);
                        if self.meta_len == 0 {
                            self.state = SplitState::Data;
                        } else {
                            self.state = SplitState::ReadingMeta;
                        }
                    }
                }
                SplitState::ReadingMeta => {
                    let need = self.meta_len - self.meta_buf.len();
                    let take = need.min(total - pos);
                    self.meta_buf.extend_from_slice(&buf[pos..pos + take]);
                    pos += take;
                    if self.meta_buf.len() == self.meta_len {
                        self.state = SplitState::Data;
                    }
                }
                SplitState::Data => {
                    self.inner.write_all(&buf[pos..])?;
                    pos = total;
                }
            }
        }

        Ok(total)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

// ─── TeeWriter (dual write to file + MD5 hasher) ──────────────────────

struct TeeWriter<'a, W: Write> {
    file: &'a mut W,
    hasher: &'a mut Md5,
}

impl<W: Write> Write for TeeWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.file.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

// ─── Unpack (decrypt) ──────────────────────────────────────────────────

/// Unpack (decrypt) all .cokacenc file groups in a directory.
/// Metadata is extracted from each chunk. After decryption, .cokacenc files are deleted if requested.
pub fn unpack_directory(dir: &Path, key_path: &Path, delete: bool) -> Result<(), CokacencError> {
    let password = load_key_file(key_path)?;
    let groups = naming::group_enc_files(dir)?;

    if groups.is_empty() {
        println!("No .cokacenc files found in {}", dir.display());
        return Ok(());
    }

    for (group_id, chunks) in &groups {
        println!("Unpacking: group {}... ({} chunk(s))", &group_id[..8.min(group_id.len())], chunks.len());
        match unpack_file_group(dir, chunks, &password) {
            Ok(original_name) => {
                if delete {
                    for chunk_info in chunks {
                        let _ = fs::remove_file(&chunk_info.path);
                    }
                }
                println!("  Done: {}", original_name);
            }
            Err(e) => {
                eprintln!("  Error (group {}): {}", group_id, e);
                return Err(e);
            }
        }
    }

    Ok(())
}

/// Decrypt and merge a group of chunk files into the original file.
/// Returns the original filename on success.
fn unpack_file_group(
    dir: &Path,
    chunks: &[naming::EncFileInfo],
    password: &[u8],
) -> Result<String, CokacencError> {
    if chunks.is_empty() {
        return Err(CokacencError::NoEncFiles("empty group".to_string()));
    }

    // Validate sequence continuity
    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.seq_index != i {
            let expected_label = naming::seq_label(i)?;
            return Err(CokacencError::MissingChunk { expected: expected_label });
        }
    }

    let group_id = &chunks[0].group_id;
    let temp_path = dir.join(format!(".{}.unpacking", group_id));

    let out_file = File::create(&temp_path)?;
    let mut file_writer = BufWriter::new(out_file);
    let mut md5_hasher = Md5::new();

    let mut original_name = String::new();
    let mut expected_md5 = String::new();
    let mut file_size = 0u64;
    let mut modified = 0i64;
    let mut permissions: u32 = 0;

    for (i, chunk_info) in chunks.iter().enumerate() {
        let enc_file = File::open(&chunk_info.path)?;
        let mut reader = BufReader::new(enc_file);

        let (salt, iv, _header_filename) = read_header(&mut reader)?;
        let key = derive_key(password, &salt);

        // Decrypt through MetadataSplitWriter -> TeeWriter(file, md5)
        let meta_bytes;
        {
            let mut tee = TeeWriter {
                file: &mut file_writer,
                hasher: &mut md5_hasher,
            };
            let mut split = MetadataSplitWriter::new(&mut tee);
            decrypt_chunk_streaming(&mut reader, &mut split, &key, &iv)?;
            meta_bytes = split.take_metadata_bytes()?;
        }

        let meta: ChunkMetadata = serde_json::from_slice(&meta_bytes)
            .map_err(|e| CokacencError::MetadataParse(e.to_string()))?;

        // Validate chunk metadata
        if meta.chunk_index != i {
            let _ = fs::remove_file(&temp_path);
            return Err(CokacencError::MetadataParse(
                format!("Chunk index mismatch: expected {}, got {}", i, meta.chunk_index),
            ));
        }

        if i == 0 {
            original_name = meta.filename.clone();
            expected_md5 = meta.file_md5.clone();
            file_size = meta.file_size;
            modified = meta.modified;
            permissions = meta.permissions;
            println!("  -> {}", original_name);
        } else {
            // Cross-check metadata consistency across chunks
            if meta.filename != original_name || (!expected_md5.is_empty() && meta.file_md5 != expected_md5) {
                let _ = fs::remove_file(&temp_path);
                return Err(CokacencError::MetadataParse(
                    "Inconsistent metadata across chunks".to_string(),
                ));
            }
        }
    }

    file_writer.flush()?;
    drop(file_writer);

    // Verify MD5 (skip if MD5 was not computed during encryption)
    let md5_hex = format!("{:032x}", md5_hasher.finalize());
    if !expected_md5.is_empty() && md5_hex != expected_md5 {
        let _ = fs::remove_file(&temp_path);
        return Err(CokacencError::Md5Mismatch {
            expected: expected_md5,
            actual: md5_hex,
        });
    }

    if expected_md5.is_empty() {
        println!("  MD5 verification: skipped");
    } else {
        println!("  MD5 verified: {}", md5_hex);
    }

    // Verify file size
    let actual_size = fs::metadata(&temp_path).map(|m| m.len()).unwrap_or(0);
    if actual_size != file_size {
        let _ = fs::remove_file(&temp_path);
        return Err(CokacencError::Other(
            format!("Size mismatch: expected {}, got {}", file_size, actual_size),
        ));
    }

    // Rename to original filename
    let out_path = dir.join(&original_name);
    fs::rename(&temp_path, &out_path)?;

    // Restore permissions
    #[cfg(unix)]
    if permissions != 0 {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&out_path, fs::Permissions::from_mode(permissions));
    }

    // Restore mtime
    #[cfg(unix)]
    if modified > 0 {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        if let Ok(cpath) = CString::new(out_path.as_os_str().as_bytes()) {
            let times = [
                libc::timespec { tv_sec: modified as libc::time_t, tv_nsec: 0 }, // atime
                libc::timespec { tv_sec: modified as libc::time_t, tv_nsec: 0 }, // mtime
            ];
            #[allow(unsafe_code)]
            unsafe {
                libc::utimensat(libc::AT_FDCWD, cpath.as_ptr(), times.as_ptr(), 0);
            }
        }
    }

    Ok(original_name)
}
