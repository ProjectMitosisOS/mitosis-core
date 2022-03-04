#include "kernel_helper.h"

#define DEFAULT_PERMISSION S_IRUSR|S_IWUSR|S_IRGRP|S_IROTH

uint nthreads = 12;
module_param(nthreads, uint, DEFAULT_PERMISSION);

uint time = 10;
module_param(time, uint, DEFAULT_PERMISSION);

uint report_interval = 1;
module_param(report_interval, uint, DEFAULT_PERMISSION);
