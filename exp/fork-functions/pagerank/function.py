import igraph
import os
import sys

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

size = 100000

graph = igraph.Graph.Barabasi(size, 10)

@tick_execution_time
def lambda_handler():
    """
                    "{\"size\":\"100000\"}"

    :return:
    """
#    print("start page rank")
#    result = graph.pagerank(implementation="power",niter=1000)
    result = graph.pagerank()
    print(result[0])
#    print(graph)
    #print(graph.is_dag())

@mitosis_bench
def bench():
    lambda_handler()


if __name__ == '__main__':
    bench()