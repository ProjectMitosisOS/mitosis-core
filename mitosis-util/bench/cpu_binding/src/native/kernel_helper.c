#include "kernel_helper.h"

#define DEFAULT_PERMISSION S_IRUSR|S_IWUSR|S_IRGRP|S_IROTH

uint thread_count = 12;
module_param(thread_count, uint, DEFAULT_PERMISSION);

uint time = 10;
module_param(time, uint, DEFAULT_PERMISSION);

uint report_interval = 1;
module_param(report_interval, uint, DEFAULT_PERMISSION);
