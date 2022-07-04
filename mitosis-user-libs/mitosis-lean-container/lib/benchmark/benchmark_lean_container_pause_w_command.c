#include "../core/lean_container.h"

#include <stdio.h>
#include <assert.h>
#include <unistd.h>
#include <signal.h>
#include <sys/prctl.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <errno.h>
#include <time.h>
#include <stdlib.h>

#define REPORT_INTERVAL_IN_SECOND 1
#define NANOSECONDS_IN_SECOND 1e9
#define NANOSECONDS_IN_MILLISECOND 1e6
#define MAX_COMMAND_LENGTH 256
#define MAX_ENV_VAR_COUNT 256

#define SOCKET_NAME "uds.socket"

static long count = 0;

static long get_passed_nanosecond(struct timespec* start, struct timespec* end) {
    return NANOSECONDS_IN_SECOND*(end->tv_sec - start->tv_sec) + (end->tv_nsec - start->tv_nsec);
}

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

void report(struct timespec* start, struct timespec* end) {
    static long last_count = 0;
    static long last_timestamp = 0;

    // Report every second
    static const long interval = REPORT_INTERVAL_IN_SECOND * NANOSECONDS_IN_SECOND;

    long elapsed_time = get_passed_nanosecond(start, end);
    long elapsed_time_since_last_report = elapsed_time - last_timestamp;
    if (elapsed_time_since_last_report > interval) {
        printf("pause/unpause %ld lean containers in %f second(s), latency per container %fms\n", count-last_count, elapsed_time_since_last_report/NANOSECONDS_IN_SECOND, (elapsed_time_since_last_report/NANOSECONDS_IN_MILLISECOND)/(count-last_count));
        last_timestamp = elapsed_time;
        last_count = count;
    }
    return;
}

int test_pause_unpause_container(char* name, int client_socket) {
    int ret;
    char buf;
    ret = unpause_container(name);
    if (ret != 0) {
        printf("unable to unpause container");
        return ret;
    }

    ret = send(client_socket, &buf, 1, 0);
    if (ret != 1) {
        printf("send error: %d\n", ret);
        perror("send");
        return ret;
    }

    ret = recv(client_socket, &buf, 1, 0);
    if (ret != 1) {
        printf("recv error: %d\n", ret);
        perror("recv");
        return ret;
    }

    ret = pause_container(name);
    if (ret != 0) {
        printf("unable to pause container");
        return ret;
    }
    return 0;
}

int main(int argc, char* argv[]) {
    if (argc < 5) {
        printf("Usage: %s [time in seconds to run the benchmark] [container name] [/path/to/rootfs] [command (absolute path)] [command opts]\n", argv[0]);
        return -1;
    }
    
    long benchmark_time = atol(argv[1]);
    long benchmark_time_nanoseconds = benchmark_time * NANOSECONDS_IN_SECOND;

    char* name = argv[2];
    char* rootfs_path = argv[3];
    char* command = argv[4];
    char* execve_argv[MAX_COMMAND_LENGTH];
    char* execve_envp[MAX_ENV_VAR_COUNT];
    char uds_socket_path[108];
    int argv_index = 0;

    struct timespec start, now;

    signal(SIGPIPE, SIG_IGN);

    // setup argv array
    for (int i = 4; i < argc && argv_index < MAX_COMMAND_LENGTH; i++, argv_index++)
        execve_argv[argv_index] = argv[i];
    execve_argv[argv_index] = NULL;

    // setup envp array
    // TODO: support more environment variables
    execve_envp[0] = "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
    execve_envp[1] = NULL;

    struct ContainerSpec spec;
    int ret;
    pid_t pid, cached_namespace;
    
    // unlimited resources
    spec.cpu_start = -1;
    spec.cpu_end = -1;
    spec.memory_in_mb = -1;
    spec.numa_start = -1;
    spec.numa_end = -1;
    
    ret = init_cgroup();
    assert(ret == 0);

    ret = add_lean_container_template(name, &spec);
    assert(ret == 0);

    cached_namespace = setup_cached_namespace(rootfs_path);

    // setup the lean container of `name`
    // and the rootfs of the lean container is specified by second parameter
    pid = setup_lean_container_w_double_fork(name, rootfs_path, cached_namespace);
    if (pid < 0) {
        printf("set lean container failed!\n");
        goto clean;
    }

    if (pid) {
        printf("this is the lean container launcher process!\n");
    } else {
        pid = getpid();
        printf("this is the process in the lean container, pid in container: %d\n", pid);

        // we are now running in the lean container!
        // launch the command by execve
        execve(command, execve_argv, execve_envp);

        // should never reach here
        assert(0);
    }

    int client_socket = socket(AF_UNIX, SOCK_STREAM, 0);
    if (client_socket < 0)
        assert(0);
    
    sprintf(uds_socket_path, "%s/%s", rootfs_path, SOCKET_NAME);

    struct sockaddr_un addr;
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, uds_socket_path, sizeof(addr.sun_path));

    printf("%s\n", addr.sun_path);

    // wait for the container process to listen on the unix domain socket
    sleep(5);

    if (connect(client_socket, (struct sockaddr *)&addr, sizeof(addr)) == -1)
        assert(0);

    long elapsed_time;

    clock_gettime(CLOCK_REALTIME, &start);
    for (;;) {
        test_pause_unpause_container(name, client_socket);
        clock_gettime(CLOCK_REALTIME, &now);
        count++;
        report(&start, &now);
        elapsed_time = get_passed_nanosecond(&start, &now);
        if (elapsed_time > benchmark_time_nanoseconds) {
            break;
        }
    }

    printf("total: pause/unpause %ld lean containers in %f second(s)\n", count, elapsed_time/NANOSECONDS_IN_SECOND);

    ret = unpause_container(name);
    if (ret != 0) {
        printf("unable to unpause container\n");
    }

    kill(pid, SIGKILL);

    pid_t child = wait_pid(pid);
    if (child != pid) {
        printf("child pid: %d, expected: %d\n", child, pid);
    }

clean:
    ret = remove_cached_namespace(cached_namespace, rootfs_path);
    assert(ret == 0);

    ret = remove_lean_container_template(name);
    assert(ret == 0);

    // we do not need to call `deinit_cgroup` as there are possibly other running benchmarks

    printf("pass lean container unit test!\n");
    return 0;
}
