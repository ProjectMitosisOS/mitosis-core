#define BUF_LENGTH 256
#include "kernel_helper.h"

char SAMPLE_ARR[BUF_LENGTH] = {'\0'};
char* SAMPLE = SAMPLE_ARR;
module_param_string(SAMPLE, SAMPLE_ARR, BUF_LENGTH, 0);

long SAMPLE_LONG = 2;
module_param(SAMPLE_LONG, long, 0644);
