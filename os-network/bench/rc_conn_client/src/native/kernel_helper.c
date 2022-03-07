#include "kernel_helper.h"

#define BUF_LENGTH 256
#define DEFAULT_PERMISSION S_IRUSR|S_IWUSR|S_IRGRP|S_IROTH

uint remote_service_id_base = 50;
module_param(remote_service_id_base, uint, DEFAULT_PERMISSION);

uint nic_count = 2;
module_param(nic_count, uint, DEFAULT_PERMISSION);

uint running_secs = 30;
module_param(running_secs, uint, DEFAULT_PERMISSION);

uint report_interval = 1;
module_param(report_interval, uint, DEFAULT_PERMISSION);

uint thread_count = 12;
module_param(thread_count, uint, DEFAULT_PERMISSION);

char gids_arr[BUF_LENGTH] = {'I', 'P', 'A', 'D', 'S', '\0'};
char* gids = gids_arr;
module_param_string(gids, gids_arr, BUF_LENGTH, DEFAULT_PERMISSION);
