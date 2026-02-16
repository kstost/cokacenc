#![allow(dead_code)]

mod crypto;
mod error;
mod naming;
mod pack;
mod unpack;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rand::RngCore;

#[derive(Parser)]
#[command(
    name = "cokacenc",
    version,
    about = "AES-256-CBC file encryption + split tool",
    long_about = "\
AES-256-CBC file encryption + split tool (v2 format)

cokacenc encrypts files in a directory using AES-256-CBC and
optionally splits them into chunks of a specified size.
Uses 2-pass processing with metadata embedded in each chunk.

━━━ How It Works ━━━

  pack   : Pass 1: gather metadata (+ compute MD5 with --md5)
           Pass 2: encrypt with metadata embedded in each chunk (→ delete original with --delete)
  unpack : Group .cokacenc files by group ID → decrypt in order → extract metadata
           → merge into single file → MD5 verify (if available) → restore permissions/mtime (→ delete .cokacenc with --delete)

━━━ Encryption Details ━━━

  Algorithm       : AES-256-CBC (PKCS7 padding)
  Key derivation  : PBKDF2-HMAC-SHA512, 100,000 iterations
  Salt/IV         : Independent 16-byte random per chunk
  Integrity check : Full MD5 hash (embedded in chunk metadata, optional with --md5)

  → Each chunk contains full file metadata (name, size, MD5, permissions, mtime).
  → Each chunk can be decrypted independently.

━━━ Chunk File Format (44-byte header + ciphertext) ━━━

  Header    : [8B magic \"COKACENC\"][4B version LE (=2)][16B PBKDF2 salt][16B AES IV]
  Plaintext : [4B meta_len LE u32][metadata JSON][file data...]

━━━ Output Filename Convention (v2) ━━━

  <group_id 16hex>_<seq 4letter>.cokacenc
  group_id = 8 random bytes (16 hex chars), seq = aaaa, aaab, ... zzzz (max 456,976)
  Original filename is stored inside encrypted metadata, not in the filename.

━━━ Key File ━━━

  A key file is any text file whose contents are used directly as the password.
  Leading and trailing whitespace is automatically trimmed.

━━━ Examples ━━━

  # Generate a key file
  cokacenc generate --output secret.key

  # Encrypt all files in a directory (default 1800MB chunk split)
  cokacenc pack --dir ./data --key secret.key

  # Encrypt with 500MB chunk split
  cokacenc pack --dir ./data --key secret.key --size 500

  # Decrypt encrypted files
  cokacenc unpack --dir ./data --key secret.key

━━━ Notes ━━━

  - With --delete, original files are removed after successful pack.
  - With --delete, .cokacenc files are removed after successful unpack.
  - Hidden files (starting with .) and .cokacenc files are excluded from pack.
  - The same key file must be used for both pack and unpack.
  - v2 format is NOT compatible with v1 encrypted files."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt and split files in a directory (v2 format)
    ///
    /// Encrypts all regular files in the specified directory using AES-256-CBC.
    /// Files exceeding --size are automatically split into multiple chunks.
    /// Each chunk embeds full metadata (filename, MD5, size, permissions, mtime).
    ///
    /// Processing flow (2-pass):
    ///   1. Pass 1: Gather metadata (size, mtime, permissions) + compute MD5 hash (with --md5)
    ///   2. Pass 2: Encrypt with metadata embedded at the start of each chunk's plaintext
    ///      (each chunk gets an independent salt/IV)
    ///   3. With --delete, remove the original file
    ///
    /// Output filenames:
    ///   <group_id 16hex>_<seq 4letter>.cokacenc  (seq: aaaa~zzzz)
    ///   Original filename is stored inside encrypted metadata.
    ///
    /// Excluded from processing:
    ///   - Hidden files (starting with .)
    ///   - Already encrypted files (ending with .cokacenc)
    ///
    /// Examples:
    ///   cokacenc pack --dir ./mydir --key secret.key
    ///   cokacenc pack --dir ./mydir --key secret.key --size 500
    Pack {
        /// Directory path containing files to encrypt
        ///
        /// All regular files in this directory will be encrypted.
        /// Subdirectories are not traversed.
        /// Encrypted .cokacenc files are created in the same directory.
        #[arg(long, value_name = "PATH")]
        dir: PathBuf,

        /// Key file path (used as password)
        ///
        /// The text file contents are used as input for PBKDF2-HMAC-SHA512 key derivation.
        /// Leading/trailing whitespace is automatically trimmed.
        /// The same file is required for unpack.
        #[arg(long, value_name = "FILE")]
        key: PathBuf,

        /// Maximum chunk size in MB
        ///
        /// If the plaintext size exceeds this value, the file is split into multiple chunks.
        /// Default is 1800MB (approximately 1.76GB).
        #[arg(long, default_value = "1800", value_name = "MB")]
        size: u64,

        /// Delete original files after successful encryption
        ///
        /// When specified, the original file is deleted after encryption succeeds.
        /// Without this option, original files are kept as-is.
        #[arg(long)]
        delete: bool,

        /// Compute and embed MD5 hash for integrity verification
        ///
        /// When specified, an MD5 hash of the original file is computed (first pass)
        /// and embedded in chunk metadata. During unpack, the hash is verified.
        /// Without this option, MD5 computation is skipped for faster encryption.
        #[arg(long)]
        md5: bool,
    },

    /// Generate a random key file
    ///
    /// Generates a key file using cryptographically secure random bytes.
    /// Produces a Base64-encoded string of 64 bytes (512 bits) by default.
    ///
    /// If the file already exists, an error is raised instead of overwriting.
    /// Use --force to overwrite an existing file.
    ///
    /// Examples:
    ///   cokacenc generate --output secret.key
    ///   cokacenc generate --output secret.key --length 128
    ///   cokacenc generate --output secret.key --force
    Generate {
        /// Output key file path
        ///
        /// A random key file will be created at this path.
        /// An error occurs if the file already exists (use --force to override).
        #[arg(long, value_name = "FILE")]
        output: PathBuf,

        /// Key length in bytes (before Base64 encoding)
        ///
        /// The number of random bytes to generate. Since the output is Base64-encoded,
        /// the actual file size will be approximately 33% larger.
        /// Default is 64 bytes (512 bits).
        #[arg(long, default_value = "64", value_name = "BYTES")]
        length: usize,

        /// Overwrite existing file
        #[arg(long)]
        force: bool,
    },

    /// Decrypt and merge .cokacenc files in a directory (v2 format)
    ///
    /// Decrypts .cokacenc files in the specified directory and restores the original files.
    /// Original filename, permissions, and mtime are restored from embedded metadata.
    ///
    /// Processing flow:
    ///   1. Group .cokacenc files by group ID (from filename)
    ///   2. Decrypt each group's chunks in sequence order (aaaa, aaab, ...)
    ///   3. Extract metadata from each chunk, merge file data while computing MD5 hash
    ///   4. Verify integrity by comparing with the full MD5 from metadata (skipped if MD5 was not computed during pack)
    ///   5. Restore original filename, permissions, and mtime
    ///   6. With --delete, remove the .cokacenc files
    ///
    /// MD5 verification:
    ///   - Full 32-character MD5 comparison against metadata (if MD5 was computed during pack)
    ///   - On mismatch, the decrypted file is deleted and an error is raised
    ///   - If pack was done without --md5, verification is automatically skipped
    ///
    /// Examples:
    ///   cokacenc unpack --dir ./mydir --key secret.key
    Unpack {
        /// Directory path containing .cokacenc files
        ///
        /// The .cokacenc files in this directory will be decrypted.
        /// Restored original files are created in the same directory.
        #[arg(long, value_name = "PATH")]
        dir: PathBuf,

        /// Key file path (used as password)
        ///
        /// Must be the same key file used during pack.
        /// Using a different key file will cause decryption to fail (padding error).
        #[arg(long, value_name = "FILE")]
        key: PathBuf,

        /// Delete .cokacenc files after successful decryption
        ///
        /// When specified, .cokacenc files are deleted after decryption and MD5
        /// verification succeed. Without this option, .cokacenc files are kept as-is.
        #[arg(long)]
        delete: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Pack {
            dir,
            key,
            size,
            delete,
            md5,
        } => pack::pack_directory(&dir, &key, size, delete, md5),
        Commands::Generate {
            output,
            length,
            force,
        } => generate_key(&output, length, force),
        Commands::Unpack { dir, key, delete } => unpack::unpack_directory(&dir, &key, delete),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn generate_key(
    output: &std::path::Path,
    length: usize,
    force: bool,
) -> Result<(), error::CokacencError> {
    if !force && output.exists() {
        return Err(error::CokacencError::Other(format!(
            "File already exists: {} (use --force to overwrite)",
            output.display()
        )));
    }

    let mut raw = vec![0u8; length];
    rand::thread_rng().fill_bytes(&mut raw);

    // Base64 encode (URL-safe, no padding) without extra dependencies
    let encoded = base64_encode(&raw);

    std::fs::write(output, encoded.as_bytes())?;
    println!("Generated key file: {} ({} random bytes, {} chars Base64)", output.display(), length, encoded.len());
    Ok(())
}

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARSET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARSET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[((triple >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(CHARSET[(triple & 0x3F) as usize] as char);
        }
    }
    out
}
