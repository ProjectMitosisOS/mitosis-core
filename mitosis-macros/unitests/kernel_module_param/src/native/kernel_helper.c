#include "kernel_helper.h"

#define BUF_LENGTH 256
#define DEFAULT_PERMISSION S_IRUSR|S_IWUSR|S_IRGRP|S_IROTH

long sample_long = 2;
module_param(sample_long, long, DEFAULT_PERMISSION);

int sample_int = 3;
module_param(sample_int, int, DEFAULT_PERMISSION);

char sample_arr[BUF_LENGTH] = {'I', 'P', 'A', 'D', 'S', '\0'};
char* sample_str = sample_arr;
module_param_string(sample_str, sample_arr, BUF_LENGTH, DEFAULT_PERMISSION);
