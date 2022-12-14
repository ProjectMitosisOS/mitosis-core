#include "../core/lean_container.h"

#include <assert.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>

int main() {
    char* name = "test";
    struct ContainerSpec spec;
    int ret;
    pid_t pid;
    
    // limit memory to 128MB
    spec.cpu_start = -1;
    spec.cpu_end = -1;
    spec.memory_in_mb = 128;
    spec.numa_start = -1;
    spec.numa_end = -1;
    
    ret = init_cgroup();
    assert(ret == 0);

    ret = add_lean_container_template(name, &spec);
    assert(ret == 0);

    // setup the lean container of `name`
    // and the rootfs of the lean container is specified by second parameter
    pid = setup_lean_container(name, ".", -1);
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
        return 0;
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
