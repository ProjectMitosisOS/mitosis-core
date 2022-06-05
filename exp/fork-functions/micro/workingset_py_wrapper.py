from ctypes import *

import os
import argparse

def get_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("-working_set", type=int, default=16777216,
                    help="working set size")
    parser.add_argument("-app_name", type=str, default="micro", help="application name")
    args = parser.parse_args()  
    return args    

def main_micro():    
    args = get_args()

    print("start testing, ",args.working_set)
    so_workingset = cdll.LoadLibrary(os.getcwd()+ "/libmicro_function.so")
    so_workingset.init_buffer(args.working_set)
    so_workingset.handler("Cold", args.working_set)
                    
def main_hello():
    args = get_args()
    print("hello world")   

if __name__ == "__main__":
#    main_micro()
    main_hello()
                 
