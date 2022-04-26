#include <assert.h>
#include <unistd.h>
#include <stdio.h>
#include "../../mitosis-user-libs/mitosis-c-client/include/syscall.h"

int
main()
{
    int sd = sopen();
    assert(sd != 0);
    int cnt = 0;
    printf("time %d\n", cnt++);
    sleep(1);

    fork_prepare(sd, 73);

    while (1) {
        printf("time %d\n", cnt++);
        sleep(1);
    }
    return 0;
}
