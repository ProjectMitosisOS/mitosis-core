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
// #define DEBUG
#include <pthread.h>

#ifdef DEBUG
#define debug_printf(...) printf(__VA_ARGS__)
#else
#define debug_printf
#endif

#define NANOSECONDS_IN_SECOND 1e9
#define NANOSECONDS_IN_MILLISECOND 1e6
#define REPORT_INTERVAL_IN_SECOND 1
#define MAX_COMMAND_LENGTH 256

static long count = 0;
static char *execve_argv[MAX_COMMAND_LENGTH];
static char *execve_envp[MAX_COMMAND_LENGTH];
static int empty_process;

typedef struct thread_args {
    int worker_id;
} thargs_t;


int genRandomString(int length, char *ouput) {
    int flag, i;
    time_t t;
    t = time(NULL);
    pid_t pid = getpid();
    srand((unsigned) t + pid);
    for (i = 0; i < length - 1; i++) {
        flag = rand() % 3;
        switch (flag) {
            case 0:
                ouput[i] = 'A' + rand() % 26;
                break;
            case 1:
                ouput[i] = 'a' + rand() % 26;
                break;
            case 2:
                ouput[i] = '0' + rand() % 10;
                break;
            default:
                ouput[i] = 'x';
                break;
        }
    }
    ouput[length - 1] = '\0';
    return 0;
}

static long get_passed_nanosecond(struct timespec *start, struct timespec *end) {
    return NANOSECONDS_IN_SECOND * (end->tv_sec - start->tv_sec) + (end->tv_nsec - start->tv_nsec);
}

void report(char *name, struct timespec *start, struct timespec *end) {
    static long last_count = 0;
    static long last_timestamp = 0;

    // Report every second
    static const long interval = REPORT_INTERVAL_IN_SECOND * NANOSECONDS_IN_SECOND;

    long elapsed_time = get_passed_nanosecond(start, end);
    long elapsed_time_since_last_report = elapsed_time - last_timestamp;
    if (elapsed_time_since_last_report > interval) {
        long op = count - last_count;
        double latency = (elapsed_time_since_last_report / NANOSECONDS_IN_MILLISECOND) / op;
        long qps = (op / (elapsed_time_since_last_report / NANOSECONDS_IN_SECOND));
        printf("[%s] Throughput: %ld containers/sec, latency %f ms\n", name, qps, latency);

        last_timestamp = elapsed_time;
        last_count = count;
    }
}

/**
 * Body function for starting lean container
 * */
static int test_setup(char *name, int namespace, char *rootfs_path, char *command) {
    pid_t pid = fork();
    if (pid < 0) {
        debug_printf("set lean container failed!");
        return -1;
    }

    if (pid) {
        debug_printf("this is the lean container launcher process!\n");
    } else {
        // we are now running in the lean container!
        // exit immediately to avoid performance overhead in benchmark
        if (!empty_process) {
            execve(command, execve_argv, execve_envp);
            assert(0);
        }
        exit(0);
    }

    // wait for the containered process to exit
    pid_t child = waitpid(pid, NULL, 0);
    if (child != pid) {
        printf("child pid: %d, expected: %d\n", child, pid);
        return -1;
    }
    return 0;
}

void *worker(void *thrgs) {
    thargs_t *args = (thargs_t *) thrgs;
    printf("The worker %d start\n", args->worker_id);

    pthread_exit(NULL);
}

int master(int thread_num, long bench_sec) {
    pthread_t threads[thread_num];
    thargs_t thrgs[thread_num];

    for (int i = 0; i < thread_num; ++i) {
        thrgs[i].worker_id = i;
        pthread_create(&threads[i], NULL, worker, &thrgs[i]);
    }

    sleep(bench_sec);

    for (int i = 0; i < thread_num; ++i) {
        pthread_join(threads[i], NULL);
    }
}


int main(int argc, char **argv) {
    if (argc < 2) {
        printf("Usage: %s [time in seconds to run the benchmark] [lean container name (default: test)]\n", argv[0]);
        return 0;
    }

    /* benchmark in seconds */
    long benchmark_time = atol(argv[1]);
    long benchmark_time_nanoseconds = benchmark_time * NANOSECONDS_IN_SECOND;

    empty_process = atoi(argv[2]);
    /* name */
    char *rootfs_abs_path = argv[3];
    char *command = argv[4];

    int len = 12;
    char rand_name[len];
    genRandomString(len, rand_name);
    char *name = rand_name;
    int argv_index = 0;
    for (int i = 4; i < argc && argv_index < MAX_COMMAND_LENGTH; ++i, ++argv_index) {
        execve_argv[argv_index] = argv[i];
    }
    execve_argv[argv_index] = NULL;

    printf("Running for %ld seconds, lean container name %s\n", benchmark_time, name);
//    master(thread_num, benchmark_time);

    struct timespec start, now;
    int ret;
    pid_t pid;
    long elapsed_time;

    clock_gettime(CLOCK_REALTIME, &start);

    for (;;) {
        test_setup(name, pid, rootfs_abs_path, command);
//        usleep(500 * 1000);
        clock_gettime(CLOCK_REALTIME, &now);
        count++;
        report(name, &start, &now);
        elapsed_time = get_passed_nanosecond(&start, &now);
        if (elapsed_time > benchmark_time_nanoseconds) {
            break;
        }
    }

    printf("total: start %ld raw processes in %f second(s)\n", count, elapsed_time / NANOSECONDS_IN_SECOND);

    return 0;
}
