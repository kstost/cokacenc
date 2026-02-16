# cokacenc

AES-256-CBC file encryption + split tool built with Rust.

Encrypts files in a directory and optionally splits them into chunks of a specified size. All operations are single-pass streaming — one read, one write, no temporary files.

## Features

- **AES-256-CBC** encryption with PKCS7 padding
- **PBKDF2-HMAC-SHA512** key derivation (100,000 iterations)
- **Independent salt/IV** per chunk — each chunk is independently decryptable
- **MD5 integrity verification** embedded in filenames
- **1-pass streaming** — efficient for large files, no temp files
- **Auto-split** large files into configurable chunk sizes (default 1.8GB)
- **Cross-platform** — Linux and macOS (x86_64 + ARM64)

## Install

```bash
/bin/bash -c "$(curl -fsSL https://cokacenc.cokac.com/install.sh)"
```

Or download a binary from [Releases](https://github.com/kstost/cokacenc/releases):

| Binary | Platform |
|--------|----------|
| `cokacenc-linux-aarch64` | Linux ARM64 |
| `cokacenc-linux-x86_64` | Linux x86_64 |
| `cokacenc-macos-aarch64` | macOS Apple Silicon |
| `cokacenc-macos-x86_64` | macOS Intel |

## Quick Start

```bash
# 1. Generate a key file
cokacenc generate --output secret.key

# 2. Encrypt all files in a directory
cokacenc pack --dir ./data --key secret.key

# 3. Decrypt files
cokacenc unpack --dir ./data --key secret.key
```

## Commands

### `generate` — Create a key file

```bash
cokacenc generate --output secret.key
cokacenc generate --output secret.key --length 128
cokacenc generate --output secret.key --force    # overwrite existing
```

| Option | Description |
|--------|-------------|
| `--output <FILE>` | Output key file path (required) |
| `--length <BYTES>` | Key length in bytes, default 64 |
| `--force` | Overwrite existing file |

### `pack` — Encrypt and split

```bash
cokacenc pack --dir ./data --key secret.key
cokacenc pack --dir ./data --key secret.key --size 500   # 500MB chunks
cokacenc pack --dir ./data --key secret.key --size 0     # no split
cokacenc pack --dir ./data --key secret.key --delete      # remove originals
```

| Option | Description |
|--------|-------------|
| `--dir <PATH>` | Directory containing files to encrypt (required) |
| `--key <FILE>` | Key file path (required) |
| `--size <MB>` | Max chunk size in MB, default 1800 (0 = no split) |
| `--delete` | Delete original files after encryption |

- Hidden files (`.` prefix) and `.cokacenc` files are excluded.
- Subdirectories are not traversed.

### `unpack` — Decrypt and merge

```bash
cokacenc unpack --dir ./data --key secret.key
cokacenc unpack --dir ./data --key secret.key --delete    # remove .cokacenc files
```

| Option | Description |
|--------|-------------|
| `--dir <PATH>` | Directory containing .cokacenc files (required) |
| `--key <FILE>` | Key file path (required) |
| `--delete` | Delete .cokacenc files after decryption |

- Split chunks are automatically grouped and merged in order.
- MD5 integrity is verified after decryption. On mismatch, the output file is removed.

## Output Filename Convention

**Single file (no split):**

```
<fnMD5:5>.<contentMD5:8>.<original name>.cokacenc
```

**Split file:**

```
<fnMD5:5>.SPLTD.<contentMD5:8>.<seq>.<original name>.cokacenc
```

- `fnMD5` — first 5 hex chars of MD5(original filename)
- `contentMD5` — first 8 hex chars of MD5(file content)
- `seq` — 4-letter sequence `aaaa` through `zzzz` (max 456,976 chunks)

Example:

```
2e541.d4e90967.report.pdf.cokacenc
77b55.SPLTD.a8f3bc01.aaaa.database.sql.cokacenc
77b55.SPLTD.a8f3bc01.aaab.database.sql.cokacenc
77b55.SPLTD.a8f3bc01.aaac.database.sql.cokacenc
```

## Chunk File Format

Each `.cokacenc` chunk has a 44-byte header followed by ciphertext:

```
[8B magic "COKACENC"][4B version LE][16B PBKDF2 salt][16B AES IV][...ciphertext...]
```

- Magic bytes identify the file format
- Each chunk has its own random salt and IV
- Chunks can be decrypted independently

## Encryption Details

| Property | Value |
|----------|-------|
| Algorithm | AES-256-CBC (PKCS7 padding) |
| Key derivation | PBKDF2-HMAC-SHA512, 100,000 iterations |
| Salt | 16 bytes, random per chunk |
| IV | 16 bytes, random per chunk |
| Integrity | MD5 hash embedded in filename |
| Streaming | 64KB buffer, single-pass |

## Key File

A key file is any text file. Its contents are used directly as the password for PBKDF2 key derivation. Leading and trailing whitespace is automatically trimmed.

```bash
# Generate a random key (recommended)
cokacenc generate --output secret.key

# Or use any text file
echo "my secret passphrase" > secret.key
```

## Build

Requires Python 3.6+ and internet connection for initial tool setup.

```bash
# Build for current platform
python3 build.py

# Build for all platforms
python3 build.py --all

# macOS cross-compile
python3 build.py --macos

# See all options
python3 build.py --help
```

See [build_manual.md](build_manual.md) for detailed build system documentation.

## License

MIT
