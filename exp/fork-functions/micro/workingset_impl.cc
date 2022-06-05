#include <string>
#include <chrono>
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <iostream>

#define PAGE_SIZE 4096

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

char *buffer = nullptr;

void init_buffer(uint64_t workingset)
{
    buffer = (char *)mmap(
        nullptr,
        workingset, // for one page length
        PROT_READ | PROT_WRITE | PROT_EXEC,
        MAP_ANON | MAP_PRIVATE, // to a private block of hardware memory
        0,
        0);
}

void __attribute__((optimize("O2"))) handler(std::string name, uint64_t workingset)
{
    uint64_t sum = 0;
    int count = 0;
    Timer<std::chrono::nanoseconds, std::chrono::steady_clock> clock;
    auto random = clock.duration();

    clock.tick();

    auto gap = PAGE_SIZE;
    for (uint64_t i = 0; i < (uint64_t)workingset; i += gap)
    {
        // sum += *((uint64_t *)(buffer + i));
        *((uint64_t *)(buffer + i)) = i * 73 + random.count();
        count += 1;
    }

    clock.tock();
    double time = double(clock.duration().count()) / 1e6;
    std::cout << "[" << name << "] Run time = " << time << " ms\n";
}