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

// freezer state in the cgroupfs
// note that FreezerState and ContainerState can be safely casted to each other
enum FreezerState {
    FREEZER_ERROR = -1,
    FREEZER_FROZEN,
    FREEZER_FREEZING,
    FREEZER_THAWED,
};

// container state for a lean container
// note that FreezerState and ContainerState can be safely casted to each other
enum ContainerState {
    CONTAINER_ERROR = FREEZER_ERROR,
    CONTAINER_PAUSED = FREEZER_FROZEN,
    CONTAINER_PAUSING = FREEZER_FREEZING,
    CONTAINER_RUNNING = FREEZER_THAWED,
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
// if _namespace is less than 0, the container will run in a new namespace created by unshare
// otherwise the specified namespace is reused
// returns the pid of the containered process in the parent process
// returns 0 in the containered process
// return negative value on failure
int setup_lean_container(char* name, char* rootfs_path, int _namespace);

// pause/unpause the corresponding container
// returns 0 on success
int pause_container(char* name);
int unpause_container(char* name);

// get the container state: CONTAINER_PAUSED/CONTAINER_PAUSING/CONTAINER_RUNNING
int get_container_state(char* name);

// wait one container to become the specified state
int wait_until(char* name, enum FreezerState expected);

// setup lean container, with an additional call to fork (a.k.a: double fork)
// so that the process is created in a new pid namespace
// if _namespace is less than 0, the container will run in a new namespace created by unshare
// otherwise the specified namespace is reused
int setup_lean_container_w_double_fork(char* name, char* rootfs_path, int _namespace);

// setup cached namespaces
// if rootfs is not NULL, we will mount a procfs in the rootfs
// the procfs is bound to the namespace
int setup_cached_namespace(char* rootfs);
int remove_cached_namespace(int _namespace, char* rootfs);

#endif
