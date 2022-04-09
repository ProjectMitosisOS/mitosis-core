#include <assert.h>
#include <iostream>
#include <gflags/gflags.h>

#include "../../mitosis-user-libs/mitosis-c-client//include/syscall.h"


DEFINE_string(gid, "fe80:0000:0000:0000:248a:0703:009c:7c94", "connect gid");

DEFINE_int64(mac_id, 1, "machine id");
DEFINE_int64(nic_id, 0, "nic idx. Should be align with gid");

int
main(int argc, char *argv[]) {
    gflags::ParseCommandLineFlags(&argc, &argv, true);
    int sd = sopen();
    assert(sd != 0);
    // target on val08
    auto res = call_connect(sd, FLAGS_gid.c_str(), FLAGS_mac_id, FLAGS_nic_id);
    std::cout<<"connect res: " << res << std::endl;
    return 0;
}
