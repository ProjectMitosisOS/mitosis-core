#ifndef LEAN_CONTAINER_H
#define LEAN_CONTAINER_H

struct ContainerSpec {
    // negative value or zero indicates unlimited resources
    int cpu_count;
    int memory_in_mb;
    int numa_count;
};

// (de)initiate the mitosis cgroupfs
// return 0 on success
// return negative value on failure
int init_cgroup();
int deinit_cgroup();

// create/remove mitosis lean container templates
// return 0 on success
// return negative value on failure
int add_lean_container_template(char* name, struct ContainerSpec* spec);
int remove_lean_container_template(char* name);

// setup lean container
// returns the pid of the containered process in the parent process
// returns 0 in the containered process
// return negative value on failure
int setup_lean_container(char* name, char* rootfs_path);

#endif
