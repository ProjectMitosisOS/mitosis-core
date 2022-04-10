#ifndef OS_SWAP_H
#define OS_SWAP_H

#include <fcntl.h>
#include <stdlib.h>
#include <sys/ioctl.h>

#include "./common.h"

static inline int
sopen()
{
    return open("/dev/mitosis-syscalls", O_RDWR);
}

static inline int
call_nil(int sd)
{
    if (ioctl(sd, Nil, 0) == -1) {
        return -1;
    }

    return 0;
}

/*
  Dump myself as an image to the kernel,
  so that later process can swap to it.
 */
static inline int
call_dump_myself(int sd, unsigned long key)
{
    if (ioctl(sd, Dump, key) == -1) {
        return -1;
    }

    return 0;
}

static inline int
call_swap(int sd, unsigned long key)
{
    if (ioctl(sd, Swap, key) == -1) {
        return -1;
    }

    return 0;
}


static inline int
call_swap_rpc(int sd, unsigned long key)
{
    if (ioctl(sd, SwapRPC, key) == -1) {
        return -1;
    }

    return 0;
}

#endif
