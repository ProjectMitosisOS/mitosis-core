#include <assert.h>
#include <stdio.h>
#include <unistd.h>
#include <gflags/gflags.h>

#include "../../mitosis-user-libs/mitosis-c-client//include/syscall.h"

DEFINE_int64(handler_id, 73, "rfork handler id");

int
main(int argc, char *argv[]) {
    gflags::ParseCommandLineFlags(&argc, &argv, true);

    int sd = sopen();
    int cnt = 0;
    assert(sd != 0);
    sleep(1);
    printf("time %d\n", cnt++);
    fork_prepare(sd, FLAGS_handler_id);

    while (1) {
        printf("time %d\n", cnt++);
        sleep(1);
    }
    return 0;
}
