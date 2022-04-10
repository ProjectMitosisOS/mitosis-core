#include "../core/lean_container.h"

#include <assert.h>
#include <stdio.h>

int main() {
    char* name = "test";
    struct ContainerSpec spec;
    int ret;

    // unlimited resources
    spec.cpu_count = -1;
    spec.memory_in_mb = -1;
    spec.numa_count = -1;
    
    ret = init_cgroup();
    assert(ret == 0);

    ret = add_lean_container_template(name, &spec);
    assert(ret == 0);

    ret = remove_lean_container_template(name);
    assert(ret == 0);

    ret = deinit_cgroup();
    assert(ret == 0);

    printf("pass lean container unit test!\n");
    return 0;
}
