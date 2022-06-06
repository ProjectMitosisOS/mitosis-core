#include "core/lean_container.h"

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
#define NANOSECONDS_IN_MILLISECOND 1e6
#define REPORT_INTERVAL_IN_SECOND 1
#define MAX_COMMAND_LENGTH 256

static long count = 0;
char *execve_argv[MAX_COMMAND_LENGTH];
char *execve_envp[MAX_COMMAND_LENGTH];

pid_t wait_pid(pid_t pid)
{
    int ret;
    while ((ret = kill(pid, 0)) == 0)
        ;
    if (ret == -1 && errno == ESRCH)
    {
        return pid;
    }
    else
    {
        printf("ret: %d\n", ret);
        perror("kill");
        return -1;
    }
}

static long get_passed_nanosecond(struct timespec *start, struct timespec *end)
{
    return NANOSECONDS_IN_SECOND * (end->tv_sec - start->tv_sec) + (end->tv_nsec - start->tv_nsec);
}

void report(struct timespec *start, struct timespec *end)
{
    static long last_count = 0;
    static long last_timestamp = 0;

    // Report every second
    static const long interval = REPORT_INTERVAL_IN_SECOND * NANOSECONDS_IN_SECOND;

    long elapsed_time = get_passed_nanosecond(start, end);
    long elapsed_time_since_last_report = elapsed_time - last_timestamp;
    if (elapsed_time_since_last_report > interval)
    {
        long op = count - last_count;
        double latency = (elapsed_time_since_last_report / NANOSECONDS_IN_MILLISECOND) / op;
        long qps = (op / (elapsed_time_since_last_report / NANOSECONDS_IN_SECOND));
        printf("Throughput: %ld containers/sec, latency %f ms\n", qps, latency);

        last_timestamp = elapsed_time;
        last_count = count;
    }
}

int test_setup_lean_container(char *name, int namespace, char *rootfs_path, char *command)
{
    pid_t pid = setup_lean_container_w_double_fork(name,
                                                   rootfs_path,
                                                   namespace);

    if (pid < 0)
    {
        debug_printf("set lean container failed!");
        return -1;
    }

    if (pid)
    {
        debug_printf("this is the lean container launcher process!\n");
    }
    else
    {
        // we are now running in the lean container!
        // exit immediately to avoid performance overhead in benchmark
        execve(command, execve_argv, execve_envp);
        assert(0);
    }

    // wait for the containered process to exit
    pid_t child = wait_pid(pid);
    if (child != pid)
    {
        printf("child pid: %d, expected: %d\n", child, pid);
        return -1;
    }
    return 0;
}

int main(int argc, char **argv)
{
    if (argc < 2)
    {
        printf("Usage: %s [time in seconds to run the benchmark] [lean container name (default: test)]\n", argv[0]);
        return 0;
    }

    /* benchmark in seconds */
    long benchmark_time = atol(argv[1]);
    long benchmark_time_nanoseconds = benchmark_time * NANOSECONDS_IN_SECOND;
    
    /* name */
    char *name = argv[2];
    char *rootfs_abs_path = argv[3];
    char *command = argv[4];

    int argv_index = 0;
    for (int i = 4; i < argc && argv_index < MAX_COMMAND_LENGTH; ++i, ++argv_index)
    {
        execve_argv[argv_index] = argv[i];
    }
    execve_argv[argv_index] = NULL;

    printf("Running for %ld seconds, lean container name %s\n", benchmark_time, name);

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

    pid = setup_cached_namespace(NULL);

    ret = init_cgroup();
    assert(ret == 0);

    ret = add_lean_container_template(name, &spec);
    assert(ret == 0);

    clock_gettime(CLOCK_REALTIME, &start);

    for (;;)
    {
        test_setup_lean_container(name, pid, rootfs_abs_path, command);

        clock_gettime(CLOCK_REALTIME, &now);
        count++;
        report(&start, &now);
        elapsed_time = get_passed_nanosecond(&start, &now);
        if (elapsed_time > benchmark_time_nanoseconds)
        {
            break;
        }
    }

    printf("total: start %ld lean containers in %f second(s)\n", count, elapsed_time / NANOSECONDS_IN_SECOND);

clean:
    ret = remove_lean_container_template(name);
    assert(ret == 0);

    ret = remove_cached_namespace(pid, NULL);
    assert(ret == 0);

    printf("finish benchmarking lean container !\n");
    return 0;
}
