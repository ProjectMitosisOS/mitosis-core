#include <assert.h>
#include "syscall.h"

int
main()
{
    int sd = sopen();
    assert(sd != 0);

    int res = call_swap_rpc(sd, 73);
    while (1) {}
    return 0;
}
