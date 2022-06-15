//#include <stdint.h>

enum LibMITOSISCmd {
    Nil = 0,  // for test only
    Connect = 3, // connect to remote session
    Prepare = 4, // prepare the memory mapping of this process
    ResumeLocal = 5, // resume to another process
    ResumeRemote = 6, // resume to another process of remote via RPC
    PreparePing = 7, // Prepare the memory mapping of this process (and ping it in kernel)
    NilRPC = 8,     // Call the nil RPC function
};

typedef struct {
    unsigned int machine_id; // should not be zero!
    unsigned int nic_id; // nic idx according to gid
    const char *gid;
} connect_req_t;

typedef struct {
    unsigned int machine_id;    // keep `machine_id` the same as that in `connect_req_t`
    unsigned int handler_id;
} resume_remote_req_t;