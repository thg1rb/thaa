//! Read-only process metadata inspection through documented Win32 APIs.

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Instant;

use windows_sys::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER};
use windows_sys::Win32::System::Threading::{
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
};

use crate::domain::metadata::{FieldAvailability, UnavailableReason};
use crate::domain::process::{ProcessId, ProcessIdentity, ProcessInfo, ProcessResourceSample};
use crate::domain::process_provider::{
    ProcessProvider, ProcessProviderError, ProcessProviderErrorKind,
};

use super::process_native::{
    is_running, open_process, query_cumulative_cpu_time, query_executable_path,
    query_resident_memory_bytes, query_start_time, NativeDataError,
};

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

        let handle = open_process(
            process_id.get(),
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
        )
        .map_err(|error| map_open_error(process_id, error))?;
        ensure_running(&handle)?;

        let start_time = query_start_time(&handle).map_err(map_native_data_error)?;
        let executable_path = query_executable_path(&handle).map_err(map_native_data_error)?;
        let resource_sample = ProcessResourceSample {
            cumulative_cpu_time: query_cumulative_cpu_time(&handle),
            resident_memory_bytes: query_resident_memory_bytes(&handle),
            sampled_at: Some(Instant::now()),
        };
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
            resource_sample,
        })
    }
}

fn ensure_running(
    handle: &super::process_native::OwnedProcessHandle,
) -> Result<(), ProcessProviderError> {
    match is_running(handle) {
        Ok(true) => Ok(()),
        Ok(false) => Err(provider_error(ProcessProviderErrorKind::ProcessDisappeared)),
        Err(_) => Err(provider_error(
            ProcessProviderErrorKind::OperatingSystemFailure,
        )),
    }
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

fn map_open_error(process_id: ProcessId, error: u32) -> ProcessProviderError {
    let kind = match error {
        ERROR_ACCESS_DENIED => ProcessProviderErrorKind::PermissionDenied,
        ERROR_INVALID_PARAMETER if process_id.get() == 0 => ProcessProviderErrorKind::Unsupported,
        ERROR_INVALID_PARAMETER => ProcessProviderErrorKind::ProcessDisappeared,
        _ => ProcessProviderErrorKind::OperatingSystemFailure,
    };
    provider_error(kind)
}

fn map_native_data_error(error: NativeDataError) -> ProcessProviderError {
    let kind = match error {
        NativeDataError::AllocationFailure => ProcessProviderErrorKind::ProviderFailure,
        NativeDataError::MalformedData => ProcessProviderErrorKind::ParseFailure,
    };
    provider_error(kind)
}

fn provider_error(kind: ProcessProviderErrorKind) -> ProcessProviderError {
    ProcessProviderError::new(kind)
}

#[cfg(test)]
mod tests {
    use super::{map_open_error, process_name};
    use crate::domain::metadata::{FieldAvailability, UnavailableReason};
    use crate::domain::process::ProcessId;
    use crate::domain::process_provider::ProcessProviderErrorKind;
    use std::ffi::OsString;
    use std::path::PathBuf;
    use windows_sys::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER};

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
    fn idle_process_id_is_reported_as_unsupported() {
        use super::WindowsProcessProvider;
        use crate::domain::process_provider::ProcessProvider;

        let error = WindowsProcessProvider
            .inspect(ProcessId::new(0))
            .expect_err("System Idle Process is not queryable through OpenProcess");
        assert_eq!(error.kind(), ProcessProviderErrorKind::Unsupported);
    }
}
