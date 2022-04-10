#define _GNU_SOURCE
#include <sched.h>
#include <stddef.h>
#include <stdio.h>
#include <errno.h>
#include <unistd.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <string.h>

#include "lean_container.h"

#define BUF_SIZE 256
#define DEFAULT_PERMISSION S_IRWXU|S_IRGRP|S_IXGRP|S_IROTH|S_IXOTH

char* cgroup_directory_prefix[] = {
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

void cgroup_file_name(char* buf, const char* prefix, const char* name) {
    char path_buf[BUF_SIZE];
    sprintf(path_buf, prefix, name);
    sprintf(buf, "%s%s", path_buf, "/cgroup.procs");
}

int write_pid(pid_t pid, const char* cgroupfs_path) {
    char buf[BUF_SIZE];
    sprintf(buf, "%d", pid);
    size_t len = strlen(buf);
    
    int fd = open(cgroupfs_path, O_WRONLY);
    if (fd < 0) {
        perror("open");
        return -1;
    }

    ssize_t ret = write(fd, buf, len);
    if (ret != len) {
        fprintf(stderr, "write pid %s to %s returns %ld, expected %ld\n", buf, cgroupfs_path, ret, len);
        close(fd);
        return -1;
    }

    close(fd);
    return 0;
}

int init_cgroup() {
    int ret;
    char buf[BUF_SIZE];
    for (char** cgroup = cgroup_directory_prefix; *cgroup != NULL; cgroup++) {
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
    for (char** cgroup = cgroup_directory_prefix; *cgroup != NULL; cgroup++) {
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
    for (char** cgroup = cgroup_directory_prefix; *cgroup != NULL; cgroup++) {
        sprintf(buf, *cgroup, name);
        ret = mkdir(buf, DEFAULT_PERMISSION);
        if (ret < 0 && errno != EEXIST) {
            perror("mkdir");
            return -1;
        }
    }
    // TODO: add memory and cpu configurations here
    return 0;
}

int remove_lean_container_template(char* name) {
    char buf[BUF_SIZE];
    int ret;
    for (char** cgroup = cgroup_directory_prefix; *cgroup != NULL; cgroup++) {
        sprintf(buf, *cgroup, name);
        ret = rmdir(buf);
        if (ret < 0 && errno != ENOENT) {
            perror("rmdir: ");
            return -1;
        }
    }
    return 0;
}

int setup_lean_container(char* name, char* rootfs_path) {
    int ret;
    int pipefd[2];
    pid_t pid;

    if (pipe(pipefd) < 0) {
        perror("pipe");
        return -1;
    }

    if (unshare(CLONE_NEWUTS | CLONE_NEWPID | CLONE_NEWIPC | CLONE_NEWNS) < 0) {
        perror("unshare");
        goto err;
    }

    pid = fork();
    if (pid < 0) {
        perror("fork");
        goto err;
    }

    if (pid) {
        // parent process
        // write the child pid to the cgroupfs
        char sign = 'a';
        char path_buf[BUF_SIZE];
        for (char** cgroup = cgroup_directory_prefix; *cgroup != NULL; cgroup++) {
            cgroup_file_name(path_buf, *cgroup, name);
            ret = write_pid(pid, path_buf);
            if (ret < 0) {
                goto err;
            }
        }
        
        // write a sign to the pipe fd to inform the child process to run
        write(pipefd[1], &sign, sizeof(sign));
        close(pipefd[0]);
        close(pipefd[1]);
        return pid;
    } else {
        // child process
        char sign;
        read(pipefd[0], &sign, sizeof(sign));
        close(pipefd[0]);
        close(pipefd[1]);
        return 0;
    }

err:
    close(pipefd[0]);
    close(pipefd[1]);
    return -1;
}
