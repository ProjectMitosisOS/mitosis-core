#include <assert.h>
#include <gflags/gflags.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include "../../mitosis-user-libs/mitosis-c-client/include/syscall.h"

DEFINE_int64(mac_id, 0, "machine id");
DEFINE_int64(handler_id, 73, "rfork handler id");
DEFINE_int64(port, 8080, "socket port");

void wait() {
    int listenfd = 0, connfd = 0;
    struct sockaddr_in serv_addr;

    listenfd = socket(AF_INET, SOCK_DGRAM, 0);

    serv_addr.sin_family = AF_INET;
    serv_addr.sin_addr.s_addr = htonl(INADDR_ANY);
    serv_addr.sin_port = htons(FLAGS_port);

    bind(listenfd, (struct sockaddr *) &serv_addr, sizeof(serv_addr));
    char recv_buf[20];
    struct sockaddr_in addr_client;
    int len;

    int recv_num = recvfrom(listenfd,
                        recv_buf, sizeof(recv_buf), 0,
                        (struct sockaddr *) &addr_client,
                        (socklen_t *) &len);
}

int
main(int argc, char *argv[]) {
    gflags::ParseCommandLineFlags(&argc, &argv, true);
    int sd = sopen();
    wait();
    assert(sd != 0);
    fork_resume_remote(sd, FLAGS_mac_id, FLAGS_handler_id);
    assert(false);
    return 0;
}
