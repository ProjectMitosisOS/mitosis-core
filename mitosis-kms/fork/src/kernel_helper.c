#include "kernel_helper.h"

#define DEFAULT_PERMISSION S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH

long mac_id = 0;
module_param(mac_id, long, DEFAULT_PERMISSION);
