#include <assert.h>
#include <unistd.h>
#include "../../mitosis-user-libs/mitosis-c-client/include/syscall.h"

int
main()
{
    int sd = sopen();
    assert(sd != 0);
    sleep(1);

    fork_resume_remote(sd, 0, 73);
    assert(0);
    return 0;
}
