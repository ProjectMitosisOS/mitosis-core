#include "../../../mitosis-user-libs/mitosis-c-client/include/syscall.h"

#include <gflags/gflags.h>
#include <iostream>
#include <chrono>

#include <assert.h>
#include <time.h>
#include <stdio.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>

#include <string>

#define K 1024
#define M 1024 * (K)
#define PAGE_SIZE 4096

DEFINE_int64(working_set, 16777216, "working set size");
DEFINE_int64(handler_id, 73, "rfork handler id");
DEFINE_int32(whether_prepare, 0, "whether to prepare");
DEFINE_int32(profile, 1, "profile the result");
DEFINE_int32(touch_ratio, 100, "Working set touch ratio");
DEFINE_int32(exclude_execution, 0, "Return immediately after checkpoint");

extern "C"
{
    void init_buffer(uint64_t workingset);
    void handler(const char *name, uint64_t workingset);
}

template <class DT = std::chrono::milliseconds,
          class ClockT = std::chrono::steady_clock>
class Timer
{
    using timep_t = typename ClockT::time_point;
    timep_t _start = ClockT::now(), _end = {};

public:
    void tick()
    {
        _end = timep_t{};
        _start = ClockT::now();
    }

    void tock() { _end = ClockT::now(); }

    template <class T = DT>
    auto duration() const
    {
        return std::chrono::duration_cast<T>(_end - _start);
    }
};

int main(int argc, char **argv)
{
    gflags::ParseCommandLineFlags(&argc, &argv, true);
    printf("working set %d, mb: %lluMB\n", FLAGS_working_set, FLAGS_working_set / (1024 * 1024));

    int sd = sopen();

    init_buffer(FLAGS_working_set);

    // cold start
    handler("cold start", FLAGS_working_set);
    handler("warm start", FLAGS_working_set * FLAGS_touch_ratio / 100);

#if 1
    // prepare
    if (FLAGS_whether_prepare > 0)
    {
        Timer<std::chrono::microseconds, std::chrono::steady_clock> clock;
        clock.tick();
        fork_prepare_ping(sd, FLAGS_handler_id);
        clock.tock();
        if (FLAGS_profile != 0)
            std::cout << "Prepare time = " << double(clock.duration().count()) / 1000 << " ms\n";
    }

    // warm start
    if (!FLAGS_exclude_execution && FLAGS_profile != 0)
        handler("cow start", FLAGS_working_set * FLAGS_touch_ratio / 100);
    _Exit(0);
#else
    if (fork() == 0)
    {
        handler("fork child start", FLAGS_working_set);
    }
    else
    {
        sleep(2);
        handler("fork parent start", FLAGS_working_set);
    }
#endif
    // free(buffer);
    return 0;
}
