#include <errno.h>
#include <stddef.h>
#include <stdint.h>
#include <sys/proc.h>
#include <sys/sysctl.h>
#include <sys/types.h>

/* Internal Rust/C boundary: return 1 for absent, 2 for denied, 3 otherwise. */
int thaa_macos_process_start_time(int32_t pid, int64_t *seconds, int32_t *microseconds) {
    int mib[4] = {CTL_KERN, KERN_PROC, KERN_PROC_PID, pid};
    struct kinfo_proc process = {0};
    size_t length = sizeof(process);

    if (pid <= 0 || seconds == NULL || microseconds == NULL) {
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
    return 0;
}
