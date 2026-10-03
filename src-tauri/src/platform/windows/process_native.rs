//! Shared, Windows-only process HANDLE and metadata primitives.

use std::ffi::OsString;
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_INSUFFICIENT_BUFFER, ERROR_NOT_SUPPORTED,
    FILETIME, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, WaitForSingleObject,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::domain::metadata::{FieldAvailability, UnavailableReason};

pub(super) const INITIAL_IMAGE_PATH_UNITS: usize = 260;
// Bound allocation to the documented approximate maximum for extended-length
// Win32 paths, including room for the terminating UTF-16 NUL.
pub(super) const MAX_IMAGE_PATH_UNITS: usize = 32_768;
const ZERO_TIMEOUT: u32 = 0;
const FILETIME_UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;

/// Errors that are not ordinary per-field unavailability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NativeDataError {
    AllocationFailure,
    MalformedData,
}

/// Exclusively owns one process HANDLE. It is intentionally neither Clone nor Copy.
pub(super) struct OwnedProcessHandle(NonNull<std::ffi::c_void>);

impl std::fmt::Debug for OwnedProcessHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("OwnedProcessHandle(..)")
    }
}

impl Drop for OwnedProcessHandle {
    fn drop(&mut self) {
        // SAFETY: construction requires a non-null OpenProcess result. The
        // wrapper is not Clone/Copy and no other function closes the handle,
        // so Drop closes this owned HANDLE exactly once.
        unsafe {
            CloseHandle(self.0.as_ptr());
        }
    }
}

impl OwnedProcessHandle {
    pub(super) fn as_raw(&self) -> windows_sys::Win32::Foundation::HANDLE {
        self.0.as_ptr()
    }
}

/// Opens a process using exactly the caller-selected rights and no inheritance.
/// The raw error is kept inside the Windows adapter for API-context mapping.
pub(super) fn open_process(
    process_id: u32,
    desired_access: u32,
) -> Result<OwnedProcessHandle, u32> {
    // SAFETY: the API receives a numeric PID, fixed caller-controlled rights,
    // and FALSE to prevent inheritance. A successful HANDLE is exclusively
    // wrapped below; GetLastError is read immediately after a null result.
    let raw_handle = unsafe { OpenProcess(desired_access, 0, process_id) };

    match NonNull::new(raw_handle) {
        Some(handle) => Ok(OwnedProcessHandle(handle)),
        None => Err(unsafe { GetLastError() }),
    }
}

/// `Ok(true)` means active, `Ok(false)` means the process object is signaled.
pub(super) fn is_running(handle: &OwnedProcessHandle) -> Result<bool, u32> {
    // SAFETY: the owned HANDLE remains live for the call and callers open it
    // with PROCESS_SYNCHRONIZE. A zero timeout is nonblocking.
    match unsafe { WaitForSingleObject(handle.as_raw(), ZERO_TIMEOUT) } {
        WAIT_TIMEOUT => Ok(true),
        WAIT_OBJECT_0 => Ok(false),
        _ => Err(unsafe { GetLastError() }),
    }
}

pub(super) fn query_executable_path(
    handle: &OwnedProcessHandle,
) -> Result<FieldAvailability<PathBuf>, NativeDataError> {
    let mut capacity = INITIAL_IMAGE_PATH_UNITS;

    loop {
        let mut buffer = Vec::<u16>::new();
        buffer
            .try_reserve_exact(capacity)
            .map_err(|_| NativeDataError::AllocationFailure)?;
        buffer.resize(capacity, 0);
        let mut written = u32::try_from(capacity).map_err(|_| NativeDataError::MalformedData)?;

        // SAFETY: the owned process HANDLE remains live; `buffer` is writable,
        // initialized UTF-16 storage and `written` describes its capacity.
        // The successful output length is validated before conversion.
        let succeeded = unsafe {
            QueryFullProcessImageNameW(handle.as_raw(), 0, buffer.as_mut_ptr(), &mut written)
        };

        if succeeded != 0 {
            let written = usize::try_from(written).map_err(|_| NativeDataError::MalformedData)?;
            if written == 0 || written >= buffer.len() || buffer[..written].contains(&0) {
                return Err(NativeDataError::MalformedData);
            }
            return Ok(FieldAvailability::Available(PathBuf::from(
                OsString::from_wide(&buffer[..written]),
            )));
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

pub(super) fn query_start_time(
    handle: &OwnedProcessHandle,
) -> Result<FieldAvailability<SystemTime>, NativeDataError> {
    let mut creation_time = MaybeUninit::<FILETIME>::uninit();
    let mut exit_time = MaybeUninit::<FILETIME>::uninit();
    let mut kernel_time = MaybeUninit::<FILETIME>::uninit();
    let mut user_time = MaybeUninit::<FILETIME>::uninit();

    // SAFETY: each output pointer targets valid writable FILETIME storage and
    // the handle has PROCESS_QUERY_LIMITED_INFORMATION. Only initialized
    // creation-time storage is read after success.
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
    let system_time = filetime_to_system_time(ticks).ok_or(NativeDataError::MalformedData)?;

    Ok(FieldAvailability::Available(system_time))
}

pub(super) fn unavailable_reason(error: u32) -> UnavailableReason {
    match error {
        ERROR_ACCESS_DENIED => UnavailableReason::PermissionDenied,
        ERROR_NOT_SUPPORTED => UnavailableReason::Unsupported,
        _ => UnavailableReason::Inaccessible,
    }
}

pub(super) fn next_image_path_capacity(current: usize) -> Option<usize> {
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

#[cfg(test)]
mod tests {
    use super::{filetime_to_system_time, next_image_path_capacity, MAX_IMAGE_PATH_UNITS};
    use std::time::UNIX_EPOCH;

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
    fn image_path_buffer_growth_is_monotonic_and_bounded() {
        assert_eq!(next_image_path_capacity(260), Some(520));
        assert_eq!(next_image_path_capacity(16_640), Some(MAX_IMAGE_PATH_UNITS));
        assert_eq!(next_image_path_capacity(MAX_IMAGE_PATH_UNITS), None);
    }
}
