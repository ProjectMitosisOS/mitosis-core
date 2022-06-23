import argparse
import socket

# python trigger.py -child_hosts=val04 -process=2
import time

import pandas as pd

from util import *

req = {"body": {"portfolioType": "S&P", "portfolio": "1234"}}


def public_data(event):
    """
    Fetch public market data from yahoo finance api
    :return:
    """
    startTime = 1000 * time.time()
    externalServicesTime = 0
    portfolioType = event['body']['portfolioType']

    tickers_for_portfolio_types = {'S&P': ['GOOG', 'AMZN', 'MSFT', 'SVFAU', 'AB', 'ABC', 'ABCB']}
    # stocks_list = list(pd.read_csv("stocks.csv")['Symbol'])
    tickers = tickers_for_portfolio_types[portfolioType]

    prices = {}
    whole_set = pd.read_csv("yfinance.csv")
    for ticker in tickers:
        # Get last closing price
        tickTime = 1000 * time.time()
        # data = pd.read_csv("yfinance.csv")
        externalServicesTime += 1000 * time.time() - tickTime
        prices[ticker] = whole_set['Close'].unique()[0]

    # prices = {'GOOG': 1732.38, 'AMZN': 3185.27, 'MSFT': 221.02}
    response = {'statusCode': 200,
                'body': {'marketData': prices, 'whole_set': whole_set}}

    endTime = 1000 * time.time()
    return timestamp(response, event, startTime, endTime, externalServicesTime)


parser = argparse.ArgumentParser()
parser.add_argument("-child_hosts", type=str, default="", help="rpc server host")
parser.add_argument("-process", type=int, default=1, help="rpc parallel num")
args = parser.parse_args()
process = args.process
child_hosts = str(args.child_hosts).split(' ')
master_port = 7000
parent_port = 8000
trigger_port = 9000

s_udp = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s_udp.sendto(str(process).encode(), ('', master_port))
public_data(req)
for i in range(process):
    s_udp.sendto(b"data", ('', parent_port + i))
s_udp.close()
