#include "../core/lean_container.h"

#include <time.h>
#include <stdio.h>
#include <errno.h>
#include <assert.h>
#include <unistd.h>
#include <signal.h>
#include <stdlib.h> 
#include <sys/wait.h>
#include <sys/types.h>
#include <sys/prctl.h>

// #define DEBUG

#ifdef DEBUG
#define debug_printf(...) printf(__VA_ARGS__)
#else
#define debug_printf
#endif

#define NANOSECONDS_IN_SECOND 1e9
#define REPORT_INTERVAL_IN_SECOND 1

static long count = 0;

pid_t wait_pid(pid_t pid) {
    int ret;
    while ((ret = kill(pid, 0)) == 0);
    if (ret == -1 && errno == ESRCH) {
        return pid;
    } else {
        printf("ret: %d\n", ret);
        perror("kill");
        return -1;
    }
}

static long get_passed_nanosecond(struct timespec* start, struct timespec* end) {
    return NANOSECONDS_IN_SECOND*(end->tv_sec - start->tv_sec) + (end->tv_nsec - start->tv_nsec);
}

void report(struct timespec* start, struct timespec* end) {
    static long last_count = 0;
    static long last_timestamp = 0;

    // Report every second
    static const long interval = REPORT_INTERVAL_IN_SECOND * NANOSECONDS_IN_SECOND;

    long elapsed_time = get_passed_nanosecond(start, end);
    long elapsed_time_since_last_report = elapsed_time - last_timestamp;
    if (elapsed_time_since_last_report > interval) {
        printf("start %ld lean containers in %f second(s)\n", count-last_count, elapsed_time_since_last_report/NANOSECONDS_IN_SECOND);
        last_timestamp = elapsed_time;
        last_count = count;
    }
    return;
}

int test_setup_lean_container(char* name, int namespace) {
    pid_t pid = setup_lean_container_w_double_fork(name, ".", namespace);
    if (pid < 0) {
        debug_printf("set lean container failed!");
        return -1;
    }


    if (pid) {
        debug_printf("this is the lean container launcher process!\n");
    } else {
        // we are now running in the lean container!
        // exit immediately to avoid performance overhead in benchmark
        _exit(0);
    }

    pid_t child = wait_pid(pid);
    if (child != pid) {
        printf("child pid: %d, expected: %d\n", child, pid);
        return -1;
    }
    return 0;
}

int main(int argc, char** argv) {
    if (argc != 2) {
        printf("Usage: %s [time in seconds to run the benchmark]\n", argv[0]);
        return 0;
    }

    long benchmark_time = atol(argv[1]);
    long benchmark_time_nanoseconds = benchmark_time * NANOSECONDS_IN_SECOND;

    printf("Running for %ld seconds\n", benchmark_time);

    char* name = "test";
    struct ContainerSpec spec;
    struct timespec start, now;
    int ret;
    pid_t pid;
    long elapsed_time;
    
    spec.cpu_start = -1;
    spec.cpu_end = -1;
    spec.memory_in_mb = -1;
    spec.numa_start = -1;
    spec.numa_end = -1;

    pid = setup_cached_namespace();
    
    ret = init_cgroup();
    assert(ret == 0);

    ret = add_lean_container_template(name, &spec);
    assert(ret == 0);


    clock_gettime(CLOCK_REALTIME, &start);
    for (;;) {
        test_setup_lean_container(name, pid);
        clock_gettime(CLOCK_REALTIME, &now);
        count++;
        report(&start, &now);
        elapsed_time = get_passed_nanosecond(&start, &now);
        if (elapsed_time > benchmark_time_nanoseconds) {
            break;
        }
    }

    printf("total: start %ld lean containers in %f second(s)\n", count, elapsed_time/NANOSECONDS_IN_SECOND);
    
    printf("pass lean container unit test!\n");
clean:
    ret = remove_lean_container_template(name);
    assert(ret == 0);

    ret = deinit_cgroup();
    assert(ret == 0);

    ret = remove_cached_namespace(pid);
    assert(ret == 0);

    printf("clean resources!\n");
    return 0;
}
