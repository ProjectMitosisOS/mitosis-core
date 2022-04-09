#include <fcntl.h>
#include <stdlib.h>
#include <sys/ioctl.h>

#include "./common.h"

static inline int
sopen() {
    return open("/dev/mitosis-syscalls", O_RDWR);
}

static inline int
call_nil(int sd) {
    if (ioctl(sd, Nil, 0) == -1) {
        return -1;
    }

    return 0;
}

static inline int
call_connect(int sd, const char *addr, unsigned int mac_id, unsigned int nic_id) {
    connect_req_t req;
    req.gid = addr;
    req.machine_id = mac_id;
    req.nic_id = nic_id;

    if (ioctl(sd, Connect, &req) == -1) {
        return -1;
    }

    return 0;
}

/*
  Dump myself as an image to the kernel,
  so that later process can swap to it.
 */
static inline int
fork_prepare(int sd, unsigned long key) {
    if (ioctl(sd, Prepare, key) == -1) {
        return -1;
    }

    return 0;
}

static inline int
fork_resume_local(int sd, unsigned long key) {
    if (ioctl(sd, ResumeLocal, key) == -1) {
        return -1;
    }

    return 0;
}


static inline int
fork_resume_remote(int sd, unsigned long mac_id, unsigned long handler_id) {
    resume_remote_req_t req;
    req.machine_id = mac_id;
    req.handler_id = handler_id;

    if (ioctl(sd, ResumeRemote, &req) == -1) {
        return -1;
    }

    return 0;
}

