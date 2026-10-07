//! Operating-system adapters, selected only at the platform boundary.

use std::io;
use std::path::Path;

/// Reports whether both paths belong to the same mounted filesystem/volume.
/// Project-root traversal uses this guard before inspecting each ancestor.
pub(crate) fn same_filesystem(start: &Path, candidate: &Path) -> io::Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        Ok(std::fs::metadata(start)?.dev() == std::fs::metadata(candidate)?.dev())
    }

    #[cfg(windows)]
    {
        Ok(windows_volume_path(start)?.eq_ignore_ascii_case(&windows_volume_path(candidate)?))
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = (start, candidate);
        Ok(true)
    }
}

pub mod git_context;

#[cfg(windows)]
fn windows_volume_path(path: &Path) -> io::Result<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetVolumePathNameW;

    const VOLUME_PATH_BUFFER_SIZE: usize = 32_768;
    let path: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let mut buffer = vec![0_u16; VOLUME_PATH_BUFFER_SIZE];
    // SAFETY: both buffers are valid UTF-16 buffers for the duration of this
    // call, and the output buffer length is passed explicitly.
    let succeeded =
        unsafe { GetVolumePathNameW(path.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) };
    if succeeded == 0 {
        return Err(io::Error::last_os_error());
    }

    let length = buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(buffer.len());
    String::from_utf16(&buffer[..length])
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;
