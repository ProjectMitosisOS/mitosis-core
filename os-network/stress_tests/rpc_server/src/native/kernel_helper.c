#include "kernel_helper.h"

#define DEFAULT_PERMISSION S_IRUSR|S_IWUSR|S_IRGRP|S_IROTH

ulong default_qd_hint = 73;
module_param(default_qd_hint, ulong, DEFAULT_PERMISSION);

ulong test_rpc_id = 73;
module_param(test_rpc_id, ulong, DEFAULT_PERMISSION);

long running_secs = 30;
module_param(running_secs, long, DEFAULT_PERMISSION);
