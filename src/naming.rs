use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use md5::{Digest, Md5};

use crate::error::CokacencError;

pub const EXT: &str = ".cokacenc";

/// Compute the first 5 hex chars of MD5(filename).
pub fn filename_md5_prefix(name: &str) -> String {
    let hash = Md5::digest(name.as_bytes());
    format!("{:032x}", hash)[..5].to_string()
}

/// Convert index to four-letter sequence label: 0→"aaaa", max 456975→"zzzz".
pub fn seq_label(index: usize) -> Result<String, CokacencError> {
    if index > 456_975 {
        return Err(CokacencError::SeqOverflow(index));
    }
    let a = b'a' + (index / (26 * 26 * 26)) as u8;
    let b = b'a' + ((index / (26 * 26)) % 26) as u8;
    let c = b'a' + ((index / 26) % 26) as u8;
    let d = b'a' + (index % 26) as u8;
    Ok(format!("{}{}{}{}", a as char, b as char, c as char, d as char))
}

/// Parse a four-letter sequence label back to index.
fn parse_seq_label(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.len() != 4 {
        return None;
    }
    let a = bytes[0].checked_sub(b'a')? as usize;
    let b = bytes[1].checked_sub(b'a')? as usize;
    let c = bytes[2].checked_sub(b'a')? as usize;
    let d = bytes[3].checked_sub(b'a')? as usize;
    if a > 25 || b > 25 || c > 25 || d > 25 {
        return None;
    }
    Some(a * 26 * 26 * 26 + b * 26 * 26 + c * 26 + d)
}

/// Temporary chunk name during pack (before content MD5 is known).
/// Format: `<fnmd5_5>.SPLTD.TEMP.<seq>.<original_name>.cokacenc`
pub fn temp_chunk_name(dir: &Path, original_name: &str, seq: usize) -> Result<PathBuf, CokacencError> {
    let label = seq_label(seq)?;
    let fnmd5 = filename_md5_prefix(original_name);
    Ok(dir.join(format!("{}.SPLTD.TEMP.{}.{}{}", fnmd5, label, original_name, EXT)))
}

/// Temporary name for single-file encryption (before content MD5 is known).
/// Format: `<fnmd5_5>.TEMP.<original_name>.cokacenc`
pub fn temp_single_name(dir: &Path, original_name: &str) -> PathBuf {
    let fnmd5 = filename_md5_prefix(original_name);
    dir.join(format!("{}.TEMP.{}{}", fnmd5, original_name, EXT))
}

/// Final chunk name with content MD5 prefix.
/// Format: `<fnmd5_5>.SPLTD.<content_md5_8>.<seq>.<original_name>.cokacenc`
pub fn final_chunk_name(
    dir: &Path,
    original_name: &str,
    md5_hex: &str,
    seq: usize,
) -> Result<PathBuf, CokacencError> {
    let label = seq_label(seq)?;
    let fnmd5 = filename_md5_prefix(original_name);
    let md5_prefix = &md5_hex[..8.min(md5_hex.len())];
    Ok(dir.join(format!(
        "{}.SPLTD.{}.{}.{}{}",
        fnmd5, md5_prefix, label, original_name, EXT
    )))
}

/// Final single-file encrypted name.
/// Format: `<fnmd5_5>.<content_md5_8>.<original_name>.cokacenc`
pub fn single_file_enc_name(dir: &Path, original_name: &str, md5_hex: &str) -> PathBuf {
    let fnmd5 = filename_md5_prefix(original_name);
    let md5_prefix = &md5_hex[..8.min(md5_hex.len())];
    dir.join(format!("{}.{}.{}{}", fnmd5, md5_prefix, original_name, EXT))
}

/// Parsed info from a .cokacenc filename.
#[derive(Debug, Clone)]
pub struct EncFileInfo {
    pub original_name: String,
    pub is_split: bool,
    pub md5_fragment: String, // 8-char content MD5 prefix
    pub seq_index: Option<usize>,
    pub path: PathBuf,
}

/// Parse a .cokacenc filename into its components.
///
/// Single format: `<fnmd5_5>.<content_md5_8>.<original_name>.cokacenc`
/// Split format:  `<fnmd5_5>.SPLTD.<content_md5_8>.<seq>.<original_name>.cokacenc`
pub fn parse_enc_filename(path: &Path) -> Option<EncFileInfo> {
    let filename = path.file_name()?.to_str()?;
    if !filename.ends_with(EXT) {
        return None;
    }
    // Remove .cokacenc suffix
    let base = &filename[..filename.len() - EXT.len()];

    // Both formats start with 5 hex chars (fnmd5) followed by a dot
    if base.len() < 6 {
        return None;
    }
    let fnmd5_part = &base[..5];
    if !fnmd5_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    if base.as_bytes()[5] != b'.' {
        return None;
    }
    let after_fnmd5 = &base[6..]; // after "<fnmd5>."

    // Try split format: SPLTD.<content_md5_8>.<seq>.<original_name>
    if let Some(rest) = after_fnmd5.strip_prefix("SPLTD.") {
        // rest = "<content_md5_8>.<seq>.<original_name>"
        if rest.len() < 14 {
            return None;
        }
        let md5_fragment = &rest[..8];
        if !md5_fragment.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        if rest.as_bytes()[8] != b'.' {
            return None;
        }
        let seq_str = &rest[9..13];
        let seq_index = parse_seq_label(seq_str)?;
        if rest.as_bytes()[13] != b'.' {
            return None;
        }
        let original_name = &rest[14..];
        if original_name.is_empty() {
            return None;
        }

        let expected_fnmd5 = filename_md5_prefix(original_name);
        if fnmd5_part != expected_fnmd5 {
            return None;
        }

        return Some(EncFileInfo {
            original_name: original_name.to_string(),
            is_split: true,
            md5_fragment: md5_fragment.to_string(),
            seq_index: Some(seq_index),
            path: path.to_path_buf(),
        });
    }

    // Try single format: <content_md5_8>.<original_name>
    if after_fnmd5.len() < 10 {
        // 8 + "." + at least 1 char
        return None;
    }
    let md5_part = &after_fnmd5[..8];
    if !md5_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    if after_fnmd5.as_bytes()[8] != b'.' {
        return None;
    }
    let original_name = &after_fnmd5[9..];
    if original_name.is_empty() {
        return None;
    }

    let expected_fnmd5 = filename_md5_prefix(original_name);
    if fnmd5_part != expected_fnmd5 {
        return None;
    }

    Some(EncFileInfo {
        original_name: original_name.to_string(),
        is_split: false,
        md5_fragment: md5_part.to_string(),
        seq_index: None,
        path: path.to_path_buf(),
    })
}

/// Group .cokacenc files in a directory by their original filename.
/// Returns a map: original_name → sorted list of EncFileInfo.
pub fn group_enc_files(dir: &Path) -> Result<BTreeMap<String, Vec<EncFileInfo>>, CokacencError> {
    let mut groups: BTreeMap<String, Vec<EncFileInfo>> = BTreeMap::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(info) = parse_enc_filename(&path) {
            groups
                .entry(info.original_name.clone())
                .or_default()
                .push(info);
        }
    }

    // Sort each group by seq_index (None = single file, Some(n) = split chunk)
    for files in groups.values_mut() {
        files.sort_by_key(|f| f.seq_index.unwrap_or(0));
    }

    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq_label() {
        assert_eq!(seq_label(0).unwrap(), "aaaa");
        assert_eq!(seq_label(1).unwrap(), "aaab");
        assert_eq!(seq_label(25).unwrap(), "aaaz");
        assert_eq!(seq_label(26).unwrap(), "aaba");
        assert_eq!(seq_label(675).unwrap(), "aazz");
        assert_eq!(seq_label(676).unwrap(), "abaa");
        assert_eq!(seq_label(456_975).unwrap(), "zzzz");
        assert!(seq_label(456_976).is_err());
    }

    #[test]
    fn test_parse_seq_label() {
        assert_eq!(parse_seq_label("aaaa"), Some(0));
        assert_eq!(parse_seq_label("aaaz"), Some(25));
        assert_eq!(parse_seq_label("aaba"), Some(26));
        assert_eq!(parse_seq_label("aazz"), Some(675));
        assert_eq!(parse_seq_label("zzzz"), Some(456_975));
        assert_eq!(parse_seq_label("a"), None);
        assert_eq!(parse_seq_label("aa"), None);
        assert_eq!(parse_seq_label("aaa"), None);
    }

    #[test]
    fn test_filename_md5_prefix() {
        let prefix = filename_md5_prefix("myfile.txt");
        assert_eq!(prefix.len(), 5);
        assert!(prefix.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_parse_split_filename() {
        let fnmd5 = filename_md5_prefix("myfile.txt");
        let name = format!("/tmp/{}.SPLTD.abcd1234.aaaa.myfile.txt.cokacenc", fnmd5);
        let path = PathBuf::from(&name);
        let info = parse_enc_filename(&path).unwrap();
        assert_eq!(info.original_name, "myfile.txt");
        assert!(info.is_split);
        assert_eq!(info.md5_fragment, "abcd1234");
        assert_eq!(info.seq_index, Some(0));
    }

    #[test]
    fn test_parse_single_filename() {
        let fnmd5 = filename_md5_prefix("myfile.txt");
        let name = format!("/tmp/{}.abcd1234.myfile.txt.cokacenc", fnmd5);
        let path = PathBuf::from(&name);
        let info = parse_enc_filename(&path).unwrap();
        assert_eq!(info.original_name, "myfile.txt");
        assert!(!info.is_split);
        assert_eq!(info.md5_fragment, "abcd1234");
        assert_eq!(info.seq_index, None);
    }

    #[test]
    fn test_roundtrip_single_name() {
        let dir = Path::new("/tmp");
        let original = "my document.pdf";
        let md5 = "abcdef0123456789abcdef0123456789";
        let path = single_file_enc_name(dir, original, md5);
        let info = parse_enc_filename(&path).unwrap();
        assert_eq!(info.original_name, original);
        assert_eq!(info.md5_fragment, &md5[..8]);
        assert!(!info.is_split);
    }

    #[test]
    fn test_roundtrip_split_name() {
        let dir = Path::new("/tmp");
        let original = "archive.tar.gz";
        let md5 = "abcdef0123456789abcdef0123456789";
        let path = final_chunk_name(dir, original, md5, 0).unwrap();
        let info = parse_enc_filename(&path).unwrap();
        assert_eq!(info.original_name, original);
        assert_eq!(info.md5_fragment, &md5[..8]);
        assert!(info.is_split);
        assert_eq!(info.seq_index, Some(0));
    }
}
