//! Deterministic tests of the public ProcessProvider contract.

mod common;

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

use common::process_provider_contract::assert_inspects_requested_process;
use thaa_lib::domain::metadata::{FieldAvailability, UnavailableReason};
use thaa_lib::domain::process::{ProcessId, ProcessIdentity, ProcessInfo};
use thaa_lib::domain::process_provider::{
    ProcessProvider, ProcessProviderError, ProcessProviderErrorKind,
};

struct StubProcessProvider {
    response: Result<ProcessInfo, ProcessProviderError>,
    requested: Mutex<Vec<ProcessId>>,
}

impl StubProcessProvider {
    fn returning(response: Result<ProcessInfo, ProcessProviderError>) -> Self {
        Self {
            response,
            requested: Mutex::new(Vec::new()),
        }
    }
}

impl ProcessProvider for StubProcessProvider {
    fn inspect(&self, process_id: ProcessId) -> Result<ProcessInfo, ProcessProviderError> {
        self.requested
            .lock()
            .expect("test request mutex is not poisoned")
            .push(process_id);
        self.response.clone()
    }
}

fn full_info(process_id: ProcessId) -> ProcessInfo {
    ProcessInfo {
        identity: ProcessIdentity {
            pid: process_id,
            name: FieldAvailability::Available(OsString::from("Sample Process\nnot a command")),
            executable_path: FieldAvailability::Available(PathBuf::from(
                "Sample Applications/runner app",
            )),
            start_time: FieldAvailability::Available(SystemTime::UNIX_EPOCH),
        },
        command_arguments: FieldAvailability::Available(vec![
            OsString::from("runner app"),
            OsString::from("--label=a value; $(not executed)"),
            OsString::from("--quoted=\"text\""),
            OsString::from("--unicode=雪"),
        ]),
        working_directory: FieldAvailability::Available(PathBuf::from("Sample Projects/demo")),
    }
}

#[test]
fn success_preserves_requested_pid_and_full_metadata_through_trait_object() {
    let process_id = ProcessId::new(42);
    let expected = full_info(process_id);
    let provider = StubProcessProvider::returning(Ok(expected.clone()));
    let provider_object: &dyn ProcessProvider = &provider;

    let info = assert_inspects_requested_process(provider_object, process_id);

    assert_eq!(info, expected);
    assert_eq!(
        *provider
            .requested
            .lock()
            .expect("test request mutex is not poisoned"),
        vec![process_id]
    );
}

#[test]
fn unavailable_fields_remain_successful_and_reasons_remain_distinct() {
    let process_id = ProcessId::new(73);
    let mut info = full_info(process_id);
    info.identity.name = FieldAvailability::Unavailable(UnavailableReason::Unsupported);
    info.identity.start_time =
        FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation);
    info.command_arguments = FieldAvailability::Unavailable(UnavailableReason::Inaccessible);
    info.working_directory = FieldAvailability::Unavailable(UnavailableReason::PermissionDenied);
    let provider = StubProcessProvider::returning(Ok(info.clone()));

    let result = assert_inspects_requested_process(&provider, process_id);

    assert_eq!(result, info);
    assert_eq!(
        result.identity.name,
        FieldAvailability::Unavailable(UnavailableReason::Unsupported)
    );
    assert_eq!(
        result.identity.start_time,
        FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
    );
    assert_eq!(
        result.command_arguments,
        FieldAvailability::Unavailable(UnavailableReason::Inaccessible)
    );
    assert_eq!(
        result.working_directory,
        FieldAvailability::Unavailable(UnavailableReason::PermissionDenied)
    );
}

#[test]
fn process_disappearance_is_a_query_error_not_fabricated_metadata() {
    let process_id = ProcessId::new(999);
    let error = ProcessProviderError::new(ProcessProviderErrorKind::ProcessDisappeared);
    let provider = StubProcessProvider::returning(Err(error));

    let actual = provider
        .inspect(process_id)
        .expect_err("a disappeared process must remain a query failure");

    assert_eq!(actual.kind(), ProcessProviderErrorKind::ProcessDisappeared);
    assert_eq!(actual.to_string(), "process disappeared");
}

#[test]
fn whole_query_permission_denial_is_distinct_from_field_permission_denial() {
    let process_id = ProcessId::new(88);
    let whole_query = StubProcessProvider::returning(Err(ProcessProviderError::new(
        ProcessProviderErrorKind::PermissionDenied,
    )));
    let error = whole_query
        .inspect(process_id)
        .expect_err("whole-query denial is an operation error");

    let field_denial = {
        let mut info = full_info(process_id);
        info.working_directory =
            FieldAvailability::Unavailable(UnavailableReason::PermissionDenied);
        StubProcessProvider::returning(Ok(info))
    };
    let info = field_denial
        .inspect(process_id)
        .expect("field denial does not fail the whole inspection");

    assert_eq!(error.kind(), ProcessProviderErrorKind::PermissionDenied);
    assert_eq!(
        info.working_directory,
        FieldAvailability::Unavailable(UnavailableReason::PermissionDenied)
    );
}

#[test]
fn provider_failures_and_error_categories_are_stable_and_privacy_safe() {
    let categories = [
        ProcessProviderErrorKind::ProcessDisappeared,
        ProcessProviderErrorKind::PermissionDenied,
        ProcessProviderErrorKind::Unsupported,
        ProcessProviderErrorKind::MechanismUnavailable,
        ProcessProviderErrorKind::ParseFailure,
        ProcessProviderErrorKind::OperatingSystemFailure,
        ProcessProviderErrorKind::ProviderFailure,
    ];

    for category in categories {
        let error = ProcessProviderError::new(category);
        assert_eq!(error.kind(), category);
        assert_eq!(error.to_string(), category.to_string());
        assert!(!error.to_string().contains("/private/host/path"));
        assert!(!error.to_string().contains("secret-token"));
    }

    let provider = StubProcessProvider::returning(Err(ProcessProviderError::new(
        ProcessProviderErrorKind::ProviderFailure,
    )));
    assert_eq!(
        provider
            .inspect(ProcessId::new(12))
            .expect_err("provider failure remains a failure")
            .kind(),
        ProcessProviderErrorKind::ProviderFailure
    );
}

#[test]
fn native_paths_and_hostile_arguments_are_preserved_as_data() {
    let mut info = full_info(ProcessId::new(101));
    let native_executable = PathBuf::from(OsString::from("路径/runner\u{1b}[31m"));
    let native_working_directory = PathBuf::from(OsString::from("folder with spaces/子目录"));
    let hostile_arguments = vec![
        OsString::from("--value=$(touch never-run)"),
        OsString::from(";|&`$()"),
        OsString::from("line\nbreak"),
    ];
    info.identity.executable_path = FieldAvailability::Available(native_executable.clone());
    info.working_directory = FieldAvailability::Available(native_working_directory.clone());
    info.command_arguments = FieldAvailability::Available(hostile_arguments.clone());
    let provider = StubProcessProvider::returning(Ok(info));

    let result = provider
        .inspect(ProcessId::new(101))
        .expect("hostile-looking metadata is still inert data");

    assert_eq!(
        result.identity.executable_path,
        FieldAvailability::Available(native_executable)
    );
    assert_eq!(
        result.working_directory,
        FieldAvailability::Available(native_working_directory)
    );
    assert_eq!(
        result.command_arguments,
        FieldAvailability::Available(hostile_arguments)
    );
}
