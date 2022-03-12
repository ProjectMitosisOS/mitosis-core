# User-space libraries of MITOSIS 

The user-space libraries includes wrappers to `ioctrol`-based system calls, 
and user-space lean containers. 
Though in principle should implement the lean containers in the kernel, 
a user-space implementation is simpler, with some little costs for additional `fork`. 
