#include <assert.h>
#include <stdio.h>
#include <unistd.h>
#include <gflags/gflags.h>

#include "../../mitosis-user-libs/mitosis-c-client/include/syscall.h"

DEFINE_int64(handler_id, 73, "rfork handler id");
DEFINE_bool(pin, false, "whether pin in kernel");

int
main(int argc, char *argv[]) {
    gflags::ParseCommandLineFlags(&argc, &argv, true);

    int sd = sopen();
    int cnt = 0;
    assert(sd != 0);

    if (FLAGS_pin) {
        fork_prepare_ping(sd, FLAGS_handler_id);
        // return immediately
        return 0;
    } else {
        sleep(1);
        printf("time %d\n", cnt++);
        fork_prepare(sd, FLAGS_handler_id);
    }

    while (1) {
        printf("time %d\n", cnt++);
        sleep(1);
    }
}
