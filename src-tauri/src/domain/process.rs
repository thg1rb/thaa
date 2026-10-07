//! Process identity and P0 metadata values shared across operating systems.

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use super::metadata::FieldAvailability;

/// A process identifier distinct from unrelated integer values.
///
/// The wrapper applies no sentinel or range rule beyond the OS-normalized
/// unsigned value. Absence is represented with `Option<ProcessId>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(u32);

impl ProcessId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Observed identity signals for a process.
///
/// Structural equality is useful for value handling and tests; it is not an
/// authorization decision for a destructive process action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessIdentity {
    pub pid: ProcessId,
    pub name: FieldAvailability<OsString>,
    pub executable_path: FieldAvailability<PathBuf>,
    pub start_time: FieldAvailability<SystemTime>,
}

/// P0 process data associated with the identity observed by a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub identity: ProcessIdentity,
    /// OS-normalized argument elements; never a shell-escaped command string.
    pub command_arguments: FieldAvailability<Vec<OsString>>,
    pub working_directory: FieldAvailability<PathBuf>,
    /// Cumulative CPU time and resident memory observed for this process.
    /// These values are snapshot metadata, never identity or action evidence.
    pub resource_sample: ProcessResourceSample,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessResourceSample {
    /// Cumulative user + kernel CPU time since process start.
    pub cumulative_cpu_time: FieldAvailability<Duration>,
    /// Resident set / working set in bytes.
    pub resident_memory_bytes: FieldAvailability<u64>,
    /// Monotonic counter observation time, used only for CPU deltas.
    pub sampled_at: Option<Instant>,
}

/// Resource values derived for one runtime snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProcessResourceMetrics {
    /// Hundredths of one percent of total logical CPU capacity (0..=10_000).
    pub cpu_percent_hundredths: Option<u16>,
    pub resident_memory_bytes: Option<u64>,
    pub uptime: Option<Duration>,
}

impl Default for ProcessResourceSample {
    fn default() -> Self {
        Self {
            cumulative_cpu_time: FieldAvailability::Unavailable(
                super::metadata::UnavailableReason::ProviderLimitation,
            ),
            resident_memory_bytes: FieldAvailability::Unavailable(
                super::metadata::UnavailableReason::ProviderLimitation,
            ),
            sampled_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ProcessId, ProcessIdentity, ProcessInfo, ProcessResourceSample};
    use crate::domain::metadata::{FieldAvailability, UnavailableReason};
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn process_id_preserves_the_full_unsigned_range() {
        assert_eq!(ProcessId::new(u32::MAX).get(), u32::MAX);
        assert_eq!(ProcessId::new(0).get(), 0);
    }

    #[test]
    fn process_metadata_keeps_arguments_structured_and_paths_native() {
        let info = ProcessInfo {
            identity: ProcessIdentity {
                pid: ProcessId::new(42),
                name: FieldAvailability::Available(OsString::from("Sample Process")),
                executable_path: FieldAvailability::Available(PathBuf::from(
                    "Sample Projects/app runner",
                )),
                start_time: FieldAvailability::Available(SystemTime::UNIX_EPOCH),
            },
            command_arguments: FieldAvailability::Available(vec![
                OsString::from("--name"),
                OsString::from("a value with spaces"),
            ]),
            working_directory: FieldAvailability::Unavailable(UnavailableReason::PermissionDenied),
            resource_sample: ProcessResourceSample::default(),
        };

        assert_eq!(
            info.command_arguments,
            FieldAvailability::Available(vec![
                OsString::from("--name"),
                OsString::from("a value with spaces")
            ])
        );
        assert_eq!(
            info.identity.executable_path,
            FieldAvailability::Available(PathBuf::from("Sample Projects/app runner"))
        );
        assert_eq!(
            info.working_directory,
            FieldAvailability::Unavailable(UnavailableReason::PermissionDenied)
        );
    }

    #[cfg(unix)]
    #[test]
    fn filesystem_paths_can_retain_non_utf8_components() {
        use std::os::unix::ffi::OsStringExt;

        let native_component = OsString::from_vec(vec![0xff]);
        let path = PathBuf::from(native_component.clone());
        let identity = ProcessIdentity {
            pid: ProcessId::new(7),
            name: FieldAvailability::Unavailable(UnavailableReason::Unsupported),
            executable_path: FieldAvailability::Available(path.clone()),
            start_time: FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation),
        };

        assert_eq!(identity.executable_path, FieldAvailability::Available(path));
        assert!(matches!(
            &identity.executable_path,
            FieldAvailability::Available(path) if path.as_os_str().to_str().is_none()
        ));
    }
}
