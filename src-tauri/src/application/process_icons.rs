//! Presentation-only process icon enrichment for runtime snapshots.

use crate::domain::process::ProcessInfo;

pub const ICON_PNG_MAX_BYTES: usize = 64 * 1024;
pub const SNAPSHOT_ICON_MAX_BYTES: usize = 2 * 1024 * 1024;
pub const SNAPSHOT_ICON_MAX_COUNT: usize = 128;
pub const ICON_MAX_DIMENSION: u32 = 128;

/// Platform icon adapter. Failure and absence both mean the UI uses its fallback.
pub trait ProcessIconProvider: Send + Sync {
    fn icon_png(&self, process: &ProcessInfo) -> Option<Vec<u8>>;
}

/// A runtime-snapshot-scoped PNG asset. This value is presentation data only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessIconAsset {
    pub reference: String,
    pub png: Vec<u8>,
}

/// Reject non-PNG, oversized, or implausibly large images before IPC encoding.
pub fn is_bounded_png(bytes: &[u8]) -> bool {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || bytes.len() > ICON_PNG_MAX_BYTES || &bytes[..8] != PNG_SIGNATURE {
        return false;
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if width == 0 || height == 0 || width > ICON_MAX_DIMENSION || height > ICON_MAX_DIMENSION {
        return false;
    }
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_limits(png::Limits { bytes: 256 * 1024 });
    let Ok(mut reader) = decoder.read_info() else {
        return false;
    };
    if reader.info().width != width || reader.info().height != height {
        return false;
    }
    let Some(buffer_size) = reader.output_buffer_size() else {
        return false;
    };
    if buffer_size > (ICON_MAX_DIMENSION as usize * ICON_MAX_DIMENSION as usize * 4) {
        return false;
    }
    let mut output = vec![0; buffer_size];
    reader.next_frame(&mut output).is_ok()
}

#[cfg(test)]
mod tests {
    use super::{is_bounded_png, ICON_PNG_MAX_BYTES};

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut encoded = Vec::new();
        let mut encoder = png::Encoder::new(&mut encoded, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("PNG header");
        writer
            .write_image_data(&vec![0; width as usize * height as usize * 4])
            .expect("PNG pixels");
        drop(writer);
        encoded
    }

    #[test]
    fn icon_payload_bounds_reject_non_png_and_excessive_dimensions() {
        assert!(is_bounded_png(&png(64, 64)));
        assert!(!is_bounded_png(b"not a png"));
        assert!(!is_bounded_png(&png(129, 64)));
        assert!(!is_bounded_png(&vec![0; ICON_PNG_MAX_BYTES + 1]));
    }
}
