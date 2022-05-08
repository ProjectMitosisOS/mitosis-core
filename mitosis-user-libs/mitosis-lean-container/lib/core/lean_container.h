#ifndef LEAN_CONTAINER_H
#define LEAN_CONTAINER_H

struct ContainerSpec {
    // the process can run on cpu cores [cpu_start, cpu_end]
    // negative value indicates unlimited cpu resources
    int cpu_start;
    int cpu_end;
    // negative value or zero indicates unlimited memory resources
    long memory_in_mb;
    // the process can run on numa nodes [numa_start, numa_end]
    // negative value indicates unlimited numa resources
    int numa_start;
    int numa_end;
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
int setup_lean_container(char* name, char* rootfs_path, int namespace);

// pause/unpause the corresponding container
// returns 0 on success
int pause_container(char* name);
int unpause_container(char* name);

// setup lean container, with an additional call to fork (a.k.a: double fork)
// so that the process is created in a new pid namespace
int setup_lean_container_w_double_fork(char* name, char* rootfs_path, int namespace);

// setup cached namespaces
int setup_cached_namespace(char* rootfs);
int remove_cached_namespace(int namespace);

#endif
