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

DEFINE_string(file_path, "", "path to the mmap file");
DEFINE_int64(working_set, 16777216, "working set size");
DEFINE_int64(exit_on_finish, 1, "whether to halt upon the execution finishes");
DEFINE_int64(random, 1, "whether to use random access");

extern "C"
{
    void init_buffer_w_ptr(char *ptr);
    void handler(const char *name, uint64_t workingset, int profile);
    void handler_random(const char *name, uint64_t workingset, int profile);
}

int main(int argc, char **argv)
{
    gflags::ParseCommandLineFlags(&argc, &argv, true);
    printf("working set %d, mb: %lluMB\n", FLAGS_working_set, FLAGS_working_set / (1024 * 1024));

    auto fd = open(FLAGS_file_path.c_str(), O_RDWR);
    assert(fd >= 0);

    // touch the file
    auto buffer = (char *)mmap(
        nullptr,
        FLAGS_working_set, // for one page length
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE, // to a private block of hardware memory
        fd,
        0);

    init_buffer_w_ptr(buffer);
    
    for (uint i = 0;i < 1;++i) { 
        printf("Execute @%d: \n", i);
        if (FLAGS_random) {
            handler_random("File access random", FLAGS_working_set, 1);
        } else { 
            handler("File access", FLAGS_working_set, 1);
        }
    }

    if (!FLAGS_exit_on_finish) { 
        while (1) { 
            sleep(1);
        }
    }

    close(fd);

    return 0;
}