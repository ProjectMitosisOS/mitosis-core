# Functions evaluated 

The following functions are from ServerlessBench: 
- [Image processing]()

The following functions are from the [FunctionBench](https://github.com/kmu-bigdata/serverless-faas-workbench):

- [chameleon](https://ipads.se.sjtu.edu.cn:1312/distributed-rdma-serverless/mitosis-benchmarks/serverless-functions/-/tree/main/rfork/chamelon)
- [matmul](https://ipads.se.sjtu.edu.cn:1312/distributed-rdma-serverless/mitosis-benchmarks/serverless-functions/-/tree/main/rfork/matmul)

- [pyaes](https://ipads.se.sjtu.edu.cn:1312/distributed-rdma-serverless/mitosis-benchmarks/serverless-functions/-/tree/main/rfork/pyaes)
- [json](https://github.com/ddps-lab/serverless-faas-workbench/blob/master/aws/network/json_dumps_loads/lambda_function.py)
- [linpack](https://ipads.se.sjtu.edu.cn:1312/distributed-rdma-serverless/mitosis-benchmarks/serverless-functions/-/tree/main/rfork/linpack)

The following functions are from [SaBE](https://github.com/spcl/serverless-benchmarks): 

- [pagerank](https://github.com/spcl/serverless-benchmarks/tree/master/benchmarks/500.scientific/501.graph-pagerank)
- [recognition](https://github.com/ucsdsysnet/faasnap/blob/main/rootfs/guest/recognition_handler.py)



Parent framework

```python
import os
import sys
import time

sys.path.append("../../common")  # include outer path
import syscall_lib
from bench import report

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'
import numpy as np

dump_key = 73


def handler():
    global start, end
    start = time.time()
    n = 100
    A = np.random.rand(n, n)
    B = np.random.rand(n, n)
    C = np.matmul(A, B)
    end = time.time()


def checkpoint():
    fd = syscall_lib.open()
    syscall_lib.call_prepare(fd, dump_key)


if __name__ == '__main__':
    global start, end
    handler()
    checkpoint()
    handler()
```

