#include "../core/lean_container.h"
#include <stdio.h>
#include <assert.h>

int main() {
    int ret, pid;
    pid = setup_cached_namespace();
    assert(pid > 0);
    ret = remove_cached_namespace(pid);
    assert(ret == 0);
    return 0;
}
