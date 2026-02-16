use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use md5::{Digest, Md5};

use crate::crypto::{
    derive_key, generate_iv, generate_salt, load_key_file, write_header, ChunkEncryptor,
};
use crate::error::CokacencError;
use crate::naming;

const READ_BUF_SIZE: usize = 64 * 1024; // 64KB

/// Pack (encrypt + split) all eligible files in a directory.
pub fn pack_directory(
    dir: &Path,
    key_path: &Path,
    split_size_mb: u64,
    delete: bool,
) -> Result<(), CokacencError> {
    let password = load_key_file(key_path)?;
    let split_size = split_size_mb * 1024 * 1024;

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            if !path.is_file() {
                return false;
            }
            let name = e.file_name().to_string_lossy().to_string();
            // Skip .cokacenc files, hidden files
            !name.ends_with(crate::naming::EXT) && !name.starts_with('.')
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
        pack_file(&path, &name, dir, &password, split_size, delete)?;
        println!("  Done: {}", name);
    }

    Ok(())
}

/// Pack a single file: 1-pass streaming read → MD5 + encrypt → split into chunks.
fn pack_file(
    file_path: &Path,
    original_name: &str,
    out_dir: &Path,
    password: &[u8],
    split_size: u64,
    delete: bool,
) -> Result<(), CokacencError> {
    let file = File::open(file_path)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::new(file);
    let mut md5_hasher = Md5::new();

    let mut read_buf = [0u8; READ_BUF_SIZE];
    let mut chunk_index: usize = 0;
    let mut chunk_plaintext_written: u64 = 0;
    let mut temp_paths: Vec<std::path::PathBuf> = Vec::new();

    // Start first chunk
    let (mut enc, mut writer, temp_path) =
        start_new_chunk(out_dir, original_name, chunk_index, password)?;
    temp_paths.push(temp_path);

    loop {
        let bytes_read = reader.read(&mut read_buf)?;
        if bytes_read == 0 {
            break;
        }

        let data = &read_buf[..bytes_read];
        md5_hasher.update(data);

        let mut offset = 0;
        while offset < data.len() {
            let remaining_in_chunk = if split_size > 0 && split_size > chunk_plaintext_written {
                (split_size - chunk_plaintext_written) as usize
            } else if split_size == 0 {
                data.len() - offset
            } else {
                0
            };

            if remaining_in_chunk == 0 {
                // Finalize current chunk and start new one
                let final_block = enc.finalize();
                writer.write_all(&final_block)?;
                writer.flush()?;

                chunk_index += 1;
                let (new_enc, new_writer, temp_path) =
                    start_new_chunk(out_dir, original_name, chunk_index, password)?;
                enc = new_enc;
                writer = new_writer;
                temp_paths.push(temp_path);
                chunk_plaintext_written = 0;
                continue;
            }

            let consume = remaining_in_chunk.min(data.len() - offset);
            let encrypted = enc.update(&data[offset..offset + consume]);
            writer.write_all(encrypted)?;
            chunk_plaintext_written += consume as u64;
            offset += consume;
        }
    }

    // Finalize last chunk
    let final_block = enc.finalize();
    writer.write_all(&final_block)?;
    writer.flush()?;
    drop(writer);

    // Compute MD5 hex
    let md5_result = md5_hasher.finalize();
    let md5_hex = format!("{:032x}", md5_result);

    let total_chunks = chunk_index + 1;

    // Rename temp files to final names
    if total_chunks == 1 {
        let final_path = naming::single_file_enc_name(out_dir, original_name, &md5_hex);
        fs::rename(&temp_paths[0], &final_path)?;
        println!("  → {} ({})", final_path.file_name().unwrap().to_string_lossy(), format_size(file_size));
    } else {
        for (i, temp_path) in temp_paths.iter().enumerate() {
            let final_path = naming::final_chunk_name(out_dir, original_name, &md5_hex, i)?;
            fs::rename(temp_path, &final_path)?;
        }
        println!(
            "  → {} chunks, MD5 prefix: {}, total: {}",
            total_chunks,
            &md5_hex[..8],
            format_size(file_size)
        );
    }

    // Delete original file if requested
    if delete {
        fs::remove_file(file_path)?;
    }

    Ok(())
}

/// Start a new encrypted chunk file.
fn start_new_chunk(
    out_dir: &Path,
    original_name: &str,
    chunk_index: usize,
    password: &[u8],
) -> Result<(ChunkEncryptor, BufWriter<File>, std::path::PathBuf), CokacencError> {
    let temp_path = naming::temp_chunk_name(out_dir, original_name, chunk_index)?;
    let file = File::create(&temp_path)?;
    let mut writer = BufWriter::new(file);

    let salt = generate_salt();
    let iv = generate_iv();
    let key = derive_key(password, &salt);

    write_header(&mut writer, &salt, &iv)?;

    let enc = ChunkEncryptor::new(&key, &iv);

    Ok((enc, writer, temp_path))
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
