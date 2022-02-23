#define BUF_LENGTH 256
#include "kernel_helper.h"

long sample_long = 2;
module_param(sample_long, long, 0644);

int sample_int = 3;
module_param(sample_int, int, 0644);
