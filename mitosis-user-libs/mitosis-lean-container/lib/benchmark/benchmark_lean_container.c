#include "../core/lean_container.h"

#include <time.h>
#include <stdio.h>
#include <errno.h>
#include <assert.h>
#include <unistd.h>
#include <signal.h>
#include <sys/wait.h>
#include <sys/types.h>
#include <sys/prctl.h>

// #define DEBUG

#ifdef DEBUG
#define debug_printf(...) printf(__VA_ARGS__)
#else
#define debug_printf
#endif

pid_t wait_pid(pid_t pid) {
    int ret;
    while ((ret = kill(pid, 0)) == 0);
    if (ret == -1 && errno == ESRCH) {
        return pid;
    } else {
        printf("ret: %d\n", ret);
        perror("kill");
        return -1;
    }
}

static long get_passed_nanosecond(struct timespec* start, struct timespec* end) {
    return 1e9*(end->tv_sec - start->tv_sec) + (end->tv_nsec - start->tv_nsec);
}

int test_setup_lean_container(char* name) {
    pid_t pid = setup_lean_container_w_double_fork(name, ".");
    if (pid < 0) {
        debug_printf("set lean container failed!");
        return -1;
    }

    if (pid) {
        debug_printf("this is the lean container launcher process!\n");
    } else {
        // we are now running in the lean container!
        // exit immediately to avoid performance overhead in benchmark
        _exit(0);
    }

    // pid_t child = waitpid(pid, NULL, 0);
    pid_t child = wait_pid(pid);
    if (child != pid) {
        printf("child pid: %d, expected: %d\n", child, pid);
        return -1;
    }
    return 0;
}

int main() {
    char* name = "test";
    struct ContainerSpec spec;
    struct timespec start, now;
    int ret;
    pid_t pid;
    int loop = 100;
    
    spec.cpu_start = -1;
    spec.cpu_end = -1;
    spec.memory_in_mb = -1;
    spec.numa_start = -1;
    spec.numa_end = -1;

    // set this process as the subreaper
    // so that the process can reap the grandchild process
    // ret = prctl(PR_SET_CHILD_SUBREAPER, 1);
    // assert(ret == 0);
    
    ret = init_cgroup();
    assert(ret == 0);

    ret = add_lean_container_template(name, &spec);
    assert(ret == 0);

    clock_gettime(CLOCK_REALTIME, &start);

    for (int i = 0; i < loop; i++) {
        test_setup_lean_container(name);
    }

    clock_gettime(CLOCK_REALTIME, &now);

    long time = get_passed_nanosecond(&start, &now);

    printf("generate %d lean container: %ldns\n", loop, time);
    printf("average %fms\n", time/loop/1e6);
    
    printf("pass lean container unit test!\n");
clean:
    ret = remove_lean_container_template(name);
    assert(ret == 0);

    ret = deinit_cgroup();
    assert(ret == 0);

    printf("clean resources!\n");
    return 0;
}
