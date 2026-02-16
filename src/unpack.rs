use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use md5::{Digest, Md5};

use crate::crypto::{decrypt_chunk_streaming, derive_key, load_key_file, read_header};
use crate::error::CokacencError;
use crate::naming::{self, EncFileInfo};

/// Writer wrapper that feeds data to both an output file and an MD5 hasher.
struct TeeWriter<'a> {
    inner: BufWriter<File>,
    hasher: &'a mut Md5,
}

impl<'a> Write for TeeWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Unpack (decrypt + merge) all .cokacenc file groups in a directory.
pub fn unpack_directory(dir: &Path, key_path: &Path, delete: bool) -> Result<(), CokacencError> {
    let password = load_key_file(key_path)?;
    let groups = naming::group_enc_files(dir)?;

    if groups.is_empty() {
        println!("No .cokacenc files found in {}", dir.display());
        return Ok(());
    }

    for (original_name, files) in &groups {
        println!("Unpacking: {} ({} chunk(s))", original_name, files.len());
        unpack_file_group(dir, original_name, files, &password, delete)?;
        println!("  Done: {}", original_name);
    }

    Ok(())
}

/// Decrypt and merge a group of chunk files into the original file.
fn unpack_file_group(
    dir: &Path,
    original_name: &str,
    files: &[EncFileInfo],
    password: &[u8],
    delete: bool,
) -> Result<(), CokacencError> {
    if files.is_empty() {
        return Err(CokacencError::NoEncFiles(original_name.to_string()));
    }

    // Validate chunk sequence continuity for split files
    if files[0].is_split {
        for (i, f) in files.iter().enumerate() {
            let expected = i;
            let actual = f.seq_index.unwrap_or(0);
            if actual != expected {
                let expected_label = naming::seq_label(expected)?;
                return Err(CokacencError::MissingChunk {
                    expected: expected_label,
                });
            }
        }
    }

    let expected_md5 = &files[0].md5_fragment;

    let out_path = dir.join(original_name);
    let temp_path = dir.join(format!(".{}.unpacking", original_name));

    let out_file = File::create(&temp_path)?;
    let mut md5_hasher = Md5::new();

    {
        let mut tee = TeeWriter {
            inner: BufWriter::new(out_file),
            hasher: &mut md5_hasher,
        };

        for file_info in files {
            let enc_file = File::open(&file_info.path)?;
            let mut reader = BufReader::new(enc_file);

            // Read header to get salt and IV
            let (salt, iv) = read_header(&mut reader)?;
            let key = derive_key(password, &salt);

            // Decrypt chunk
            decrypt_chunk_streaming(&mut reader, &mut tee, &key, &iv)?;
        }

        tee.flush()?;
    }

    // Verify MD5
    let md5_result = md5_hasher.finalize();
    let md5_hex = format!("{:032x}", md5_result);

    // Both single and split use 8-char MD5 prefix
    let md5_matches = md5_hex.starts_with(expected_md5);

    if !md5_matches {
        // Clean up temp file
        let _ = fs::remove_file(&temp_path);
        return Err(CokacencError::Md5Mismatch {
            expected: expected_md5.to_string(),
            actual: md5_hex,
        });
    }

    println!("  MD5 verified: {}", md5_hex);

    // Rename temp to final
    fs::rename(&temp_path, &out_path)?;

    // Delete .cokacenc files if requested
    if delete {
        for file_info in files {
            fs::remove_file(&file_info.path)?;
        }
    }

    Ok(())
}
