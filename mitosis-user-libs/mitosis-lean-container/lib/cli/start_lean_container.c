#include "../core/lean_container.h"

#include <time.h>
#include <stdio.h>
#include <errno.h>
#include <assert.h>
#include <unistd.h>
#include <signal.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <sys/types.h>
#include <sys/prctl.h>


#ifdef DEBUG
#define debug_printf(...) printf(__VA_ARGS__)
#else
#define debug_printf
#endif

#define NANOSECONDS_IN_SECOND 1e9
#define NANOSECONDS_IN_MILLISECOND 1e6
#define MAX_COMMAND_LENGTH 256
#define MAX_ENV_VAR_COUNT 256

char *execve_argv[MAX_COMMAND_LENGTH];
char *execve_envp[MAX_COMMAND_LENGTH];


static long get_passed_nanosecond(struct timespec *start, struct timespec *end) {
    return NANOSECONDS_IN_SECOND * (end->tv_sec - start->tv_sec) + (end->tv_nsec - start->tv_nsec);
}

/**
 * Body function for starting lean container
 * */
static inline int test_setup_lean_container(char *name, int namespace, char *rootfs_path, char *command) {
    pid_t pid = setup_lean_container_w_double_fork(name,
                                                   rootfs_path,
                                                   namespace);

    if (pid < 0) {
        debug_printf("set lean container failed!");
        return -1;
    }

    if (pid) {
        debug_printf("this is the lean container launcher process!\n");
    } else {
        // we are now running in the lean container!
        // launch the command by execve
        execve(command, execve_argv, execve_envp);

        // should never reach here
        assert(0);
    }

    int ret = 0;
    ret = pause_container(name);
    if (ret != 0) {
        printf("unable to pause container");
    }

    ret = unpause_container(name);
    if (ret != 0) {
        printf("unable to unpause container");
    }

    // wait for the containered process to exit
    pid_t child = waitpid(pid, NULL, 0);
    if (child != pid) {
        printf("child pid: %d, expected: %d\n", child, pid);
        return -1;
    }
    return 0;
}


int main(int argc, char* argv[]) {
    if (argc < 4) {
        printf("Usage: %s [container name] [/path/to/rootfs] [command (absolute path)] [command opts]\n", argv[0]);
        return -1;
    }
    
    char* name = argv[1];
    char* rootfs_path = argv[2];
    char* command = argv[3];
    int argv_index = 0;

    // setup argv array
    for (int i = 3; i < argc && argv_index < MAX_COMMAND_LENGTH; i++, argv_index++)
        execve_argv[argv_index] = argv[i];
    execve_argv[argv_index] = NULL;

    // setup envp array
    // TODO: support more environment variables
    execve_envp[0] = "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
    execve_envp[1] = NULL;

    struct ContainerSpec spec;
    int ret;
    int count = 0;
    pid_t pid, cached_namespace;
    struct timespec start, now;
    
    // unlimited resources
    spec.cpu_start = -1;
    spec.cpu_end = -1;
    spec.memory_in_mb = -1;
    spec.numa_start = -1;
    spec.numa_end = -1;

    ret = prctl(PR_SET_CHILD_SUBREAPER);
    assert(ret == 0);
    
    ret = init_cgroup();
    assert(ret == 0);

    ret = add_lean_container_template(name, &spec);
    assert(ret == 0);

    cached_namespace = setup_cached_namespace(rootfs_path);


    clock_gettime(CLOCK_REALTIME, &start);

    while (count < 100) {
        test_setup_lean_container(name, cached_namespace, rootfs_path, command);
//        usleep(500 * 1000);
        count++;
    }
    clock_gettime(CLOCK_REALTIME, &now);

    long elapsed_time = get_passed_nanosecond(&start, &now);
    printf("total: run %ld containers in %.2f second(s)\n", count, elapsed_time / NANOSECONDS_IN_SECOND);

clean:
    ret = remove_cached_namespace(cached_namespace, rootfs_path);
    assert(ret == 0);

    ret = remove_lean_container_template(name);
    assert(ret == 0);

    ret = deinit_cgroup();
    assert(ret == 0);

    // printf("pass lean container unit test!\n");
    return 0;
}
