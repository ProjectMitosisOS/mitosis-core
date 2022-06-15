#include <pthread.h>
#include <functional>
#include <chrono>
#include <assert.h>

using usize = unsigned int;

static inline void compile_fence(void) { asm volatile("":: : "memory"); }

template<typename T = int>
class alignas(128) Thread {
    using thread_body_t = std::function<T(void)>;

    thread_body_t core_func;
    T res;
    pthread_t pid; // pthread id

public:
    explicit Thread(const thread_body_t &b) : core_func(b) {}

    void start() {
        pthread_attr_t attr;
        assert(pthread_attr_init(&attr) == 0);
        assert(pthread_create(&pid, &attr, pthread_bootstrap, (void *) this) == 0);
        assert(pthread_attr_destroy(&attr) == 0);
    }

    T join() {
        assert(pthread_join(pid, nullptr) == 0);
        return get_res();
    }

    T get_res() const { return res; }

private:
    static_assert(sizeof(T) < (128 - sizeof(thread_body_t) - sizeof(pthread_t)),
                  "xx");
    char padding[128 - (sizeof(thread_body_t) + sizeof(T) + sizeof(pthread_t))];

    static void *pthread_bootstrap(void *p) {
        Thread *self = static_cast<Thread *>(p);
        self->res = self->core_func();
        return nullptr;
    }
};


struct alignas(128) Statics {

    typedef struct {
        uint64_t counter = 0;
        uint64_t counter1 = 0;
        uint64_t counter2 = 0;
        uint64_t counter3 = 0;
        double lat = 0;
    } data_t;
    data_t data;

    char pad[128 - sizeof(data)];

    void increment(int d = 1) { data.counter += d; }

    void increment_gap_1(uint64_t d) { data.counter1 += d; }
};

class Timer {
    std::chrono::time_point<std::chrono::steady_clock> start_time_;

public:
    static constexpr double no_timeout() {
        return std::numeric_limits<double>::max();
    }

    Timer(std::chrono::time_point<std::chrono::steady_clock> t =
    std::chrono::steady_clock::now())
            : start_time_(t) {}

    ~Timer() = default;

    template<typename T>
    bool timeout(double count) const {
        return passed<T>() >= count;
    }

    double passed_sec() const { return passed<std::chrono::seconds>(); }

    double passed_msec() const { return passed<std::chrono::microseconds>(); }

    template<typename T>
    double passed() const {
        return passed<T>(std::chrono::steady_clock::now());
    }

    template<typename T>
    double passed(std::chrono::time_point<std::chrono::steady_clock> tt) const {
        const auto elapsed = std::chrono::duration_cast<T>(tt - start_time_);
        return elapsed.count();
    }

    void reset() { start_time_ = std::chrono::steady_clock::now(); }

    Timer &operator=(Timer &) = default;
};

static double report_thpt(std::vector<Statics> &statics, int epoches, int tick_interval_us = 1000000, int batch = 1) {
    const int thread_num = statics.size();
    std::vector<Statics> old_statics(statics.size());

    Timer timer;
    epoches *= 1000000 / tick_interval_us;
    for (int epoch = 0; epoch < epoches; epoch += 1) {
        usleep(tick_interval_us);

        uint64_t sum = 0;
        // now report the throughput
        for (uint i = 0; i < statics.size(); ++i) {
            auto temp = statics[i].data.counter;
            sum += (temp - old_statics[i].data.counter);
            old_statics[i].data.counter = temp;
        }

        double passed_msec = timer.passed_msec();
        double thpt = static_cast<double>(sum) / passed_msec * 1000000.0;
        double lat = batch * thread_num * passed_msec / static_cast<double>(sum);
        compile_fence();
        timer.reset();

        std::cout << "epoch @ " << epoch << ": thpt: " << thpt << " reqs/sec."
                  << passed_msec << " msec passed since last epoch. "
                  << lat << " us" << std::endl;
    }
    return 0.0;
}