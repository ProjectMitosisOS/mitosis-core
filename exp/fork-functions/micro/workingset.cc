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

char *buffer = nullptr;

template <class DT = std::chrono::milliseconds,
          class ClockT = std::chrono::steady_clock>
class Timer
{
    using timep_t = typename ClockT::time_point;
    timep_t _start = ClockT::now(), _end = {};

public:
    void tick() { 
        _end = timep_t{}; 
        _start = ClockT::now(); 
    }
    
    void tock() { _end = ClockT::now(); }
    
    template <class T = DT> 
    auto duration() const { 
        return std::chrono::duration_cast<T>(_end - _start); 
    }
};

static void init_buffer() { 
    for (uint64_t i = 0;i < (uint64_t)FLAGS_working_set; i += PAGE_SIZE) { 
        *((uint64_t *)(buffer + i)) = i * 73 + 12;
    }    
}

static void inline __attribute__((optimize("O2"))) handler(std::string name, uint64_t workingset)
{
    uint64_t sum = 0;
    int count = 0;
    Timer<std::chrono::nanoseconds , std::chrono::steady_clock> clock;
    auto random = clock.duration();

    clock.tick();

    auto gap = PAGE_SIZE;
    for (uint64_t i = 0;i < (uint64_t)workingset; i += gap) {
        // sum += *((uint64_t *)(buffer + i));
        *((uint64_t *)(buffer + i)) = i * 73 + random.count();        
        count += 1;
    }

    clock.tock();
    double time = double(clock.duration().count()) / 1e6;
    if (FLAGS_profile != 0)
        std::cout << "[" << name << "] Run time = " << time << " ms\n";
}

int main(int argc, char **argv)
{
    gflags::ParseCommandLineFlags(&argc, &argv, true);
    printf("working set %d, mb: %lluMB\n", FLAGS_working_set, FLAGS_working_set / (1024 * 1024));

    int sd = sopen();    

    // buffer = (char *)malloc(FLAGS_working_set * sizeof(char));
    buffer = (char *)mmap(
        nullptr,
        FLAGS_working_set,                         // for one page length
        PROT_READ|PROT_WRITE|PROT_EXEC,
        MAP_ANON|MAP_PRIVATE,             // to a private block of hardware memory
        0,
        0
      );    

    // cold start 
    handler("cold start",  FLAGS_working_set);
    handler("warm start",  FLAGS_working_set * FLAGS_touch_ratio / 100);

    // prepare 
    if (FLAGS_whether_prepare > 0) {
        Timer<std::chrono::microseconds , std::chrono::steady_clock> clock;
        clock.tick();
        fork_prepare_ping(sd, FLAGS_handler_id);    
        clock.tock();
        if (FLAGS_profile != 0)
            std::cout << "Prepare time = " << double(clock.duration().count())/1000 << " ms\n";
    }

    // warm start
    if (!FLAGS_exclude_execution)
        handler("cow start", FLAGS_working_set * FLAGS_touch_ratio / 100);
    _Exit(0);

    // free(buffer);
    return 0;
}
