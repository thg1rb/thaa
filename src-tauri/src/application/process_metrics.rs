//! Snapshot-local CPU sampling and process uptime calculation.

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::domain::metadata::FieldAvailability;
use crate::domain::process::{
    ProcessId, ProcessInfo, ProcessResourceMetrics, ProcessResourceSample,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ProcessInstanceKey {
    pid: ProcessId,
    start_time_since_epoch_nanos: u128,
}

#[derive(Debug, Clone, Copy)]
struct CpuObservation {
    cpu_time: Duration,
    sampled_at: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct CpuSampler {
    previous: HashMap<ProcessInstanceKey, CpuObservation>,
}

impl CpuSampler {
    /// Samples only processes in this snapshot, dropping stale/PID-reused entries.
    pub fn sample(
        &mut self,
        processes: &[(ProcessId, ProcessInfo)],
        logical_cpu_count: Option<usize>,
        observed_at: SystemTime,
    ) -> HashMap<u32, ProcessResourceMetrics> {
        let mut next_previous = HashMap::with_capacity(processes.len());
        let mut metrics_by_pid = HashMap::with_capacity(processes.len());

        for (pid, process) in processes {
            let key = process_instance_key(process, observed_at);
            let cpu_time = available_cpu_time(&process.resource_sample);
            let sampled_at = process.resource_sample.sampled_at;
            let cpu_percent_hundredths =
                key.zip(cpu_time)
                    .zip(sampled_at)
                    .and_then(|((key, cpu_time), sampled_at)| {
                        let prior = self.previous.get(&key)?;
                        let elapsed = sampled_at.checked_duration_since(prior.sampled_at)?;
                        let delta = cpu_time.checked_sub(prior.cpu_time)?;
                        cpu_percent_hundredths(delta, elapsed, logical_cpu_count?)
                    });

            if let (Some(key), Some(cpu_time), Some(sampled_at)) = (key, cpu_time, sampled_at) {
                next_previous.insert(
                    key,
                    CpuObservation {
                        cpu_time,
                        sampled_at,
                    },
                );
            }

            let resident_memory_bytes = match &process.resource_sample.resident_memory_bytes {
                FieldAvailability::Available(bytes) => Some(*bytes),
                FieldAvailability::Unavailable(_) => None,
            };
            let uptime = match &process.identity.start_time {
                FieldAvailability::Available(start_time) => {
                    observed_at.duration_since(*start_time).ok()
                }
                FieldAvailability::Unavailable(_) => None,
            };

            metrics_by_pid.insert(
                pid.get(),
                ProcessResourceMetrics {
                    cpu_percent_hundredths,
                    resident_memory_bytes,
                    uptime,
                },
            );
        }

        self.previous = next_previous;
        metrics_by_pid
    }

    #[cfg(test)]
    fn previous_len(&self) -> usize {
        self.previous.len()
    }
}

fn process_instance_key(
    process: &ProcessInfo,
    observed_at: SystemTime,
) -> Option<ProcessInstanceKey> {
    let FieldAvailability::Available(start_time) = &process.identity.start_time else {
        return None;
    };
    let start_time_since_epoch_nanos = start_time.duration_since(UNIX_EPOCH).ok()?.as_nanos();
    observed_at.duration_since(*start_time).ok()?;
    Some(ProcessInstanceKey {
        pid: process.identity.pid,
        start_time_since_epoch_nanos,
    })
}

fn available_cpu_time(sample: &ProcessResourceSample) -> Option<Duration> {
    match &sample.cumulative_cpu_time {
        FieldAvailability::Available(cpu_time) => Some(*cpu_time),
        FieldAvailability::Unavailable(_) => None,
    }
}

fn cpu_percent_hundredths(
    cpu_delta: Duration,
    elapsed: Duration,
    logical_cpu_count: usize,
) -> Option<u16> {
    let elapsed_nanos = elapsed.as_nanos();
    let cpu_count = u128::try_from(logical_cpu_count).ok()?;
    if elapsed_nanos == 0 || cpu_count == 0 {
        return None;
    }
    let normalized = cpu_delta
        .as_nanos()
        .checked_mul(10_000)?
        .checked_div(elapsed_nanos)?
        .checked_div(cpu_count)?
        .min(10_000);
    u16::try_from(normalized).ok()
}

#[cfg(test)]
mod tests {
    use super::{cpu_percent_hundredths, CpuSampler};
    use crate::domain::metadata::{FieldAvailability, UnavailableReason};
    use crate::domain::process::{ProcessId, ProcessIdentity, ProcessInfo, ProcessResourceSample};
    use std::ffi::OsString;
    use std::time::{Duration, Instant, UNIX_EPOCH};

    fn process(
        pid: u32,
        start: u64,
        cpu_seconds: u64,
        memory: Option<u64>,
        sampled_at: Instant,
    ) -> ProcessInfo {
        ProcessInfo {
            identity: ProcessIdentity {
                pid: ProcessId::new(pid),
                name: FieldAvailability::Available(OsString::from("fixture")),
                executable_path: FieldAvailability::Unavailable(
                    UnavailableReason::ProviderLimitation,
                ),
                start_time: FieldAvailability::Available(UNIX_EPOCH + Duration::from_secs(start)),
            },
            command_arguments: FieldAvailability::Unavailable(
                UnavailableReason::ProviderLimitation,
            ),
            working_directory: FieldAvailability::Unavailable(
                UnavailableReason::ProviderLimitation,
            ),
            resource_sample: ProcessResourceSample {
                cumulative_cpu_time: FieldAvailability::Available(Duration::from_secs(cpu_seconds)),
                resident_memory_bytes: memory.map_or(
                    FieldAvailability::Unavailable(UnavailableReason::PermissionDenied),
                    FieldAvailability::Available,
                ),
                sampled_at: Some(sampled_at),
            },
        }
    }

    #[test]
    fn cpu_normalizes_to_total_logical_capacity_and_caps_at_one_hundred_percent() {
        assert_eq!(
            cpu_percent_hundredths(Duration::from_millis(500), Duration::from_secs(1), 2),
            Some(2_500)
        );
        assert_eq!(
            cpu_percent_hundredths(Duration::from_secs(4), Duration::from_secs(1), 2),
            Some(10_000)
        );
        assert_eq!(
            cpu_percent_hundredths(Duration::ZERO, Duration::from_secs(1), 1),
            Some(0)
        );
    }

    #[test]
    fn cpu_rejects_zero_interval_or_cpu_count() {
        assert_eq!(
            cpu_percent_hundredths(Duration::from_secs(1), Duration::ZERO, 1),
            None
        );
        assert_eq!(
            cpu_percent_hundredths(Duration::from_secs(1), Duration::from_secs(1), 0),
            None
        );
    }

    #[test]
    fn first_sample_has_no_cpu_percentage_but_preserves_memory_and_uptime() {
        let mut sampler = CpuSampler::default();
        let now = Instant::now();
        let metrics = sampler.sample(
            &[(ProcessId::new(12), process(12, 10, 3, Some(0), now))],
            Some(4),
            UNIX_EPOCH + Duration::from_secs(16),
        );
        let metrics = metrics.get(&12).expect("metrics for process");
        assert_eq!(metrics.cpu_percent_hundredths, None);
        assert_eq!(metrics.resident_memory_bytes, Some(0));
        assert_eq!(metrics.uptime, Some(Duration::from_secs(6)));
    }

    #[test]
    fn second_sample_uses_counter_delta_and_snapshot_interval() {
        let mut sampler = CpuSampler::default();
        let start = Instant::now();
        let first = process(12, 10, 2, Some(100), start);
        sampler.sample(
            &[(ProcessId::new(12), first)],
            Some(2),
            UNIX_EPOCH + Duration::from_secs(12),
        );
        let second = process(12, 10, 3, Some(200), start + Duration::from_secs(1));
        let metrics = sampler.sample(
            &[(ProcessId::new(12), second)],
            Some(2),
            UNIX_EPOCH + Duration::from_secs(13),
        );
        assert_eq!(metrics[&12].cpu_percent_hundredths, Some(5_000));
        assert_eq!(metrics[&12].resident_memory_bytes, Some(200));
    }

    #[test]
    fn changed_process_start_time_does_not_inherit_prior_pid_sample() {
        let mut sampler = CpuSampler::default();
        let start = Instant::now();
        sampler.sample(
            &[(ProcessId::new(12), process(12, 10, 10, Some(1), start))],
            Some(1),
            UNIX_EPOCH + Duration::from_secs(20),
        );
        let metrics = sampler.sample(
            &[(
                ProcessId::new(12),
                process(12, 11, 30, Some(2), start + Duration::from_secs(1)),
            )],
            Some(1),
            UNIX_EPOCH + Duration::from_secs(21),
        );
        assert_eq!(metrics[&12].cpu_percent_hundredths, None);
        assert_eq!(sampler.previous_len(), 1);
    }

    #[test]
    fn unavailable_or_future_start_time_keeps_uptime_unavailable() {
        let mut sampler = CpuSampler::default();
        let mut info = process(12, 20, 0, None, Instant::now());
        info.identity.start_time =
            FieldAvailability::Available(UNIX_EPOCH + Duration::from_secs(30));
        let metrics = sampler.sample(
            &[(ProcessId::new(12), info)],
            None,
            UNIX_EPOCH + Duration::from_secs(29),
        );
        assert_eq!(metrics[&12].uptime, None);
        assert_eq!(metrics[&12].resident_memory_bytes, None);
        assert_eq!(sampler.previous_len(), 0);
    }

    #[test]
    fn stale_samples_are_removed_when_processes_disappear() {
        let mut sampler = CpuSampler::default();
        sampler.sample(
            &[(
                ProcessId::new(12),
                process(12, 10, 1, Some(1), Instant::now()),
            )],
            Some(1),
            UNIX_EPOCH + Duration::from_secs(12),
        );
        assert_eq!(sampler.previous_len(), 1);
        sampler.sample(&[], Some(1), UNIX_EPOCH + Duration::from_secs(13));
        assert_eq!(sampler.previous_len(), 0);
    }
}
