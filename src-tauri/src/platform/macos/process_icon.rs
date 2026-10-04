//! Optional public AppKit process-application icon lookup.

use std::ptr;

use crate::application::process_icons::{ProcessIconProvider, ICON_PNG_MAX_BYTES};
use crate::domain::process::ProcessInfo;

unsafe extern "C" {
    fn thaa_macos_process_icon_png(
        pid: i32,
        output: *mut *mut u8,
        output_length: *mut usize,
    ) -> i32;
    fn thaa_macos_process_icon_free(bytes: *mut u8);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MacOSProcessIconProvider;

impl ProcessIconProvider for MacOSProcessIconProvider {
    fn icon_png(&self, process: &ProcessInfo) -> Option<Vec<u8>> {
        let pid = i32::try_from(process.identity.pid.get()).ok()?;
        if pid <= 0 {
            return None;
        }

        let mut output = ptr::null_mut();
        let mut output_length = 0usize;
        // SAFETY: the shim validates PID and out pointers, writes only a
        // malloc-owned buffer of at most ICON_PNG_MAX_BYTES, and its paired
        // free function releases that allocation. AppKit values never escape.
        let result = unsafe { thaa_macos_process_icon_png(pid, &mut output, &mut output_length) };
        if result != 1
            || output.is_null()
            || output_length == 0
            || output_length > ICON_PNG_MAX_BYTES
        {
            if !output.is_null() {
                // SAFETY: a non-null pointer returned by the shim is owned by
                // this caller even when the returned length fails validation.
                unsafe { thaa_macos_process_icon_free(output) };
            }
            return None;
        }

        // SAFETY: successful shim output is a readable allocation of exactly
        // output_length bytes; copy it before releasing the native allocation.
        let bytes = unsafe { std::slice::from_raw_parts(output.cast_const(), output_length) };
        let copy = bytes.to_vec();
        // SAFETY: this allocation came from the paired shim allocation above.
        unsafe { thaa_macos_process_icon_free(output) };
        Some(copy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::metadata::FieldAvailability;
    use std::ffi::OsString;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn out_of_range_pid_uses_fallback_without_error() {
        let process = ProcessInfo {
            identity: crate::domain::process::ProcessIdentity {
                pid: crate::domain::process::ProcessId::new(u32::MAX),
                name: FieldAvailability::Available(OsString::from("fixture")),
                executable_path: FieldAvailability::Unavailable(
                    crate::domain::metadata::UnavailableReason::ProviderLimitation,
                ),
                start_time: FieldAvailability::Available(UNIX_EPOCH + Duration::from_secs(1)),
            },
            command_arguments: FieldAvailability::Unavailable(
                crate::domain::metadata::UnavailableReason::ProviderLimitation,
            ),
            working_directory: FieldAvailability::Unavailable(
                crate::domain::metadata::UnavailableReason::ProviderLimitation,
            ),
        };
        assert!(MacOSProcessIconProvider.icon_png(&process).is_none());
    }
}
