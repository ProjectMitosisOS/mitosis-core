#include "kernel_helper.h"

#define BUF_LENGTH 256
#define DEFAULT_PERMISSION S_IRUSR|S_IWUSR|S_IRGRP|S_IROTH

long nic_count = 2;
module_param(nic_count, long, DEFAULT_PERMISSION);

long service_id_base = 50;
module_param(service_id_base, long, DEFAULT_PERMISSION);
