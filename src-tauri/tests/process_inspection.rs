//! Deterministic tests of shared process-inspection orchestration.

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

use thaa_lib::application::process_inspection::{inspect_processes, ProcessInspectionOutcome};
use thaa_lib::domain::metadata::{FieldAvailability, UnavailableReason};
use thaa_lib::domain::process::{ProcessId, ProcessIdentity, ProcessInfo};
use thaa_lib::domain::process_provider::{
    ProcessProvider, ProcessProviderError, ProcessProviderErrorKind,
};

struct StubProcessProvider {
    responses: HashMap<u32, Result<ProcessInfo, ProcessProviderError>>,
    calls: Mutex<Vec<ProcessId>>,
}

impl StubProcessProvider {
    fn new(
        responses: impl IntoIterator<Item = (ProcessId, Result<ProcessInfo, ProcessProviderError>)>,
    ) -> Self {
        Self {
            responses: responses
                .into_iter()
                .map(|(process_id, result)| (process_id.get(), result))
                .collect(),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<ProcessId> {
        self.calls
            .lock()
            .expect("test call mutex is not poisoned")
            .clone()
    }
}

impl ProcessProvider for StubProcessProvider {
    fn inspect(&self, process_id: ProcessId) -> Result<ProcessInfo, ProcessProviderError> {
        self.calls
            .lock()
            .expect("test call mutex is not poisoned")
            .push(process_id);
        self.responses
            .get(&process_id.get())
            .cloned()
            .unwrap_or_else(|| {
                Err(ProcessProviderError::new(
                    ProcessProviderErrorKind::ProviderFailure,
                ))
            })
    }
}

fn info(process_id: ProcessId) -> ProcessInfo {
    ProcessInfo {
        identity: ProcessIdentity {
            pid: process_id,
            name: FieldAvailability::Available(OsString::from(format!(
                "process-{}",
                process_id.get()
            ))),
            executable_path: FieldAvailability::Available(PathBuf::from("Sample Apps/runner")),
            start_time: FieldAvailability::Available(SystemTime::UNIX_EPOCH),
        },
        command_arguments: FieldAvailability::Available(vec![OsString::from("runner")]),
        working_directory: FieldAvailability::Available(PathBuf::from("Sample Projects/demo")),
        resource_sample: Default::default(),
    }
}

fn outcome_ids(outcomes: &[ProcessInspectionOutcome]) -> Vec<ProcessId> {
    outcomes
        .iter()
        .map(|outcome| outcome.requested_process_id)
        .collect()
}

#[test]
fn inspects_multiple_processes_in_first_seen_order() {
    let first = ProcessId::new(10);
    let second = ProcessId::new(20);
    let provider = StubProcessProvider::new([(first, Ok(info(first))), (second, Ok(info(second)))]);

    let outcomes = inspect_processes(&provider, &[first, second]);

    assert_eq!(outcome_ids(&outcomes), vec![first, second]);
    assert_eq!(provider.calls(), vec![first, second]);
    assert!(outcomes.iter().all(|outcome| outcome.result.is_ok()));
}

#[test]
fn inspects_duplicate_process_ids_only_once_and_preserves_first_occurrence_order() {
    let first = ProcessId::new(11);
    let second = ProcessId::new(22);
    let provider = StubProcessProvider::new([(first, Ok(info(first))), (second, Ok(info(second)))]);

    let outcomes = inspect_processes(&provider, &[first, first, second, first, second]);

    assert_eq!(outcome_ids(&outcomes), vec![first, second]);
    assert_eq!(provider.calls(), vec![first, second]);
}

#[test]
fn per_process_errors_do_not_discard_other_successes_or_stop_inspection() {
    let first = ProcessId::new(31);
    let disappeared = ProcessId::new(32);
    let denied = ProcessId::new(33);
    let last = ProcessId::new(34);
    let provider = StubProcessProvider::new([
        (first, Ok(info(first))),
        (
            disappeared,
            Err(ProcessProviderError::new(
                ProcessProviderErrorKind::ProcessDisappeared,
            )),
        ),
        (
            denied,
            Err(ProcessProviderError::new(
                ProcessProviderErrorKind::PermissionDenied,
            )),
        ),
        (last, Ok(info(last))),
    ]);

    let outcomes = inspect_processes(&provider, &[first, disappeared, denied, last]);

    assert_eq!(
        outcome_ids(&outcomes),
        vec![first, disappeared, denied, last]
    );
    assert_eq!(provider.calls(), vec![first, disappeared, denied, last]);
    assert_eq!(outcomes[0].result, Ok(info(first)));
    assert_eq!(
        outcomes[1].result.as_ref().unwrap_err().kind(),
        ProcessProviderErrorKind::ProcessDisappeared
    );
    assert_eq!(
        outcomes[2].result.as_ref().unwrap_err().kind(),
        ProcessProviderErrorKind::PermissionDenied
    );
    assert_eq!(outcomes[3].result, Ok(info(last)));
}

#[test]
fn successful_partial_metadata_remains_successful() {
    let process_id = ProcessId::new(41);
    let mut partial = info(process_id);
    partial.working_directory = FieldAvailability::Unavailable(UnavailableReason::PermissionDenied);
    let provider = StubProcessProvider::new([(process_id, Ok(partial.clone()))]);

    let outcomes = inspect_processes(&provider, &[process_id]);

    assert_eq!(outcomes[0].result, Ok(partial));
}

#[test]
fn provider_failure_is_attached_to_its_pid_and_later_ids_are_still_inspected() {
    let failure = ProcessId::new(51);
    let succeeding = ProcessId::new(52);
    let provider = StubProcessProvider::new([
        (
            failure,
            Err(ProcessProviderError::new(
                ProcessProviderErrorKind::MechanismUnavailable,
            )),
        ),
        (succeeding, Ok(info(succeeding))),
    ]);

    let outcomes = inspect_processes(&provider, &[failure, succeeding]);

    assert_eq!(outcome_ids(&outcomes), vec![failure, succeeding]);
    assert_eq!(
        outcomes[0].result.as_ref().unwrap_err().kind(),
        ProcessProviderErrorKind::MechanismUnavailable
    );
    assert_eq!(outcomes[1].result, Ok(info(succeeding)));
    assert_eq!(provider.calls(), vec![failure, succeeding]);
}

#[test]
fn mismatched_provider_identity_becomes_provider_failure_for_requested_pid() {
    let requested = ProcessId::new(61);
    let returned = ProcessId::new(62);
    let provider = StubProcessProvider::new([(requested, Ok(info(returned)))]);

    let outcomes = inspect_processes(&provider, &[requested]);

    assert_eq!(outcomes[0].requested_process_id, requested);
    assert_eq!(
        outcomes[0].result.as_ref().unwrap_err().kind(),
        ProcessProviderErrorKind::ProviderFailure
    );
}

#[test]
fn empty_input_returns_no_results_and_makes_no_provider_calls() {
    let provider = StubProcessProvider::new([]);

    let outcomes = inspect_processes(&provider, &[]);

    assert!(outcomes.is_empty());
    assert!(provider.calls().is_empty());
}

#[test]
fn hostile_looking_process_arguments_remain_inert_structured_data() {
    let process_id = ProcessId::new(71);
    let mut process_info = info(process_id);
    let arguments = vec![
        OsString::from("--token=$(never-execute)"),
        OsString::from(";|&`$()"),
        OsString::from("雪"),
    ];
    process_info.command_arguments = FieldAvailability::Available(arguments.clone());
    let provider = StubProcessProvider::new([(process_id, Ok(process_info))]);

    let outcomes = inspect_processes(&provider, &[process_id]);

    assert_eq!(
        outcomes[0]
            .result
            .as_ref()
            .expect("inspection should succeed")
            .command_arguments,
        FieldAvailability::Available(arguments)
    );
}
