#include "../core/lean_container.h"

#include <assert.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>

#define MAX_COMMAND_LENGTH 256
#define MAX_ENV_VAR_COUNT 256

int main(int argc, char* argv[]) {
    if (argc < 3) {
        printf("Usage: %s [/path/to/rootfs] [command (absolute path)] [command opts]\n", argv[0]);
        return -1;
    }
    
    char* rootfs_path = argv[1];
    char* command = argv[2];
    char* name = "test";
    char* execve_argv[MAX_COMMAND_LENGTH];
    char* execve_envp[MAX_ENV_VAR_COUNT];
    int argv_index = 0;

    // setup argv array
    for (int i = 2; i < argc && argv_index < MAX_COMMAND_LENGTH; i++, argv_index++)
        execve_argv[argv_index] = argv[i];
    execve_argv[argv_index] = NULL;

    // setup envp array
    // TODO: support more 
    execve_envp[0] = "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
    execve_envp[1] = NULL;

    struct ContainerSpec spec;
    int ret;
    pid_t pid;
    
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

    // setup the lean container of `name`
    // and the rootfs of the lean container is specified by second parameter
    pid = setup_lean_container(name, rootfs_path, -1);
    if (pid < 0) {
        printf("set lean container failed!");
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

    ret = pause_container(name);
    if (ret != 0) {
        printf("unable to pause container");
    }

    ret = unpause_container(name);
    if (ret != 0) {
        printf("unable to unpause container");
    }

    pid_t child = waitpid(-1, NULL, 0);
    if (child != pid) {
        printf("child pid: %d, expected: %d\n", child, pid);
    }

clean:
    ret = remove_lean_container_template(name);
    assert(ret == 0);

    ret = deinit_cgroup();
    assert(ret == 0);

    printf("pass lean container unit test!\n");
    return 0;
}
