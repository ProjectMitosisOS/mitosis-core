import mmap
import os
import sys
from ctypes import sizeof
from chameleon import PageTemplate

sys.path.append("../common")  # include outer path
from criu_wrapper import *

import syscall_lib
import argparse
import time


def get_timestamp():
    from datetime import datetime
    return str(datetime.now())

@tick_execution_time
def handler():
    template = PageTemplate("""
    <html>
    <head>
        <title>${title} - Simple Web Application</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 40px; }
            h1 { color: #333366; }
            ul { background-color: #f0f0f0; padding: 20px; }
            li { color: #663333; }
            footer { font-size: 0.8em; color: #777777; margin-top: 20px; }
        </style>
    </head>
    <body>
        <h1>Welcome to ${title}!</h1>
        <p>Hello, ${user}! Thank you for visiting.</p>
        <ul>
            <li tal:repeat="item items">
                ${item}
            </li>
        </ul>
        <footer>
            <p>Last updated at: ${timestamp()}</p>
        </footer>
    </body>
    </html>
    """)
    rendered = template.render(title="Chameleon Page", user="Guest", items=['Item 1', 'Item 2', 'Item 3'], timestamp=get_timestamp)
    # print(rendered)

@criu_bench
def bench():
    handler()
    
if __name__ == '__main__':
    bench()
