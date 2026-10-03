//! Read-only process metadata inspection through documented Win32 APIs.

use std::ffi::OsString;
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_INSUFFICIENT_BUFFER,
    ERROR_INVALID_PARAMETER, ERROR_NOT_SUPPORTED, FILETIME, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, WaitForSingleObject,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
};

use crate::domain::metadata::{FieldAvailability, UnavailableReason};
use crate::domain::process::{ProcessId, ProcessIdentity, ProcessInfo};
use crate::domain::process_provider::{
    ProcessProvider, ProcessProviderError, ProcessProviderErrorKind,
};

const INITIAL_IMAGE_PATH_UNITS: usize = 260;
// Bound allocation to the documented approximate maximum for extended-length
// Win32 paths, including room for the terminating UTF-16 NUL.
const MAX_IMAGE_PATH_UNITS: usize = 32_768;
const ZERO_TIMEOUT: u32 = 0;
const FILETIME_UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;

/// Windows implementation of the shared read-only process inspection contract.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsProcessProvider;

impl ProcessProvider for WindowsProcessProvider {
    fn inspect(&self, process_id: ProcessId) -> Result<ProcessInfo, ProcessProviderError> {
        if process_id.get() == 0 {
            // Microsoft documents PID 0 as the System Idle Process, which
            // OpenProcess deliberately refuses as an inspectable process.
            return Err(provider_error(ProcessProviderErrorKind::Unsupported));
        }

        let handle = open_process(process_id)?;
        ensure_running(&handle)?;

        let start_time = query_start_time(&handle)?;
        let executable_path = query_executable_path(&handle)?;
        let name = process_name(&executable_path);

        // The handle remains anchored to the same process object even if its
        // numeric PID is later reused. A signaled object means this snapshot
        // cannot be returned as a currently inspectable process.
        ensure_running(&handle)?;

        Ok(ProcessInfo {
            identity: ProcessIdentity {
                pid: process_id,
                name,
                executable_path,
                start_time,
            },
            // Windows exposes a command-line string, not the target program's
            // authoritative argv boundaries. Do not reconstruct structured
            // arguments from that string.
            command_arguments: FieldAvailability::Unavailable(
                UnavailableReason::ProviderLimitation,
            ),
            // No supported public Win32 API queries an arbitrary process's
            // current directory. It is not inferred from image or argv data.
            working_directory: FieldAvailability::Unavailable(
                UnavailableReason::ProviderLimitation,
            ),
        })
    }
}

/// Owns one process HANDLE. It is intentionally neither Clone nor Copy.
struct OwnedProcessHandle(NonNull<std::ffi::c_void>);

impl std::fmt::Debug for OwnedProcessHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("OwnedProcessHandle(..)")
    }
}

impl Drop for OwnedProcessHandle {
    fn drop(&mut self) {
        // SAFETY: the handle was returned as non-null by OpenProcess and is
        // owned exclusively by this wrapper. Drop runs once; no code copies,
        // closes, or lets the handle escape before this call.
        unsafe {
            CloseHandle(self.0.as_ptr());
        }
    }
}

impl OwnedProcessHandle {
    fn as_raw(&self) -> windows_sys::Win32::Foundation::HANDLE {
        self.0.as_ptr()
    }
}

fn open_process(process_id: ProcessId) -> Result<OwnedProcessHandle, ProcessProviderError> {
    // Query limited information is sufficient for image path and creation
    // time. SYNCHRONIZE is the only additional right and enables an exact
    // process-object signaled check without the STILL_ACTIVE exit-code
    // ambiguity. No VM, debug, or termination rights are requested.
    // SAFETY: OpenProcess receives a valid numeric process ID, a fixed
    // provider-controlled access mask, and FALSE to prevent handle inheritance.
    let raw_handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            process_id.get(),
        )
    };

    NonNull::new(raw_handle)
        .map(OwnedProcessHandle)
        .ok_or_else(|| map_open_error(process_id, unsafe { GetLastError() }))
}

fn ensure_running(handle: &OwnedProcessHandle) -> Result<(), ProcessProviderError> {
    // SAFETY: the owned handle is valid for this call and includes
    // PROCESS_SYNCHRONIZE. A zero timeout makes this a nonblocking state query.
    match unsafe { WaitForSingleObject(handle.as_raw(), ZERO_TIMEOUT) } {
        WAIT_TIMEOUT => Ok(()),
        WAIT_OBJECT_0 => Err(provider_error(ProcessProviderErrorKind::ProcessDisappeared)),
        _ => Err(provider_error(
            ProcessProviderErrorKind::OperatingSystemFailure,
        )),
    }
}

fn query_executable_path(
    handle: &OwnedProcessHandle,
) -> Result<FieldAvailability<PathBuf>, ProcessProviderError> {
    let mut capacity = INITIAL_IMAGE_PATH_UNITS;

    loop {
        let mut buffer = Vec::<u16>::new();
        buffer
            .try_reserve_exact(capacity)
            .map_err(|_| provider_error(ProcessProviderErrorKind::ProviderFailure))?;
        buffer.resize(capacity, 0);
        let mut written = u32::try_from(capacity)
            .map_err(|_| provider_error(ProcessProviderErrorKind::ProviderFailure))?;

        // SAFETY: the owned process handle remains valid during this call;
        // `buffer` is writable, initialized, and `written` describes exactly
        // its allocated UTF-16 capacity. Successful output length is checked
        // before slicing and converting it.
        let succeeded = unsafe {
            QueryFullProcessImageNameW(handle.as_raw(), 0, buffer.as_mut_ptr(), &mut written)
        };

        if succeeded != 0 {
            let written = usize::try_from(written)
                .map_err(|_| provider_error(ProcessProviderErrorKind::ParseFailure))?;
            if written == 0 || written >= buffer.len() || buffer[..written].contains(&0) {
                return Err(provider_error(ProcessProviderErrorKind::ParseFailure));
            }
            let path = PathBuf::from(OsString::from_wide(&buffer[..written]));
            return Ok(FieldAvailability::Available(path));
        }

        let error = unsafe { GetLastError() };
        if error != ERROR_INSUFFICIENT_BUFFER {
            return Ok(FieldAvailability::Unavailable(unavailable_reason(error)));
        }

        let Some(next_capacity) = next_image_path_capacity(capacity) else {
            return Ok(FieldAvailability::Unavailable(
                UnavailableReason::ProviderLimitation,
            ));
        };
        capacity = next_capacity;
    }
}

fn query_start_time(
    handle: &OwnedProcessHandle,
) -> Result<FieldAvailability<SystemTime>, ProcessProviderError> {
    let mut creation_time = MaybeUninit::<FILETIME>::uninit();
    let mut exit_time = MaybeUninit::<FILETIME>::uninit();
    let mut kernel_time = MaybeUninit::<FILETIME>::uninit();
    let mut user_time = MaybeUninit::<FILETIME>::uninit();

    // SAFETY: all out-pointers refer to valid, writable FILETIME storage and
    // the handle has the PROCESS_QUERY_LIMITED_INFORMATION right. Only the
    // creation-time output is read after the API reports success.
    let succeeded = unsafe {
        GetProcessTimes(
            handle.as_raw(),
            creation_time.as_mut_ptr(),
            exit_time.as_mut_ptr(),
            kernel_time.as_mut_ptr(),
            user_time.as_mut_ptr(),
        )
    };

    if succeeded == 0 {
        return Ok(FieldAvailability::Unavailable(unavailable_reason(unsafe {
            GetLastError()
        })));
    }

    // SAFETY: successful GetProcessTimes initializes all four output values.
    let creation_time = unsafe { creation_time.assume_init() };
    let ticks =
        (u64::from(creation_time.dwHighDateTime) << 32) | u64::from(creation_time.dwLowDateTime);
    let system_time = filetime_to_system_time(ticks)
        .ok_or_else(|| provider_error(ProcessProviderErrorKind::ParseFailure))?;

    Ok(FieldAvailability::Available(system_time))
}

fn process_name(executable_path: &FieldAvailability<PathBuf>) -> FieldAvailability<OsString> {
    match executable_path {
        FieldAvailability::Available(path) => path
            .file_stem()
            .map(|name| FieldAvailability::Available(name.to_os_string()))
            .unwrap_or(FieldAvailability::Unavailable(
                UnavailableReason::ProviderLimitation,
            )),
        FieldAvailability::Unavailable(reason) => FieldAvailability::Unavailable(*reason),
    }
}

fn next_image_path_capacity(current: usize) -> Option<usize> {
    (current < MAX_IMAGE_PATH_UNITS).then(|| {
        current
            .checked_mul(2)
            .unwrap_or(MAX_IMAGE_PATH_UNITS)
            .min(MAX_IMAGE_PATH_UNITS)
    })
}

fn filetime_to_system_time(ticks: u64) -> Option<SystemTime> {
    if ticks >= FILETIME_UNIX_EPOCH_TICKS {
        let nanos = ticks
            .checked_sub(FILETIME_UNIX_EPOCH_TICKS)?
            .checked_mul(100)?;
        UNIX_EPOCH.checked_add(Duration::from_nanos(nanos))
    } else {
        let nanos = FILETIME_UNIX_EPOCH_TICKS
            .checked_sub(ticks)?
            .checked_mul(100)?;
        UNIX_EPOCH.checked_sub(Duration::from_nanos(nanos))
    }
}

fn map_open_error(process_id: ProcessId, error: u32) -> ProcessProviderError {
    let kind = match error {
        ERROR_ACCESS_DENIED => ProcessProviderErrorKind::PermissionDenied,
        ERROR_INVALID_PARAMETER if process_id.get() == 0 => ProcessProviderErrorKind::Unsupported,
        ERROR_INVALID_PARAMETER => ProcessProviderErrorKind::ProcessDisappeared,
        _ => ProcessProviderErrorKind::OperatingSystemFailure,
    };
    provider_error(kind)
}

fn unavailable_reason(error: u32) -> UnavailableReason {
    match error {
        ERROR_ACCESS_DENIED => UnavailableReason::PermissionDenied,
        ERROR_NOT_SUPPORTED => UnavailableReason::Unsupported,
        _ => UnavailableReason::Inaccessible,
    }
}

fn provider_error(kind: ProcessProviderErrorKind) -> ProcessProviderError {
    ProcessProviderError::new(kind)
}

#[cfg(test)]
mod tests {
    use super::{
        filetime_to_system_time, map_open_error, next_image_path_capacity, process_name,
        MAX_IMAGE_PATH_UNITS,
    };
    use crate::domain::metadata::{FieldAvailability, UnavailableReason};
    use crate::domain::process::ProcessId;
    use crate::domain::process_provider::ProcessProviderErrorKind;
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::time::UNIX_EPOCH;
    use windows_sys::Win32::Foundation::{
        ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, ERROR_NOT_SUPPORTED,
    };

    #[test]
    fn process_name_is_the_file_stem_of_the_verified_image_path() {
        assert_eq!(
            process_name(&FieldAvailability::Available(PathBuf::from(
                r"C:\Program Files\Sample\runtime.exe"
            ))),
            FieldAvailability::Available(OsString::from("runtime"))
        );
    }

    #[test]
    fn unavailable_image_path_does_not_fabricate_a_process_name() {
        assert_eq!(
            process_name(&FieldAvailability::Unavailable(
                UnavailableReason::PermissionDenied
            )),
            FieldAvailability::Unavailable(UnavailableReason::PermissionDenied)
        );
    }

    #[test]
    fn filetime_conversion_preserves_epoch_and_subsecond_precision() {
        assert_eq!(
            filetime_to_system_time(116_444_736_000_000_000),
            Some(UNIX_EPOCH)
        );
        assert_eq!(
            filetime_to_system_time(116_444_736_012_345_678),
            UNIX_EPOCH.checked_add(std::time::Duration::from_nanos(1_234_567_800))
        );
    }

    #[test]
    fn filetime_conversion_supports_pre_epoch_and_rejects_overflow() {
        assert_eq!(
            filetime_to_system_time(116_444_735_990_000_000),
            UNIX_EPOCH.checked_sub(std::time::Duration::from_secs(1))
        );
        assert_eq!(filetime_to_system_time(u64::MAX), None);
    }

    #[test]
    fn open_process_error_mapping_accounts_for_idle_pid_context() {
        assert_eq!(
            map_open_error(ProcessId::new(0), ERROR_INVALID_PARAMETER).kind(),
            ProcessProviderErrorKind::Unsupported
        );
        assert_eq!(
            map_open_error(ProcessId::new(123), ERROR_INVALID_PARAMETER).kind(),
            ProcessProviderErrorKind::ProcessDisappeared
        );
        assert_eq!(
            map_open_error(ProcessId::new(123), ERROR_ACCESS_DENIED).kind(),
            ProcessProviderErrorKind::PermissionDenied
        );
    }

    #[test]
    fn field_errors_remain_distinct_from_operation_failures() {
        use super::unavailable_reason;

        assert_eq!(
            unavailable_reason(ERROR_ACCESS_DENIED),
            UnavailableReason::PermissionDenied
        );
        assert_eq!(
            unavailable_reason(ERROR_NOT_SUPPORTED),
            UnavailableReason::Unsupported
        );
        assert_eq!(unavailable_reason(1234), UnavailableReason::Inaccessible);
    }

    #[test]
    fn image_path_buffer_growth_is_monotonic_and_bounded() {
        assert_eq!(next_image_path_capacity(260), Some(520));
        assert_eq!(next_image_path_capacity(16_640), Some(MAX_IMAGE_PATH_UNITS));
        assert_eq!(next_image_path_capacity(MAX_IMAGE_PATH_UNITS), None);
    }

    #[test]
    fn idle_process_id_is_reported_as_unsupported() {
        use super::WindowsProcessProvider;
        use crate::domain::process_provider::ProcessProvider;

        let error = WindowsProcessProvider
            .inspect(ProcessId::new(0))
            .expect_err("System Idle Process is not queryable through OpenProcess");
        assert_eq!(error.kind(), ProcessProviderErrorKind::Unsupported);
    }
}
