#ifndef LEAN_CONTAINER_H
#define LEAN_CONTAINER_H

struct ContainerSpec {
    // negative value or zero indicates unlimited resources
    int cpu_count;
    long memory_in_mb;
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

// setup lean container, with template name and the rootfs path
// returns the pid of the containered process in the parent process
// returns 0 in the containered process
// return negative value on failure
int setup_lean_container(char* name, char* rootfs_path);

// pause/unpause the corresponding container
// returns 0 on success
int pause_container(char* name);
int unpause_container(char* name);

#endif
