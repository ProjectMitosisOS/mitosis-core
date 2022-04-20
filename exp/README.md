Contains all of the experiments upon MITOSIS.

- `os-network`: Basic RDMA operation microbenchmark
    - `rc-connection`: TODO:xxxx
    
- `fork-micro`: All of the microbenchmark upon the fork critical path
    - `bench_prepare_time.cc` / `bench_prepare_time.py`: Exp of the influence of **different working-set memory** on the **prepare time** 
    - TODO: Figure in paper
    - `bench_exe_time_parent.cc` / `bench_exe_time_parent.py`: Exp of the influence of **different working-set memory** on the **execution time**
    - TODO: Figure in paper
            
    


Note: All of the child process could be boosted via the file `common/simple_child.cc`. And you can use the scripts under `scripts/` directly.