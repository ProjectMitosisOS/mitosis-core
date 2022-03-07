#include "kernel_helper.h"

#define BUF_LENGTH 256
#define DEFAULT_PERMISSION S_IRUSR|S_IWUSR|S_IRGRP|S_IROTH

uint nic_count = 2;
module_param(nic_count, uint, DEFAULT_PERMISSION);

uint service_id_base = 50;
module_param(service_id_base, uint, DEFAULT_PERMISSION);
