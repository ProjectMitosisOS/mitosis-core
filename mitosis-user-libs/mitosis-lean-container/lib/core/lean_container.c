#include <stddef.h>
#include <stdio.h>
#include <errno.h>
#include <unistd.h>
#include <sys/stat.h>

#include "lean_container.h"

#define BUF_SIZE 256
#define DEFAULT_PERMISSION S_IRWXU|S_IRGRP|S_IXGRP|S_IROTH|S_IXOTH

char* cgroup_names[] = {
    "/sys/fs/cgroup/hugetlb/mitosis/%s",
    "/sys/fs/cgroup/perf_event/mitosis/%s",
    "/sys/fs/cgroup/net_cls,net_prio/mitosis/%s",
    "/sys/fs/cgroup/pids/mitosis/%s",
    "/sys/fs/cgroup/devices/mitosis/%s",
    "/sys/fs/cgroup/freezer/mitosis/%s",
    "/sys/fs/cgroup/cpu,cpuacct/mitosis/%s",
    "/sys/fs/cgroup/cpuset/mitosis/%s",
    "/sys/fs/cgroup/blkio/mitosis/%s",
    "/sys/fs/cgroup/memory/mitosis/%s",
    "/sys/fs/cgroup/systemd/mitosis/%s",
    NULL,
};

int init_cgroup() {
    int ret;
    char buf[BUF_SIZE];
    for (char** cgroup = cgroup_names; *cgroup != NULL; cgroup++) {
        sprintf(buf, *cgroup, "");
        ret = mkdir(buf, DEFAULT_PERMISSION);
        if (ret < 0 && errno != EEXIST) {
            perror("mkdir");
            return -1;
        }
    }
    return 0;
}

int deinit_cgroup() {
    int ret;
    char buf[BUF_SIZE];
    for (char** cgroup = cgroup_names; *cgroup != NULL; cgroup++) {
        sprintf(buf, *cgroup, "");
        ret = rmdir(buf);
        if (ret < 0 && errno != ENOENT) {
            perror("rmdir");
            return -1;
        }
    }
    return 0;
}

int add_lean_container_template(char* name, struct ContainerSpec* spec) {
    char buf[BUF_SIZE];
    int ret;
    for (char** cgroup = cgroup_names; *cgroup != NULL; cgroup++) {
        sprintf(buf, *cgroup, name);
        ret = mkdir(buf, DEFAULT_PERMISSION);
        if (ret < 0 && errno != EEXIST) {
            perror("mkdir: ");
            return -1;
        }
    }
    // TODO: add memory and cpu configurations here
    return 0;
}

int remove_lean_container_template(char* name) {
    char buf[BUF_SIZE];
    int ret;
    for (char** cgroup = cgroup_names; *cgroup != NULL; cgroup++) {
        sprintf(buf, *cgroup, name);
        ret = rmdir(buf);
        if (ret < 0 && errno != ENOENT) {
            perror("rmdir: ");
            return -1;
        }
    }
    return 0;
}
