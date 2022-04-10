#ifndef LEAN_CONTAINER_H
#define LEAN_CONTAINER_H

struct ContainerSpec {
    // negative value indicate unlimited resources
    long cpu_count;
    long memory_in_mb;
};

int init_cgroup();
int deinit_cgroup();
int add_lean_container_template(char* name, struct ContainerSpec* spec);
int remove_lean_container_template(char* name);

int setup_lean_container(char* name, char* rootfs_path);

#endif
