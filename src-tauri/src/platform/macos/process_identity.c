#include <errno.h>
#include <libproc.h>
#include <signal.h>
#include <stddef.h>
#include <stdint.h>
#include <sys/proc.h>
#include <sys/sysctl.h>
#include <sys/types.h>
#include <unistd.h>

_Static_assert(sizeof(pid_t) == sizeof(int32_t), "pid_t must match the checked Rust PID width");

/* Internal Rust/C boundary: return 1 for absent, 2 for denied, 3 otherwise. */
int thaa_macos_process_snapshot(int32_t pid, int64_t *seconds, int32_t *microseconds, int32_t *parent_pid) {
    int mib[4] = {CTL_KERN, KERN_PROC, KERN_PROC_PID, pid};
    struct kinfo_proc process = {0};
    size_t length = sizeof(process);

    if (pid <= 0 || seconds == NULL || microseconds == NULL || parent_pid == NULL) {
        return 3;
    }

    if (sysctl(mib, 4, &process, &length, NULL, 0) != 0) {
        if (errno == ESRCH) {
            return 1;
        }
        if (errno == EACCES || errno == EPERM) {
            return 2;
        }
        return 3;
    }

    if (length == 0) {
        return 1;
    }

    if (length != sizeof(process) || process.kp_proc.p_pid != pid) {
        return 3;
    }

    if (process.kp_proc.p_stat == SZOMB) {
        return 1;
    }

    if (process.kp_proc.p_starttime.tv_sec < 0 ||
        process.kp_proc.p_starttime.tv_usec < 0 ||
        process.kp_proc.p_starttime.tv_usec >= 1000000) {
        return 3;
    }

    *seconds = (int64_t)process.kp_proc.p_starttime.tv_sec;
    *microseconds = (int32_t)process.kp_proc.p_starttime.tv_usec;
    if (process.kp_eproc.e_ppid < 0) {
        return 3;
    }
    *parent_pid = (int32_t)process.kp_eproc.e_ppid;
    return 0;
}

/* Internal Rust/C boundary: return stable status codes, not errno values. */
int thaa_macos_process_resources(int32_t pid, uint64_t *cpu_nanoseconds, uint64_t *resident_bytes) {
    struct rusage_info_v4 usage = {0};

    if (pid <= 0 || cpu_nanoseconds == NULL || resident_bytes == NULL) {
        return 3;
    }

    if (proc_pid_rusage(pid, RUSAGE_INFO_V4, (rusage_info_t *)&usage) != 0) {
        if (errno == ESRCH) {
            return 1;
        }
        if (errno == EACCES || errno == EPERM) {
            return 2;
        }
        return 3;
    }

    if (UINT64_MAX - usage.ri_user_time < usage.ri_system_time) {
        return 3;
    }

    *cpu_nanoseconds = usage.ri_user_time + usage.ri_system_time;
    *resident_bytes = usage.ri_resident_size;
    return 0;
}

/* Rust action values: 1 = graceful/SIGTERM, 2 = force/SIGKILL.
 * Return values are stable internal statuses, not errno values. */
int thaa_macos_signal_process(int32_t pid, int32_t action) {
    int signal_number;

    if (pid <= 0) {
        return 5;
    }

    switch (action) {
        case 1:
            signal_number = SIGTERM;
            break;
        case 2:
            signal_number = SIGKILL;
            break;
        default:
            return 5;
    }

    if (kill((pid_t)pid, signal_number) == 0) {
        return 0;
    }

    /* Capture errno before any further operation can overwrite it. */
    int saved_errno = errno;
    if (saved_errno == ESRCH) {
        return 1;
    }
    if (saved_errno == EPERM) {
        return 2;
    }
    if (saved_errno == EINVAL) {
        return 3;
    }
    return 4;
}
