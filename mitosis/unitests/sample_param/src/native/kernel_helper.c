#define BUF_LENGTH 256
#include "kernel_helper.h"

long sample_long = 2;
module_param(sample_long, long, 0644);

int sample_int = 3;
module_param(sample_int, int, 0644);

char sample_arr[BUF_LENGTH] = {'I', 'P', 'A', 'D', 'S', '\0'};
char* sample_str = sample_arr;
module_param_string(sample_str, sample_arr, BUF_LENGTH, 0644);
