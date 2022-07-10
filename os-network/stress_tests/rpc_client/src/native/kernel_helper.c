#include "kernel_helper.h"

#define BUF_LENGTH 256
#define DEFAULT_PERMISSION S_IRUSR|S_IWUSR|S_IRGRP|S_IROTH

ulong server_qd_hint = 73;
module_param(server_qd_hint, ulong, DEFAULT_PERMISSION);

ulong client_qd_hint = 74;
module_param(client_qd_hint, ulong, DEFAULT_PERMISSION);

ulong server_service_id = 0;
module_param(server_service_id, ulong, DEFAULT_PERMISSION);

ulong client_service_id_base = 1;
module_param(client_service_id_base, ulong, DEFAULT_PERMISSION);

ulong thread_count = 1;
module_param(thread_count, ulong, DEFAULT_PERMISSION);

uint test_rpc_id = 73;
module_param(test_rpc_id, uint, DEFAULT_PERMISSION);

ulong session_id_base = 0;
module_param(session_id_base, ulong, DEFAULT_PERMISSION);

long running_secs = 10;
module_param(running_secs, long, DEFAULT_PERMISSION);

char gid_arr[BUF_LENGTH] = "fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c";
char* gid = gid_arr;
module_param_string(gid, gid_arr, BUF_LENGTH, DEFAULT_PERMISSION);
