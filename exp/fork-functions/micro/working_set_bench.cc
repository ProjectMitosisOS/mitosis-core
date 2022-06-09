#include <gflags/gflags.h>
#include <iostream>
#include <chrono>

#include <assert.h>
#include <time.h>
#include <stdio.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>

#include <string>

#define K 1024
#define M 1024 * (K)
#define PAGE_SIZE 4096
#define NANOSECONDS_IN_SECOND 1e9
#define NANOSECONDS_IN_MILLISECOND 1e6
#define REPORT_INTERVAL_IN_SECOND 1
#define MAX_COMMAND_LENGTH 256
DEFINE_int64(working_set, 16777216, "working set size");
DEFINE_int64(run_sec, 10, "running seconds");
DEFINE_int64(profile, 0, "profile each func latency");

static int count = 0;

extern "C"
{
void init_buffer(uint64_t workingset);
void handler(const char *name, uint64_t workingset, int profile);
}

static long get_passed_nanosecond(struct timespec *start, struct timespec *end) {
    return NANOSECONDS_IN_SECOND * (end->tv_sec - start->tv_sec) + (end->tv_nsec - start->tv_nsec);
}

void report(char *name, struct timespec *start, struct timespec *end) {
    static long last_count = 0;
    static long last_timestamp = 0;

    // Report every second
    static const long interval = REPORT_INTERVAL_IN_SECOND * NANOSECONDS_IN_SECOND;

    long elapsed_time = get_passed_nanosecond(start, end);
    long elapsed_time_since_last_report = elapsed_time - last_timestamp;
    if (elapsed_time_since_last_report > interval) {
        long op = count - last_count;
        double latency = (elapsed_time_since_last_report / NANOSECONDS_IN_MILLISECOND) / op;
        long qps = (op / (elapsed_time_since_last_report / NANOSECONDS_IN_SECOND));
        printf("[%s] Throughput: %ld containers/sec, latency %f ms\n", name, qps, latency);

        last_timestamp = elapsed_time;
        last_count = count;
    }
}

int main(int argc, char **argv) {
    gflags::ParseCommandLineFlags(&argc, &argv, true);
    struct timespec start, now;
    clock_gettime(CLOCK_REALTIME, &start);
    long elapsed_time;
    long benchmark_time_nanoseconds = FLAGS_run_sec * NANOSECONDS_IN_SECOND;
    init_buffer(FLAGS_working_set);

    for (;;) {
        handler("name", FLAGS_working_set, FLAGS_profile);
        clock_gettime(CLOCK_REALTIME, &now);
        count++;
        report("name", &start, &now);
        elapsed_time = get_passed_nanosecond(&start, &now);
        if (elapsed_time > benchmark_time_nanoseconds) {
            break;
        }
    }

    // free(buffer);
    return 0;
}