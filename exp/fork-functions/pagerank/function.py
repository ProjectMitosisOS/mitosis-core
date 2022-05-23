import igraph
import os
import sys

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

size = 50000

graph = igraph.Graph.Barabasi(size, 10)

@tick_execution_time
def lambda_handler():
    """
                    "{\"size\":\"100000\"}"

    :return:
    """
    result = graph.pagerank()

@mitosis_bench
def bench():
    lambda_handler()


if __name__ == '__main__':
    bench()