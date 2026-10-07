//! Optional icon extraction from an OS-observed executable path.

use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, RGBQUAD,
};
use windows_sys::Win32::UI::Shell::ExtractIconExW;
use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

use crate::application::process_icons::{ProcessIconProvider, ICON_PNG_MAX_BYTES};
use crate::domain::metadata::FieldAvailability;
use crate::domain::process::ProcessInfo;

const MAX_SIDE: u32 = 128;

#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsProcessIconProvider;

impl ProcessIconProvider for WindowsProcessIconProvider {
    fn icon_png(&self, process: &ProcessInfo) -> Option<Vec<u8>> {
        let FieldAvailability::Available(path) = &process.identity.executable_path else {
            return None;
        };
        extract_icon_png(path)
    }
}

fn extract_icon_png(path: &Path) -> Option<Vec<u8>> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    if wide.is_empty() || wide.contains(&0) {
        return None;
    }
    let mut terminated = wide;
    terminated.push(0);
    let mut icon = ptr::null_mut();
    // SAFETY: `terminated` is NUL-terminated UTF-16 and remains alive for the
    // call. We request one large icon; ownership is accepted only when the
    // documented return count confirms one icon was extracted.
    let count = unsafe { ExtractIconExW(terminated.as_ptr(), 0, &mut icon, ptr::null_mut(), 1) };
    // ExtractIconExW documents UINT_MAX as its error sentinel. Only a return
    // count of one establishes that the output is a valid icon we own; do not
    // wrap or destroy an output value on zero, error, or unexpected counts.
    if !one_icon_extracted(count) || icon.is_null() {
        return None;
    }
    let icon = OwnedIcon(icon);
    icon_to_png(icon.0)
}

fn one_icon_extracted(count: u32) -> bool {
    count == 1
}

fn icon_to_png(icon: HICON) -> Option<Vec<u8>> {
    let mut info = ICONINFO::default();
    // SAFETY: `info` is writable, and `icon` is an owned HICON live for the
    // entire conversion. Successful GetIconInfo creates caller-owned bitmaps.
    if unsafe { GetIconInfo(icon, &mut info) } == 0 {
        return None;
    }
    let bitmaps = OwnedIconBitmaps {
        color: info.hbmColor,
        mask: info.hbmMask,
    };
    if bitmaps.color.is_null() || bitmaps.mask.is_null() {
        return None;
    }

    let (width, height) = bitmap_dimensions(bitmaps.color)?;
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        return None;
    }
    if bitmap_dimensions(bitmaps.mask)? != (width, height) {
        return None;
    }
    let dc = OwnedScreenDc::new()?;
    let bgra = read_bitmap_32(dc.0, bitmaps.color, width, height)?;
    let mask = read_mask_1(dc.0, bitmaps.mask, width, height)?;
    let mut rgba = Vec::with_capacity(bgra.len());
    let bgra_pixels = bgra.as_chunks::<4>().0;
    let has_alpha = bgra_pixels.iter().any(|pixel| pixel[3] != 0);
    let mask_stride = mask_stride(width)?;
    for (index, pixel) in bgra_pixels.iter().enumerate() {
        let x = index % width as usize;
        let y = index / width as usize;
        let transparent = (mask[y * mask_stride + (x / 8)] & (0x80 >> (x % 8))) != 0;
        rgba.extend_from_slice(&[
            pixel[2],
            pixel[1],
            pixel[0],
            if has_alpha {
                pixel[3]
            } else if transparent {
                0
            } else {
                255
            },
        ]);
    }

    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&rgba).ok()?;
    }
    (encoded.len() <= ICON_PNG_MAX_BYTES).then_some(encoded)
}

fn bitmap_dimensions(bitmap: HBITMAP) -> Option<(u32, u32)> {
    let mut native = MaybeUninit::<BITMAP>::zeroed();
    // SAFETY: GetObjectW writes at most one BITMAP into valid storage; the
    // object is an HBITMAP returned by GetIconInfo and remains owned here.
    let size = unsafe {
        GetObjectW(
            bitmap,
            std::mem::size_of::<BITMAP>() as i32,
            native.as_mut_ptr().cast::<c_void>(),
        )
    };
    if size != std::mem::size_of::<BITMAP>() as i32 {
        return None;
    }
    // SAFETY: GetObjectW reported that it initialized a full BITMAP.
    let bitmap = unsafe { native.assume_init() };
    Some((
        u32::try_from(bitmap.bmWidth).ok()?,
        u32::try_from(bitmap.bmHeight).ok()?,
    ))
}

fn read_bitmap_32(dc: HDC, bitmap: HBITMAP, width: u32, height: u32) -> Option<Vec<u8>> {
    let length = usize::try_from(width.checked_mul(height)?.checked_mul(4)?).ok()?;
    let mut pixels = vec![0u8; length];
    let mut info = bitmap_info(width, height, 32, 0);
    // SAFETY: `pixels` is sized for width*height*4 bytes. `bitmap` is not
    // selected into a DC; the top-down 32-bit BI_RGB header matches storage.
    let rows = unsafe {
        GetDIBits(
            dc,
            bitmap,
            0,
            height,
            pixels.as_mut_ptr().cast::<c_void>(),
            &mut info,
            DIB_RGB_COLORS,
        )
    };
    (rows == height as i32).then_some(pixels)
}

fn read_mask_1(dc: HDC, bitmap: HBITMAP, width: u32, height: u32) -> Option<Vec<u8>> {
    let stride = mask_stride(width)?;
    let length = stride.checked_mul(height as usize)?;
    let mut pixels = vec![0u8; length];
    let mut info = BitmapInfoTwoColors {
        header: bitmap_info(width, height, 1, 2).bmiHeader,
        colors: [RGBQUAD::default(); 2],
    };
    // The one-bit AND mask uses a two-entry black/white palette.
    info.colors[0] = RGBQUAD {
        rgbBlue: 0,
        rgbGreen: 0,
        rgbRed: 0,
        rgbReserved: 0,
    };
    info.colors[1] = RGBQUAD {
        rgbBlue: 255,
        rgbGreen: 255,
        rgbRed: 255,
        rgbReserved: 0,
    };
    // SAFETY: `pixels` uses the DWORD-aligned one-bit row stride and `info`
    // reserves the two palette entries required by a 1bpp DIB.
    let rows = unsafe {
        GetDIBits(
            dc,
            bitmap,
            0,
            height,
            pixels.as_mut_ptr().cast::<c_void>(),
            (&mut info as *mut BitmapInfoTwoColors).cast::<BITMAPINFO>(),
            DIB_RGB_COLORS,
        )
    };
    (rows == height as i32).then_some(pixels)
}

#[repr(C)]
struct BitmapInfoTwoColors {
    header: BITMAPINFOHEADER,
    colors: [RGBQUAD; 2],
}

fn bitmap_info(width: u32, height: u32, bit_count: u16, colors: u32) -> BITMAPINFO {
    BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: bit_count,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: colors,
            biClrImportant: colors,
        },
        bmiColors: [RGBQUAD::default()],
    }
}

fn mask_stride(width: u32) -> Option<usize> {
    usize::try_from(width.checked_add(31)? / 32)
        .ok()?
        .checked_mul(4)
}

struct OwnedIcon(HICON);

impl Drop for OwnedIcon {
    fn drop(&mut self) {
        // SAFETY: ExtractIconExW returned this exclusively owned HICON; it is
        // destroyed once after conversion or any early return.
        unsafe { DestroyIcon(self.0) };
    }
}

struct OwnedIconBitmaps {
    color: HBITMAP,
    mask: HBITMAP,
}

impl Drop for OwnedIconBitmaps {
    fn drop(&mut self) {
        // SAFETY: GetIconInfo created caller-owned bitmap handles. Delete each
        // non-null bitmap once; neither handle escapes this conversion.
        unsafe {
            if !self.color.is_null() {
                DeleteObject(self.color);
            }
            if !self.mask.is_null() {
                DeleteObject(self.mask);
            }
        }
    }
}

struct OwnedScreenDc(HDC);

impl OwnedScreenDc {
    fn new() -> Option<Self> {
        // SAFETY: a null HWND requests the screen DC, which is released with
        // ReleaseDC by Drop on all paths.
        let dc = unsafe { GetDC(ptr::null_mut()) };
        (!dc.is_null()).then_some(Self(dc))
    }
}

impl Drop for OwnedScreenDc {
    fn drop(&mut self) {
        // SAFETY: this DC came from GetDC(NULL) and remains exclusively owned.
        unsafe { ReleaseDC(ptr::null_mut(), self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::time::{Duration, UNIX_EPOCH};

    fn process_with_path(path: std::path::PathBuf) -> ProcessInfo {
        ProcessInfo {
            identity: crate::domain::process::ProcessIdentity {
                pid: crate::domain::process::ProcessId::new(1),
                name: FieldAvailability::Available(OsString::from("fixture")),
                executable_path: FieldAvailability::Available(path),
                start_time: FieldAvailability::Available(UNIX_EPOCH + Duration::from_secs(1)),
            },
            command_arguments: FieldAvailability::Unavailable(
                crate::domain::metadata::UnavailableReason::ProviderLimitation,
            ),
            working_directory: FieldAvailability::Unavailable(
                crate::domain::metadata::UnavailableReason::ProviderLimitation,
            ),
            resource_sample: Default::default(),
        }
    }

    #[test]
    fn extracts_a_bounded_icon_from_the_windows_system_icon_resource() {
        let windows = std::env::var_os("WINDIR").expect("Windows directory");
        let path = Path::new(&windows).join("System32").join("shell32.dll");
        let bytes = extract_icon_png(&path).expect("shell32 has an icon resource");
        assert!(crate::application::process_icons::is_bounded_png(&bytes));
    }

    #[test]
    fn missing_executable_metadata_uses_the_fallback_path() {
        let mut process = process_with_path(Path::new("unused.exe").to_path_buf());
        process.identity.executable_path = FieldAvailability::Unavailable(
            crate::domain::metadata::UnavailableReason::ProviderLimitation,
        );
        assert!(WindowsProcessIconProvider.icon_png(&process).is_none());
    }

    #[test]
    fn only_one_extracted_icon_is_owned() {
        assert!(one_icon_extracted(1));
        assert!(!one_icon_extracted(0));
        assert!(!one_icon_extracted(u32::MAX));
        assert!(!one_icon_extracted(2));
    }
}
