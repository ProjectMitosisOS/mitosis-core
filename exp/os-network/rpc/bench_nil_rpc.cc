#include <gflags/gflags.h>

#include <assert.h>
#include<time.h>
#include <unistd.h>
#include "../../../mitosis-user-libs/mitosis-c-client/include/syscall.h"
#include <iostream>
#include "bench_nil_rpc.hh"

#define K 1024
#define M 1024*(K)

DEFINE_int64(handler_id, 73, "handler id");
DEFINE_int64(mac_id, 0, "remote machine mac id");
DEFINE_int64(run_sec, 10, "running seconds");
DEFINE_string(gid, "fe80:0000:0000:0000:ec0d:9a03:0078:6416", "connect gid");
DEFINE_int64(nic_id, 0, "nic idx. Should be align with gid");
DEFINE_int64(threads, 1, "#Threads used.");


using Thread_t = Thread<usize>;

usize
worker_fn(const usize &worker_id, Statics *s);


bool volatile running = true;
static int sd;
int
main(int argc, char **argv) {
    gflags::ParseCommandLineFlags(&argc, &argv, true);

    std::vector<Thread_t *> workers;
    std::vector<Statics> worker_statics(FLAGS_threads);
    sd = sopen();
    assert(sd != 0);
    auto res = call_connect(sd, FLAGS_gid.c_str(), FLAGS_mac_id, FLAGS_nic_id);
    assert(res == 0);
    for (uint i = 0; i < FLAGS_threads; ++i) {
        workers.push_back(
                new Thread_t(std::bind(worker_fn, i, &(worker_statics[i]))));
    }

    // start the workers
    for (auto w: workers) {
        w->start();
    }
    report_thpt(worker_statics, FLAGS_run_sec); // report for 10 seconds
    running = false;                           // stop workers
    compile_fence();
    // wait for workers to join
    for (auto w: workers) {
        w->join();
    }

}


usize
worker_fn(const usize &worker_id, Statics *s) {
    Statics &ss = *s;
    while (running) {
        nil_rpc(sd, FLAGS_mac_id, FLAGS_handler_id);
        ss.increment(1);
    }
    return 0;
}