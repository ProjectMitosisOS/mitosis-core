import igraph
import os
import sys

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

size = 128 * 1024

graph = igraph.Graph.Barabasi(size, 10)

@tick_execution_time
def lambda_handler():
    result = graph.pagerank()

@mitosis_bench
def bench():
    lambda_handler()


if __name__ == '__main__':
    bench()