//! Windows-1250 → UTF-8 decoding for bank CSV exports.

use std::path::Path;

use encoding_rs::WINDOWS_1250;

/// Errors produced while obtaining Windows-1250 bytes to decode.
///
/// Decoding itself (`decode_windows1250`) cannot fail: `encoding_rs`
/// implements Windows-1250 as a *total* single-byte decoder — every byte
/// value 0x00-0xFF maps to a defined Unicode code point, so there is no
/// "invalid byte" case to report (verified empirically: no byte in that
/// range causes `WINDOWS_1250.decode` to flag an error). The only failure
/// that can realistically occur in this part of the pipeline is reading the
/// input file itself, so that's what this type represents.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// Reading the input file failed (not found, permission denied, etc.).
    #[error("failed to read input file: {0}")]
    Io(#[from] std::io::Error),
}

/// Decodes a Windows-1250 encoded byte buffer into a UTF-8 `String`.
///
/// This is infallible by construction: see [`DecodeError`] for why
/// Windows-1250 decoding itself has no error case.
pub fn decode_windows1250(bytes: &[u8]) -> String {
    let (text, _encoding_used, _had_errors) = WINDOWS_1250.decode(bytes);
    text.into_owned()
}

/// Reads a file and decodes its contents as Windows-1250 into UTF-8.
///
/// This is the only place a real failure can occur in the decode stage
/// (the read itself), so it's where [`DecodeError`] is actually produced.
///
/// # Errors
///
/// Returns `DecodeError::Io` if the file can't be read.
pub fn read_and_decode_windows1250(path: &Path) -> Result<String, DecodeError> {
    let bytes = std::fs::read(path)?;
    Ok(decode_windows1250(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::{check, let_assert};

    /// Plain ASCII bytes are valid in every single-byte Windows code page,
    /// so they should decode unchanged.
    #[test]
    fn decodes_ascii_bytes() {
        check!(decode_windows1250(b"hello") == "hello");
    }

    /// Windows-1250 maps high bytes like 0xB9 to Polish-specific characters
    /// (here, lowercase "ą") that aren't representable in plain ASCII.
    #[test]
    fn decodes_polish_diacritics() {
        check!(decode_windows1250(&[0xB9]) == "ą");
    }

    /// Reading a nonexistent file surfaces as `DecodeError::Io` rather than
    /// panicking or silently returning empty text.
    #[test]
    fn reports_io_error_for_missing_file() {
        let_assert!(
            Err(DecodeError::Io(_)) =
                read_and_decode_windows1250(Path::new("/nonexistent/sample_export.csv"))
        );
    }
}
