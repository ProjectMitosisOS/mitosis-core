import igraph
import os
import sys

sys.path.append("../../common")  # include outer path
from func_bench_wrapper import *

size = 100000

graph = igraph.Graph.Barabasi(size, 10)

def lambda_handler():
    """
                    "{\"size\":\"100000\"}"

    :return:
    """
#    print("start page rank")
#    result = graph.pagerank(implementation="power",niter=1000)
    graph.pagerank()
#    print(graph)
    #print(graph.is_dag())
@func_exec_bench
def bench():
    lambda_handler()


if __name__ == '__main__':
    bench()