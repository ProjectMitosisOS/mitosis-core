#include <gflags/gflags.h>

#include <assert.h>
#include<time.h>
#include <stdio.h>
#include <unistd.h>

#include <string>
#include "../../mitosis-user-libs/mitosis-c-client/include/syscall.h"

#define K 1024
#define M 1024*(K)

char *buffer;
clock_t start, end;

DEFINE_int64(working_set, 16777216, "working set size");
DEFINE_int64(handler_id, 73, "rfork handler id");
DEFINE_int64(run_sec, 10, "running seconds");

static inline void report(std::string name) {
    double tm = ((double) (end - start) / CLOCKS_PER_SEC) * 1000000;
    printf("[%s] time: %.2f us\n", name.c_str(), tm);
}

/**
 * @param: woking_sz: Touched memory in Bytes
 * */
static inline void touch_working_set(unsigned int working_sz) {
    start = clock();
    if (buffer == nullptr) return;
    int sum = 0;
    for (int i = 0; i < working_sz; ++i) {
        sum += buffer[i];
    }
    end = clock();
    report("execution");
}

int
main(int argc, char **argv) {
    gflags::ParseCommandLineFlags(&argc, &argv, true);

    int sd = sopen();
    int cnt = 0;
    assert(sd != 0);
    sleep(1);
    printf("counter %d, working set %d\n", cnt++, FLAGS_working_set);


    buffer = (char *) malloc(FLAGS_working_set * sizeof(char));
    free(buffer);
    buffer = (char *) malloc(FLAGS_working_set * sizeof(char));
    touch_working_set(FLAGS_working_set);
    fork_prepare(sd, FLAGS_handler_id);

    touch_working_set(FLAGS_working_set);
    while (cnt < FLAGS_run_sec) {
        printf("counter %d\n", cnt++);
        sleep(1);
    }
    free(buffer);
    return 0;
}
