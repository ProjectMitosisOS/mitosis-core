#include <assert.h>
#include <string>
#include "syscall.h"

int
main(int argc, char *argv[]) {
    if (argc < 3) {
        printf("Wrong argc: %d. Usage: %s [mac_id] [handler_id]\n", argc, argv[0]);
        return -1;
    }

    int mac_id = std::stoi(argv[1]);
    int handler_id = std::stoi(argv[2]);
    int sd = sopen();
    assert(sd != 0);
    fork_resume_remote(sd, mac_id, handler_id);
    return 0;
}
